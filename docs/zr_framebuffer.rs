//! src/core/zr_framebuffer.rs — Framebuffer + painter clipping + core draw ops.
//!
//! Implements a bounded, deterministic in-memory framebuffer with a clip stack
//! and invariant-safe drawing primitives. Ops avoid per-call allocations and ensure
//! wide glyphs are never split across cell boundaries.

use core::cmp;
use core::mem;
use core::ptr;

use crate::unicode::zr_grapheme::{zr_grapheme_iter_init, zr_grapheme_next, ZrGrapheme, ZrGraphemeIter};
use crate::unicode::zr_utf8::zr_utf8_decode_one;
use crate::unicode::zr_width::{zr_width_grapheme_utf8, zr_width_policy_default};
use crate::util::zr_checked::*;
use crate::util::zr_macros::*;
use crate::core::zr_fb::*; // for ZrFb, ZrCell, ZrStyle, ZrFbLink, ZrRect, ZrFbPainter, ZrResult, etc.

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/// U+FFFD replacement character in UTF-8.
const ZR_UTF8_REPLACEMENT: [u8; 3] = [0xEF, 0xBF, 0xBD];
const ZR_UTF8_REPLACEMENT_LEN: usize = 3;

/// Reject terminal control scalars in wrapper-provided link URI/ID text.
const ZR_FB_UTF8_ASCII_CONTROL_MAX: u32 = 0x20;
const ZR_FB_UTF8_ASCII_DEL: u32 = 0x7F;
const ZR_FB_UTF8_C1_MIN: u32 = 0x80;
const ZR_FB_UTF8_C1_MAX_EXCL: u32 = 0xA0;

const ZR_FB_LINKS_INITIAL_CAP: usize = 8;
const ZR_FB_LINK_BYTES_INITIAL_CAP: usize = 256;
const ZR_FB_LINK_ENTRY_MAX_BYTES: usize = ZR_FB_LINK_URI_MAX_BYTES + ZR_FB_LINK_ID_MAX_BYTES;

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn zr_fb_utf8_grapheme_bytes_safe_for_terminal(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let mut off = 0;
    while off < bytes.len() {
        let d = zr_utf8_decode_one(&bytes[off..]);
        if d.size == 0 || d.valid == 0 {
            return false;
        }
        let s = d.scalar;
        if s < ZR_FB_UTF8_ASCII_CONTROL_MAX
            || s == ZR_FB_UTF8_ASCII_DEL
            || (s >= ZR_FB_UTF8_C1_MIN && s < ZR_FB_UTF8_C1_MAX_EXCL)
        {
            return false;
        }
        off += d.size as usize;
    }
    true
}

fn zr_style_default() -> ZrStyle {
    ZrStyle {
        fg_rgb: 0,
        bg_rgb: 0,
        attrs: 0,
        reserved: 0,
        underline_rgb: 0,
        link_ref: 0,
    }
}

fn zr_fb_links_zero(fb: &mut ZrFb) {
    fb.links = Vec::new();
    fb.link_bytes = Vec::new();
}

fn zr_fb_links_release(fb: &mut ZrFb) {
    fb.links.clear();
    fb.link_bytes.clear();
}

fn zr_fb_links_reset(fb: &mut ZrFb) {
    fb.links.clear();
    fb.link_bytes.clear();
}

fn zr_fb_link_span_eq(
    fb: &ZrFb,
    e: &ZrFbLink,
    uri: &[u8],
    id: &[u8],
) -> bool {
    if e.uri_len as usize != uri.len() || e.id_len as usize != id.len() {
        return false;
    }
    if !uri.is_empty() && &fb.link_bytes[e.uri_off as usize..][..uri.len()] != uri {
        return false;
    }
    if !id.is_empty() && &fb.link_bytes[e.id_off as usize..][..id.len()] != id {
        return false;
    }
    true
}

fn zr_fb_links_ensure_cap(fb: &mut ZrFb, need_links: usize) -> ZrResult {
    if need_links <= fb.links.capacity() {
        return ZR_OK;
    }
    let mut next_cap = if fb.links.capacity() == 0 {
        ZR_FB_LINKS_INITIAL_CAP
    } else {
        fb.links.capacity()
    };
    while next_cap < need_links {
        if next_cap > (usize::MAX / 2) {
            next_cap = need_links;
            break;
        }
        next_cap *= 2;
    }
    fb.links.reserve(next_cap - fb.links.len());
    // In fallible allocation we'd check, but reserve cannot fail; for OOM we'd need a custom allocator.
    // For portability we keep the same semantics: if reserve fails (OOM), return error.
    // However, Rust's Vec::reserve panics on OOM. To match C behavior, we could try to allocate manually,
    // but for translation we assume OOM is unrecoverable and keep the panic. Alternatively, we can use try_reserve.
    if fb.links.try_reserve(next_cap - fb.links.len()).is_err() {
        return ZR_ERR_OOM;
    }
    ZR_OK
}

fn zr_fb_link_bytes_ensure_cap(fb: &mut ZrFb, need_bytes: usize) -> ZrResult {
    if need_bytes <= fb.link_bytes.capacity() {
        return ZR_OK;
    }
    let mut next_cap = if fb.link_bytes.capacity() == 0 {
        ZR_FB_LINK_BYTES_INITIAL_CAP
    } else {
        fb.link_bytes.capacity()
    };
    while next_cap < need_bytes {
        if next_cap > (usize::MAX / 2) {
            next_cap = need_bytes;
            break;
        }
        next_cap *= 2;
    }
    if fb.link_bytes.try_reserve(next_cap - fb.link_bytes.len()).is_err() {
        return ZR_ERR_OOM;
    }
    ZR_OK
}

// -----------------------------------------------------------------------------
// Public link management
// -----------------------------------------------------------------------------

pub fn zr_fb_links_clone_from(dst: &mut ZrFb, src: &ZrFb) -> ZrResult {
    if src.links.is_empty() {
        zr_fb_links_reset(dst);
        return ZR_OK;
    }
    if src.link_bytes.is_empty() && !src.link_bytes.is_empty() {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    zr_fb_links_ensure_cap(dst, src.links.len())?;
    zr_fb_link_bytes_ensure_cap(dst, src.link_bytes.len())?;
    dst.links.clear();
    dst.links.extend_from_slice(&src.links);
    dst.link_bytes.clear();
    dst.link_bytes.extend_from_slice(&src.link_bytes);
    ZR_OK
}

pub fn zr_fb_copy_damage_rects(
    dst: &mut ZrFb,
    src: &ZrFb,
    rects: &[ZrDamageRect],
) -> ZrResult {
    if dst.cols != src.cols || dst.rows != src.rows {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    if rects.is_empty() || dst as *const _ == src as *const _ {
        return ZR_OK;
    }
    if dst.cols == 0 || dst.rows == 0 {
        return ZR_OK;
    }
    if dst.cells.is_empty() || src.cells.is_empty() {
        return ZR_ERR_INVALID_ARGUMENT;
    }

    // Copy links first because cell link_ref indices will be the same.
    zr_fb_links_clone_from(dst, src)?;

    let max_x = dst.cols - 1;
    let max_y = dst.rows - 1;
    for r in rects {
        let mut x0 = r.x0;
        let mut y0 = r.y0;
        let mut x1 = r.x1;
        let mut y1 = r.y1;
        if x0 > x1 || y0 > y1 {
            continue;
        }
        if x0 > max_x || y0 > max_y {
            continue;
        }
        x1 = cmp::min(x1, max_x);
        y1 = cmp::min(y1, max_y);
        if x0 > x1 || y0 > y1 {
            continue;
        }
        let span_cells = (x1 - x0 + 1) as usize;
        let span_bytes = span_cells * mem::size_of::<ZrCell>();
        for y in y0..=y1 {
            let row_start = (y as usize) * (dst.cols as usize) + (x0 as usize);
            let src_ptr = src.cells.as_ptr().add(row_start) as *const u8;
            let dst_ptr = dst.cells.as_mut_ptr().add(row_start) as *mut u8;
            unsafe {
                ptr::copy_nonoverlapping(src_ptr, dst_ptr, span_bytes);
            }
        }
    }
    ZR_OK
}

// -----------------------------------------------------------------------------
// Rectangle helpers
// -----------------------------------------------------------------------------

fn zr_rect_empty() -> ZrRect {
    ZrRect { x: 0, y: 0, w: 0, h: 0 }
}

fn zr_rect_right(r: ZrRect) -> i64 {
    r.x as i64 + r.w as i64
}
fn zr_rect_bottom(r: ZrRect) -> i64 {
    r.y as i64 + r.h as i64
}

fn zr_fb_bounds_rect(fb: &ZrFb) -> ZrRect {
    ZrRect {
        x: 0,
        y: 0,
        w: fb.cols as i32,
        h: fb.rows as i32,
    }
}

fn zr_rect_intersect(a: ZrRect, b: ZrRect) -> ZrRect {
    if a.w <= 0 || a.h <= 0 || b.w <= 0 || b.h <= 0 {
        return zr_rect_empty();
    }
    let ax2 = zr_rect_right(a);
    let ay2 = zr_rect_bottom(a);
    let bx2 = zr_rect_right(b);
    let by2 = zr_rect_bottom(b);
    let x1 = cmp::max(a.x, b.x);
    let y1 = cmp::max(a.y, b.y);
    let x2 = cmp::min(ax2, bx2);
    let y2 = cmp::min(ay2, by2);
    let w = (x2 - x1 as i64) as i32;
    let h = (y2 - y1 as i64) as i32;
    if w <= 0 || h <= 0 {
        zr_rect_empty()
    } else {
        ZrRect { x: x1, y: y1, w, h }
    }
}

fn zr_rect_contains(r: ZrRect, x: i32, y: i32) -> bool {
    if r.w <= 0 || r.h <= 0 {
        return false;
    }
    if x < r.x || y < r.y {
        return false;
    }
    let x2 = r.x as i64 + r.w as i64;
    let y2 = r.y as i64 + r.h as i64;
    (x as i64) < x2 && (y as i64) < y2
}

fn zr_fb_has_backing(fb: &ZrFb) -> bool {
    !fb.cells.is_empty() && fb.cols != 0 && fb.rows != 0
}

fn zr_fb_cell_index(fb: &ZrFb, x: u32, y: u32) -> Option<usize> {
    if !zr_fb_has_backing(fb) || x >= fb.cols || y >= fb.rows {
        return None;
    }
    let row_start = (y as usize)
        .checked_mul(fb.cols as usize)
        .and_then(|v| v.checked_add(x as usize));
    row_start
}

fn zr_cell_set_space(cell: &mut ZrCell, style: ZrStyle) {
    cell.glyph = [0; ZR_CELL_GLYPH_MAX];
    cell.glyph[0] = b' ';
    cell.glyph_len = 1;
    cell.width = 1;
    cell._pad0 = 0;
    cell.style = style;
}

fn zr_cell_set_grapheme_width1(cell: &mut ZrCell, bytes: &[u8], style: ZrStyle) {
    cell.glyph = [0; ZR_CELL_GLYPH_MAX];
    let copy_len = cmp::min(bytes.len(), ZR_CELL_GLYPH_MAX);
    cell.glyph[..copy_len].copy_from_slice(&bytes[..copy_len]);
    cell.glyph_len = copy_len as u8;
    cell.width = 1;
    cell._pad0 = 0;
    cell.style = style;
}

fn zr_cell_set_continuation(cell: &mut ZrCell, style: ZrStyle) {
    cell.glyph = [0; ZR_CELL_GLYPH_MAX];
    cell.glyph_len = 0;
    cell.width = 0;
    cell._pad0 = 0;
    cell.style = style;
}

fn zr_cell_is_continuation(cell: &ZrCell) -> bool {
    cell.width == 0
}

fn zr_cell_is_wide_lead(cell: &ZrCell) -> bool {
    cell.width == 2
}

fn zr_i64_fits_i32(v: i64) -> bool {
    v >= i32::MIN as i64 && v <= i32::MAX as i64
}

// -----------------------------------------------------------------------------
// Framebuffer init/resize/clear
// -----------------------------------------------------------------------------

pub fn zr_fb_init(fb: &mut ZrFb, cols: u32, rows: u32) -> ZrResult {
    fb.cols = 0;
    fb.rows = 0;
    fb.cells = Vec::new();
    zr_fb_links_zero(fb);
    zr_fb_resize(fb, cols, rows)
}

pub fn zr_fb_release(fb: &mut ZrFb) {
    fb.cells.clear();
    zr_fb_links_release(fb);
    fb.cols = 0;
    fb.rows = 0;
}

pub fn zr_fb_cell(fb: &mut ZrFb, x: u32, y: u32) -> Option<&mut ZrCell> {
    let idx = zr_fb_cell_index(fb, x, y)?;
    fb.cells.get_mut(idx)
}

pub fn zr_fb_cell_const(fb: &ZrFb, x: u32, y: u32) -> Option<&ZrCell> {
    let idx = zr_fb_cell_index(fb, x, y)?;
    fb.cells.get(idx)
}

fn zr_fb_links_slots_limit(fb: &ZrFb) -> usize {
    if fb.cols == 0 || fb.rows == 0 {
        return 1;
    }
    let cell_count = (fb.cols as usize).saturating_mul(fb.rows as usize);
    // Allow (live cells * 2) + 1
    cell_count.saturating_mul(2).saturating_add(1)
}

fn zr_fb_link_bytes_limit(slots_limit: usize) -> usize {
    slots_limit.saturating_mul(ZR_FB_LINK_ENTRY_MAX_BYTES)
}

fn zr_fb_links_compact_live(fb: &mut ZrFb) -> ZrResult {
    if fb.links.is_empty() {
        fb.link_bytes.clear();
        return ZR_OK;
    }
    if !zr_fb_has_backing(fb) {
        zr_fb_links_reset(fb);
        return ZR_OK;
    }
    let cell_count = (fb.cols as usize)
        .checked_mul(fb.rows as usize)
        .ok_or(ZR_ERR_LIMIT)?;
    if cell_count == 0 {
        zr_fb_links_reset(fb);
        return ZR_OK;
    }

    let mut remap = vec![0u32; fb.links.len() + 1];
    // Mark live refs
    for cell in &fb.cells {
        let ref_ = cell.style.link_ref;
        if ref_ == 0 {
            continue;
        }
        if (ref_ as usize) <= fb.links.len() {
            remap[ref_ as usize] = u32::MAX;
        } else {
            // Out of range, clear it
            // We cannot mutate cell here because of shared borrow; we'll fix later.
        }
    }
    // Fix out-of-range refs (needs mutable access)
    for cell in &mut fb.cells {
        let ref_ = cell.style.link_ref;
        if ref_ != 0 && (ref_ as usize) > fb.links.len() {
            cell.style.link_ref = 0;
        }
    }

    let old_links_len = fb.links.len();
    let mut new_links_len = 0;
    let mut new_link_bytes_len = 0;
    let mut new_links = Vec::with_capacity(old_links_len);
    let mut new_bytes = Vec::new();

    for i in 0..old_links_len {
        let old_ref = i + 1;
        if remap[old_ref] != u32::MAX {
            continue;
        }
        let src = &fb.links[i];
        let span_len = src.uri_len as usize + src.id_len as usize;
        if src.uri_off as usize + src.uri_len as usize > fb.link_bytes.len()
            || src.id_off as usize + src.id_len as usize > fb.link_bytes.len()
        {
            return ZR_ERR_FORMAT;
        }
        if src.id_off != src.uri_off + src.uri_len {
            return ZR_ERR_FORMAT;
        }
        // Copy data
        let start = new_bytes.len();
        if span_len != 0 {
            let data_start = src.uri_off as usize;
            new_bytes.extend_from_slice(&fb.link_bytes[data_start..][..span_len]);
        }
        let dst = ZrFbLink {
            uri_off: start as u32,
            uri_len: src.uri_len,
            id_off: (start + src.uri_len as usize) as u32,
            id_len: src.id_len,
        };
        new_links.push(dst);
        remap[old_ref] = (new_links_len + 1) as u32;
        new_links_len += 1;
        new_link_bytes_len = new_bytes.len();
    }

    // Remap cell references
    for cell in &mut fb.cells {
        let ref_ = cell.style.link_ref;
        if ref_ == 0 {
            continue;
        }
        let new_ref = remap.get(ref_ as usize).copied().unwrap_or(0);
        cell.style.link_ref = new_ref;
    }

    fb.links = new_links;
    fb.link_bytes = new_bytes;
    ZR_OK
}

pub fn zr_fb_link_intern(
    fb: &mut ZrFb,
    uri: &[u8],
    id: &[u8],
) -> ZrResult<u32> {
    if uri.is_empty() || uri.len() > ZR_FB_LINK_URI_MAX_BYTES {
        return Err(ZR_ERR_LIMIT);
    }
    if id.len() > ZR_FB_LINK_ID_MAX_BYTES {
        return Err(ZR_ERR_LIMIT);
    }
    // Check existing
    for (i, e) in fb.links.iter().enumerate() {
        if zr_fb_link_span_eq(fb, e, uri, id) {
            return Ok((i + 1) as u32);
        }
    }

    let need_links = fb.links.len() + 1;
    let need_bytes = fb.link_bytes.len() + uri.len() + id.len();

    let slots_limit = zr_fb_links_slots_limit(fb);
    let bytes_limit = zr_fb_link_bytes_limit(slots_limit);
    let would_grow = need_links > fb.links.capacity() || need_bytes > fb.link_bytes.capacity();
    if would_grow || need_links > slots_limit || need_bytes > bytes_limit {
        zr_fb_links_compact_live(fb)?;
        let need_links2 = fb.links.len() + 1;
        let need_bytes2 = fb.link_bytes.len() + uri.len() + id.len();
        if need_links2 > slots_limit || need_bytes2 > bytes_limit {
            return Err(ZR_ERR_LIMIT);
        }
        // Recompute after compact
    }

    zr_fb_links_ensure_cap(fb, need_links)?;
    zr_fb_link_bytes_ensure_cap(fb, need_bytes)?;

    let uri_off = fb.link_bytes.len() as u32;
    fb.link_bytes.extend_from_slice(uri);
    let id_off = fb.link_bytes.len() as u32;
    fb.link_bytes.extend_from_slice(id);
    let e = ZrFbLink {
        uri_off,
        uri_len: uri.len() as u32,
        id_off,
        id_len: id.len() as u32,
    };
    fb.links.push(e);
    Ok(fb.links.len() as u32)
}

pub fn zr_fb_link_lookup(
    fb: &ZrFb,
    link_ref: u32,
) -> ZrResult<(&[u8], &[u8])> {
    if link_ref == 0 || (link_ref as usize) > fb.links.len() {
        return Err(ZR_ERR_FORMAT);
    }
    let e = &fb.links[(link_ref - 1) as usize];
    let uri_start = e.uri_off as usize;
    let uri_end = uri_start + e.uri_len as usize;
    let id_start = e.id_off as usize;
    let id_end = id_start + e.id_len as usize;
    if uri_end > fb.link_bytes.len() || id_end > fb.link_bytes.len() {
        return Err(ZR_ERR_FORMAT);
    }
    let uri_slice = &fb.link_bytes[uri_start..uri_end];
    let id_slice = &fb.link_bytes[id_start..id_end];
    Ok((uri_slice, id_slice))
}

pub fn zr_fb_clear(fb: &mut ZrFb, style: Option<&ZrStyle>) -> ZrResult {
    zr_fb_links_reset(fb);
    if !zr_fb_has_backing(fb) {
        return ZR_OK;
    }
    let style = style.copied().unwrap_or_else(zr_style_default);
    for cell in &mut fb.cells {
        zr_cell_set_space(cell, style);
    }
    ZR_OK
}

fn zr_fb_alloc_cells(cols: u32, rows: u32) -> ZrResult<Vec<ZrCell>> {
    if cols == 0 || rows == 0 {
        return Ok(Vec::new());
    }
    let count = (cols as usize)
        .checked_mul(rows as usize)
        .ok_or(ZR_ERR_LIMIT)?;
    let mut cells = Vec::with_capacity(count);
    cells.resize_with(count, || {
        let mut c = unsafe { mem::zeroed() };
        zr_cell_set_space(&mut c, zr_style_default());
        c
    });
    Ok(cells)
}

fn zr_fb_repair_row(fb: &mut ZrFb, y: u32) {
    if !zr_fb_has_backing(fb) || y >= fb.rows || fb.cols == 0 {
        return;
    }
    for x in 0..fb.cols {
        let Some(cell) = zr_fb_cell(fb, x, y) else { continue };
        if zr_cell_is_continuation(cell) {
            if x == 0 {
                zr_cell_set_space(cell, cell.style);
                continue;
            }
            let lead = zr_fb_cell_const(fb, x - 1, y);
            if !lead.map_or(false, zr_cell_is_wide_lead) {
                zr_cell_set_space(cell, cell.style);
            }
        } else if zr_cell_is_wide_lead(cell) {
            if x + 1 >= fb.cols {
                zr_cell_set_grapheme_width1(cell, &ZR_UTF8_REPLACEMENT, cell.style);
                continue;
            }
            let cont = zr_fb_cell(fb, x + 1, y);
            if let Some(cont) = cont {
                if !zr_cell_is_continuation(cont) {
                    zr_cell_set_grapheme_width1(cell, &ZR_UTF8_REPLACEMENT, cell.style);
                    zr_cell_set_space(cont, cell.style);
                }
            }
        }
    }
}

pub fn zr_fb_resize(fb: &mut ZrFb, cols: u32, rows: u32) -> ZrResult {
    if cols == fb.cols && rows == fb.rows {
        return ZR_OK;
    }
    let mut new_cells = zr_fb_alloc_cells(cols, rows)?;
    let mut tmp = ZrFb {
        cols,
        rows,
        cells: new_cells,
        links: Vec::new(),
        link_bytes: Vec::new(),
    };
    zr_fb_clear(&mut tmp, None)?;
    if zr_fb_has_backing(fb) {
        zr_fb_links_clone_from(&mut tmp, fb)?;
        let copy_cols = cmp::min(fb.cols, tmp.cols);
        let copy_rows = cmp::min(fb.rows, tmp.rows);
        for y in 0..copy_rows {
            for x in 0..copy_cols {
                if let (Some(src), Some(dst)) = (zr_fb_cell_const(fb, x, y), zr_fb_cell(&mut tmp, x, y)) {
                    *dst = *src;
                }
            }
            zr_fb_repair_row(&mut tmp, y);
        }
    }
    // Swap
    mem::swap(fb, &mut tmp);
    ZR_OK
}

// -----------------------------------------------------------------------------
// Painter and clipping
// -----------------------------------------------------------------------------

pub fn zr_fb_painter_begin(
    p: &mut ZrFbPainter,
    fb: &mut ZrFb,
    clip_stack: &mut [ZrRect],
) -> ZrResult {
    if clip_stack.is_empty() {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    p.fb = fb;
    p.clip_stack = clip_stack;
    p.clip_cap = clip_stack.len();
    p.clip_len = 1;
    p.clip_stack[0] = zr_fb_bounds_rect(fb);
    ZR_OK
}

fn zr_painter_clip_cur(p: &ZrFbPainter) -> ZrRect {
    if p.clip_len == 0 {
        return zr_rect_empty();
    }
    p.clip_stack[p.clip_len - 1]
}

pub fn zr_fb_clip_push(p: &mut ZrFbPainter, clip: ZrRect) -> ZrResult {
    if p.clip_len >= p.clip_cap {
        return ZR_ERR_LIMIT;
    }
    let bounds = zr_fb_bounds_rect(p.fb);
    let next = zr_rect_intersect(bounds, clip);
    let next = zr_rect_intersect(zr_painter_clip_cur(p), next);
    p.clip_stack[p.clip_len] = next;
    p.clip_len += 1;
    ZR_OK
}

pub fn zr_fb_clip_pop(p: &mut ZrFbPainter) -> ZrResult {
    if p.clip_len <= 1 {
        return ZR_ERR_LIMIT;
    }
    p.clip_len -= 1;
    ZR_OK
}

fn zr_painter_can_touch(p: &ZrFbPainter, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 {
        return false;
    }
    let (ux, uy) = (x as u32, y as u32);
    if ux >= p.fb.cols || uy >= p.fb.rows {
        return false;
    }
    zr_rect_contains(zr_painter_clip_cur(p), x, y)
}


fn zr_painter_can_write_width2(p: &ZrFbPainter, x: u32, y: u32) -> bool {
    if x + 1 >= p.fb.cols {
        return false;
    }
    zr_painter_can_touch(p, x as i32, y as i32)
        && zr_painter_can_touch(p, (x + 1) as i32, y as i32)
}

pub fn zr_fb_put_grapheme(
    p: &mut ZrFbPainter,
    x: i32,
    y: i32,
    bytes: &[u8],
    width: u8,
    style: &ZrStyle,
) -> ZrResult {
    if !zr_fb_has_backing(p.fb) {
        return ZR_OK;
    }
    if width != 1 && width != 2 {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    let mut out_bytes = bytes;
    let mut out_len = bytes.len();
    let mut try_wide = width == 2;

    // Canonicalize empty graphemes to space
    if out_len == 0 {
        out_bytes = b" ";
        out_len = 1;
        try_wide = false;
    }
    if out_len > ZR_CELL_GLYPH_MAX {
        out_bytes = &ZR_UTF8_REPLACEMENT;
        out_len = ZR_UTF8_REPLACEMENT_LEN;
        try_wide = false;
    }
    if !zr_fb_utf8_grapheme_bytes_safe_for_terminal(&out_bytes[..out_len]) {
        out_bytes = &ZR_UTF8_REPLACEMENT;
        out_len = ZR_UTF8_REPLACEMENT_LEN;
        try_wide = false;
    }
    if x < 0 || y < 0 {
        return ZR_OK;
    }
    let ux = x as u32;
    let uy = y as u32;
    if ux >= p.fb.cols || uy >= p.fb.rows {
        return ZR_OK;
    }
    if try_wide {
        if zr_painter_write_width2(p, ux, uy, &out_bytes[..out_len], *style) {
            return ZR_OK;
        }
        // Fallback to replacement
        out_bytes = &ZR_UTF8_REPLACEMENT;
        out_len = ZR_UTF8_REPLACEMENT_LEN;
    }
    let _ = zr_painter_write_width1(p, ux, uy, &out_bytes[..out_len], *style);
    ZR_OK
}

/// Overwrite a single cell with a width-1 grapheme while preserving wide invariants.
fn zr_painter_write_width1(
    p: &mut ZrFbPainter,
    x: u32,
    y: u32,
    bytes: &[u8],
    style: ZrStyle,
) -> bool {
    if !zr_painter_can_touch(p, x as i32, y as i32) {
        return false;
    }
    let Some(cell) = zr_fb_cell(p.fb, x, y) else { return false };
    // If writing into a continuation, clear both cells
    if zr_cell_is_continuation(cell) {
        if x == 0 {
            return false;
        }
        let Some(lead) = zr_fb_cell(p.fb, x - 1, y) else { return false };
        zr_cell_set_space(lead, style);
        zr_cell_set_space(cell, style);
    }
    // If overwriting a wide lead, clear its continuation
    if zr_cell_is_wide_lead(cell) {
        if x + 1 >= p.fb.cols {
            return false;
        }
        let Some(cont) = zr_fb_cell(p.fb, x + 1, y) else { return false };
        zr_cell_set_space(cont, style);
    }
    // If next cell is a continuation (wide lead), clear it
    if x + 1 < p.fb.cols {
        if let Some(next) = zr_fb_cell(p.fb, x + 1, y) {
            if zr_cell_is_continuation(next) {
                zr_cell_set_space(next, style);
            }
        }
    }
    zr_cell_set_grapheme_width1(cell, bytes, style);
    true
}

/// Write a width-2 grapheme (lead + continuation) while preserving invariants.
fn zr_painter_write_width2(
    p: &mut ZrFbPainter,
    x: u32,
    y: u32,
    bytes: &[u8],
    style: ZrStyle,
) -> bool {
    if !zr_painter_can_write_width2(p, x, y) {
        return false;
    }
    let space = b" ";
    if !zr_painter_write_width1(p, x, y, space, style) {
        return false;
    }
    if !zr_painter_write_width1(p, x + 1, y, space, style) {
        return false;
    }
    let Some(c0) = zr_fb_cell(p.fb, x, y) else { return false };
    let Some(c1) = zr_fb_cell(p.fb, x + 1, y) else { return false };
    zr_cell_set_grapheme_width1(c0, bytes, style);
    c0.width = 2;
    zr_cell_set_continuation(c1, style);
    true
}

// -----------------------------------------------------------------------------
// Drawing primitives
// -----------------------------------------------------------------------------

pub fn zr_fb_fill_rect(p: &mut ZrFbPainter, r: ZrRect, style: &ZrStyle) -> ZrResult {
    if r.w <= 0 || r.h <= 0 {
        return ZR_OK;
    }
    if !zr_fb_has_backing(p.fb) {
        return ZR_OK;
    }
    let bounds = zr_fb_bounds_rect(p.fb);
    let clip = zr_painter_clip_cur(p);
    let mut rr = zr_rect_intersect(r, bounds);
    rr = zr_rect_intersect(rr, clip);
    if rr.w <= 0 || rr.h <= 0 {
        return ZR_OK;
    }
    let space = b" ";
    for y in rr.y..rr.y + rr.h {
        for x in rr.x..rr.x + rr.w {
            if x < 0 || y < 0 {
                continue;
            }
            let _ = zr_painter_write_width1(p, x as u32, y as u32, space, *style);
        }
    }
    ZR_OK
}

fn zr_draw_repeat_ascii(p: &mut ZrFbPainter, x: i32, y: i32, len: i32, ch: u8, style: &ZrStyle) -> ZrResult {
    if len <= 0 {
        return ZR_OK;
    }
    for i in 0..len {
        let xx = x + i;
        if xx < 0 {
            continue;
        }
        let _ = zr_fb_put_grapheme(p, xx, y, &[ch], 1, style);
    }
    ZR_OK
}

pub fn zr_fb_draw_hline(p: &mut ZrFbPainter, x: i32, y: i32, len: i32, style: &ZrStyle) -> ZrResult {
    zr_draw_repeat_ascii(p, x, y, len, b'-', style)
}

pub fn zr_fb_draw_vline(p: &mut ZrFbPainter, x: i32, y: i32, len: i32, style: &ZrStyle) -> ZrResult {
    if len <= 0 {
        return ZR_OK;
    }
    for i in 0..len {
        let yy = y + i;
        let _ = zr_fb_put_grapheme(p, x, yy, b"|", 1, style);
    }
    ZR_OK
}

pub fn zr_fb_draw_box(p: &mut ZrFbPainter, r: ZrRect, style: &ZrStyle) -> ZrResult {
    if r.w <= 0 || r.h <= 0 {
        return ZR_OK;
    }
    let x1 = r.x as i64;
    let y1 = r.y as i64;
    let x2 = x1 + r.w as i64 - 1;
    let y2 = y1 + r.h as i64 - 1;
    let draw_char = |px: i32, py: i32, ch: u8| {
        let _ = zr_fb_put_grapheme(p, px, py, &[ch], 1, style);
    };
    if zr_i64_fits_i32(x1) && zr_i64_fits_i32(y1) {
        draw_char(x1 as i32, y1 as i32, b'+');
    }
    if zr_i64_fits_i32(x2) && zr_i64_fits_i32(y1) {
        draw_char(x2 as i32, y1 as i32, b'+');
    }
    if zr_i64_fits_i32(x1) && zr_i64_fits_i32(y2) {
        draw_char(x1 as i32, y2 as i32, b'+');
    }
    if zr_i64_fits_i32(x2) && zr_i64_fits_i32(y2) {
        draw_char(x2 as i32, y2 as i32, b'+');
    }
    for xx in (x1 + 1)..x2 {
        if zr_i64_fits_i32(xx) && zr_i64_fits_i32(y1) {
            draw_char(xx as i32, y1 as i32, b'-');
        }
        if zr_i64_fits_i32(xx) && zr_i64_fits_i32(y2) {
            draw_char(xx as i32, y2 as i32, b'-');
        }
    }
    for yy in (y1 + 1)..y2 {
        if zr_i64_fits_i32(x1) && zr_i64_fits_i32(yy) {
            draw_char(x1 as i32, yy as i32, b'|');
        }
        if zr_i64_fits_i32(x2) && zr_i64_fits_i32(yy) {
            draw_char(x2 as i32, yy as i32, b'|');
        }
    }
    ZR_OK
}

pub fn zr_fb_draw_scrollbar_v(
    p: &mut ZrFbPainter,
    track: ZrRect,
    thumb: ZrRect,
    track_style: &ZrStyle,
    thumb_style: &ZrStyle,
) -> ZrResult {
    zr_fb_fill_rect(p, track, track_style)?;
    let ch = b'#';
    for y in thumb.y..thumb.y + thumb.h {
        for x in thumb.x..thumb.x + thumb.w {
            if x < 0 || y < 0 {
                continue;
            }
            let _ = zr_fb_put_grapheme(p, x, y, ch, 1, thumb_style);
        }
    }
    ZR_OK
}

pub fn zr_fb_draw_scrollbar_h(
    p: &mut ZrFbPainter,
    track: ZrRect,
    thumb: ZrRect,
    track_style: &ZrStyle,
    thumb_style: &ZrStyle,
) -> ZrResult {
    zr_fb_draw_scrollbar_v(p, track, thumb, track_style, thumb_style)
}

pub fn zr_fb_put_grapheme(
    p: &mut ZrFbPainter,
    x: i32,
    y: i32,
    bytes: &[u8],
    width: u8,
    style: &ZrStyle,
) -> ZrResult {
    if !zr_fb_has_backing(p.fb) {
        return ZR_OK;
    }
    if width != 1 && width != 2 {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    let mut out_bytes = bytes;
    let mut out_len = bytes.len();
    let mut try_wide = width == 2;

    // Canonicalize empty graphemes to space
    if out_len == 0 {
        out_bytes = b" ";
        out_len = 1;
        try_wide = false;
    }
    if out_len > ZR_CELL_GLYPH_MAX {
        out_bytes = &ZR_UTF8_REPLACEMENT;
        out_len = ZR_UTF8_REPLACEMENT_LEN;
        try_wide = false;
    }
    if !zr_fb_utf8_grapheme_bytes_safe_for_terminal(&out_bytes[..out_len]) {
        out_bytes = &ZR_UTF8_REPLACEMENT;
        out_len = ZR_UTF8_REPLACEMENT_LEN;
        try_wide = false;
    }
    if x < 0 || y < 0 {
        return ZR_OK;
    }
    let ux = x as u32;
    let uy = y as u32;
    if ux >= p.fb.cols || uy >= p.fb.rows {
        return ZR_OK;
    }
    if try_wide {
        if zr_painter_write_width2(p, ux, uy, &out_bytes[..out_len], *style) {
            return ZR_OK;
        }
        // Fallback to replacement
        out_bytes = &ZR_UTF8_REPLACEMENT;
        out_len = ZR_UTF8_REPLACEMENT_LEN;
    }
    let _ = zr_painter_write_width1(p, ux, uy, &out_bytes[..out_len], *style);
    ZR_OK
}

fn zr_rects_overlap(a: ZrRect, b: ZrRect) -> bool {
    let i = zr_rect_intersect(a, b);
    i.w > 0 && i.h > 0
}

pub fn zr_fb_blit_rect(p: &mut ZrFbPainter, dst: ZrRect, src: ZrRect) -> ZrResult {
    if dst.w <= 0 || dst.h <= 0 || src.w <= 0 || src.h <= 0 {
        return ZR_OK;
    }
    if !zr_fb_has_backing(p.fb) {
        return ZR_OK;
    }
    let w = cmp::min(dst.w, src.w);
    let h = cmp::min(dst.h, src.h);
    if w <= 0 || h <= 0 {
        return ZR_OK;
    }
    let mut dst_eff = dst;
    dst_eff.w = w;
    dst_eff.h = h;
    let mut src_eff = src;
    src_eff.w = w;
    src_eff.h = h;
    let overlap = zr_rects_overlap(dst_eff, src_eff);
    let mut y0 = 0;
    let mut y1 = h;
    let mut ystep = 1;
    let mut x0 = 0;
    let mut x1 = w;
    let mut xstep = 1;
    if overlap {
        if dst_eff.y > src_eff.y {
            y0 = h - 1;
            y1 = -1;
            ystep = -1;
        } else if dst_eff.y == src_eff.y && dst_eff.x > src_eff.x {
            x0 = w - 1;
            x1 = -1;
            xstep = -1;
        }
    }
    let clip = zr_painter_clip_cur(p);
    let mut oy = y0;
    while oy != y1 {
        let sy = src_eff.y + oy;
        let dy = dst_eff.y + oy;
        let mut ox = x0;
        while ox != x1 {
            let sx = src_eff.x + ox;
            let dx = dst_eff.x + ox;
            if !zr_rect_contains(clip, dx, dy) {
                ox += xstep;
                continue;
            }
            if sx < 0 || sy < 0 || dx < 0 || dy < 0 {
                ox += xstep;
                continue;
            }
            let usx = sx as u32;
            let usy = sy as u32;
            if usx >= p.fb.cols || usy >= p.fb.rows {
                ox += xstep;
                continue;
            }
            let Some(cell) = zr_fb_cell_const(p.fb, usx, usy) else {
                ox += xstep;
                continue;
            };
            if zr_cell_is_continuation(cell) {
                ox += xstep;
                continue;
            }
            // Wide lead that doesn't fully fit: replace with U+FFFD
            if cell.width == 2 && (ox + 1) >= w {
                let _ = zr_fb_put_grapheme(
                    p,
                    dx,
                    dy,
                    &ZR_UTF8_REPLACEMENT,
                    1,
                    &cell.style,
                );
                ox += xstep;
                continue;
            }
            let _ = zr_fb_put_grapheme(
                p,
                dx,
                dy,
                &cell.glyph[..cell.glyph_len as usize],
                cell.width,
                &cell.style,
            );
            ox += xstep;
        }
        oy += ystep;
    }
    ZR_OK
}

pub fn zr_fb_count_cells_utf8(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let mut total = 0;
    let mut iter = ZrGraphemeIter::new(bytes);
    while let Some(g) = iter.next() {
        let w = zr_width_grapheme_utf8(&bytes[g.offset..][..g.size], zr_width_policy_default());
        total += w as usize;
    }
    total
}

pub fn zr_fb_draw_text_bytes(
    p: &mut ZrFbPainter,
    x: i32,
    y: i32,
    bytes: &[u8],
    style: &ZrStyle,
) -> ZrResult {
    if !zr_fb_has_backing(p.fb) || bytes.is_empty() {
        return ZR_OK;
    }
    if y < 0 || (y as u32) >= p.fb.rows {
        return ZR_OK;
    }
    let mut cx = x as i64;
    let mut iter = ZrGraphemeIter::new(bytes);
    while let Some(g) = iter.next() {
        let gb = &bytes[g.offset..][..g.size];
        let w = zr_width_grapheme_utf8(gb, zr_width_policy_default());
        if w == 0 {
            continue;
        }
        let mut out_bytes = gb;
        let mut out_len = gb.len();
        let mut out_w = w;
        let out_adv = w;
        if out_len > ZR_CELL_GLYPH_MAX {
            out_bytes = &ZR_UTF8_REPLACEMENT;
            out_len = ZR_UTF8_REPLACEMENT_LEN;
            out_w = 1;
        }
        if out_w == 2 {
            let cx1 = cx + 1;
            if zr_i64_fits_i32(cx) && zr_i64_fits_i32(cx1) {
                let ix = cx as i32;
                let lead_touch = zr_painter_can_touch(p, ix, y);
                if !lead_touch {
                    out_w = 0;
                } else if !zr_painter_can_touch(p, ix + 1, y) {
                    out_bytes = &ZR_UTF8_REPLACEMENT;
                    out_len = ZR_UTF8_REPLACEMENT_LEN;
                    out_w = 1;
                }
            } else {
                out_w = 0;
            }
        }
        if out_w != 0 && zr_i64_fits_i32(cx) {
            let _ = zr_fb_put_grapheme(p, cx as i32, y, &out_bytes[..out_len], out_w, style);
        }
        cx = cx.checked_add(out_adv as i64).ok_or(ZR_ERR_LIMIT)?;
    }
    ZR_OK
}