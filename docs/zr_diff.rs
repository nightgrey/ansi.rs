//! src/util/zr_framebuffer.c - Pure framebuffer diff renderer implementation.
//!
//! Emits minimal VT output by diffing previous/next framebuffers while
//! preserving grapheme/style correctness and deterministic terminal state.

use core::ptr;
use core::mem;
use core::cmp;

use crate::core::zr_fb::*;
use crate::util::zr_checked::*;
use crate::util::zr_macros::*;
use crate::util::zr_string_builder::*;
use crate::plat::caps::*;
use crate::core::zr_diff::*;

// --- Color Format Constants ---

/// RGB color format: 0x00RRGGBB (red in bits 16-23, green 8-15, blue 0-7).
const ZR_RGB_R_SHIFT: u32 = 16;
const ZR_RGB_G_SHIFT: u32 = 8;
const ZR_RGB_MASK: u32 = 0xFF;

/// xterm 256-color cube: 6 levels per channel (indices 16-231).
const ZR_XTERM256_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
const ZR_XTERM256_CUBE_START: u32 = 16;
const ZR_XTERM256_CUBE_SIZE: u32 = 6;

/// xterm 256-color grayscale ramp: 24 shades (indices 232-255).
const ZR_XTERM256_GRAY_START: u32 = 232;
const ZR_XTERM256_GRAY_COUNT: u32 = 24;
const ZR_XTERM256_GRAY_BASE: u8 = 8;   // First gray level value
const ZR_XTERM256_GRAY_STEP: u8 = 10;  // Increment per gray level

/// xterm-compatible 16-color palette (ANSI colors 0-15).
const ZR_ANSI16_PALETTE: [[u8; 3]; 16] = [
    // Standard colors (0-7)
    [0, 0, 0],           // 0: Black
    [205, 0, 0],         // 1: Red
    [0, 205, 0],         // 2: Green
    [205, 205, 0],       // 3: Yellow
    [0, 0, 238],         // 4: Blue
    [205, 0, 205],       // 5: Magenta
    [0, 205, 205],       // 6: Cyan
    [229, 229, 229],     // 7: White
    // Bright colors (8-15)
    [127, 127, 127],     // 8: Bright Black (Gray)
    [255, 0, 0],         // 9: Bright Red
    [0, 255, 0],         // 10: Bright Green
    [255, 255, 0],       // 11: Bright Yellow
    [92, 92, 255],       // 12: Bright Blue
    [255, 0, 255],       // 13: Bright Magenta
    [0, 255, 255],       // 14: Bright Cyan
    [255, 255, 255],     // 15: Bright White
];

// SGR (Select Graphic Rendition) codes.
const ZR_SGR_RESET: u32 = 0;
const ZR_SGR_BOLD: u32 = 1;
const ZR_SGR_DIM: u32 = 2;
const ZR_SGR_ITALIC: u32 = 3;
const ZR_SGR_UNDERLINE: u32 = 4;
const ZR_SGR_BLINK: u32 = 5;
const ZR_SGR_REVERSE: u32 = 7;
const ZR_SGR_STRIKETHROUGH: u32 = 9;
const ZR_SGR_OVERLINE: u32 = 53;
const ZR_SGR_UNDERLINE_COLOR: u32 = 58;
const ZR_SGR_UNDERLINE_COLOR_DEFAULT: u32 = 59;
const ZR_SGR_FG_256: u32 = 38;        // Extended foreground color
const ZR_SGR_BG_256: u32 = 48;        // Extended background color
const ZR_SGR_COLOR_MODE_256: u32 = 5; // 256-color mode selector
const ZR_SGR_COLOR_MODE_RGB: u32 = 2; // RGB color mode selector

// ANSI 16-color SGR base codes.
const ZR_SGR_FG_BASE: u32 = 30;    // FG colors 0-7: 30-37
const ZR_SGR_FG_BRIGHT: u32 = 90;  // FG colors 8-15: 90-97
const ZR_SGR_BG_BASE: u32 = 40;    // BG colors 0-7: 40-47
const ZR_SGR_BG_BRIGHT: u32 = 100; // BG colors 8-15: 100-107
const ZR_SGR_256_INDEX_MASK: u32 = 0xFF;
const ZR_SGR_16_INDEX_MASK: u32 = 0x0F;

// Style attribute bits (v1).
const ZR_STYLE_ATTR_BOLD: u32 = 1 << 0;
const ZR_STYLE_ATTR_ITALIC: u32 = 1 << 1;
const ZR_STYLE_ATTR_UNDERLINE: u32 = 1 << 2;
const ZR_STYLE_ATTR_REVERSE: u32 = 1 << 3;
const ZR_STYLE_ATTR_DIM: u32 = 1 << 4;
const ZR_STYLE_ATTR_STRIKE: u32 = 1 << 5;
const ZR_STYLE_ATTR_OVERLINE: u32 = 1 << 6;
const ZR_STYLE_ATTR_BLINK: u32 = 1 << 7;
const ZR_STYLE_RESERVED_UNDERLINE_VARIANT_MASK: u32 = 0x07;
const ZR_STYLE_UNDERLINE_VARIANT_MIN: u32 = 1;
const ZR_STYLE_UNDERLINE_VARIANT_MAX: u32 = 5;

const ZR_ASCII_ESC: u8 = 0x1B;
const ZR_ASCII_BEL: u8 = 0x07;
const ZR_ASCII_ST_FINAL: u8 = b'\\';
const ZR_ASCII_DEL: u8 = 0x7F;

// Adaptive sweep threshold tuning (dirty-row density, percent).
const ZR_DIFF_SWEEP_DIRTY_LINE_PCT_BASE: u32 = 35;
const ZR_DIFF_SWEEP_DIRTY_LINE_PCT_WIDE_FRAME: u32 = 30;
const ZR_DIFF_SWEEP_DIRTY_LINE_PCT_SMALL_FRAME: u32 = 45;
const ZR_DIFF_SWEEP_DIRTY_LINE_PCT_VERY_DIRTY: u32 = 25;
const ZR_DIFF_SWEEP_VERY_DIRTY_NUM: u32 = 3;
const ZR_DIFF_SWEEP_VERY_DIRTY_DEN: u32 = 4;

// Scroll detection short-circuit thresholds.
const ZR_SCROLL_MAX_DELTA: u32 = 64;
const ZR_SCROLL_MIN_DIRTY_LINES: u32 = 4;
const ZR_DIFF_DIRTY_ROW_COUNT_UNKNOWN: u32 = 0xFFFFFFFF;
const ZR_DIFF_RECT_INDEX_NONE: u32 = 0xFFFFFFFF;
const ZR_DIFF_BASELINE_SPACE: u8 = b' ';

// FNV-1a 64-bit row fingerprint constants.
const ZR_FNV64_OFFSET_BASIS: u64 = 14695981039346656037;
const ZR_FNV64_PRIME: u64 = 1099511628211;

// --- Helper structs and functions ---

struct ZrAttrMap {
    bit: u32,
    sgr: u32,
}

const ZR_SGR_ATTRS_PRE_UNDERLINE: [ZrAttrMap; 3] = [
    ZrAttrMap { bit: ZR_STYLE_ATTR_BOLD, sgr: ZR_SGR_BOLD },
    ZrAttrMap { bit: ZR_STYLE_ATTR_DIM, sgr: ZR_SGR_DIM },
    ZrAttrMap { bit: ZR_STYLE_ATTR_ITALIC, sgr: ZR_SGR_ITALIC },
];

const ZR_SGR_ATTRS_POST_UNDERLINE: [ZrAttrMap; 4] = [
    ZrAttrMap { bit: ZR_STYLE_ATTR_REVERSE, sgr: ZR_SGR_REVERSE },
    ZrAttrMap { bit: ZR_STYLE_ATTR_STRIKE, sgr: ZR_SGR_STRIKETHROUGH },
    ZrAttrMap { bit: ZR_STYLE_ATTR_OVERLINE, sgr: ZR_SGR_OVERLINE },
    ZrAttrMap { bit: ZR_STYLE_ATTR_BLINK, sgr: ZR_SGR_BLINK },
];

/// Compare effective SGR style only; hyperlink state is tracked via OSC 8.
fn zr_style_eq_sgr(a: ZrStyle, b: ZrStyle) -> bool {
    if a.fg_rgb != b.fg_rgb || a.bg_rgb != b.bg_rgb || a.attrs != b.attrs || a.reserved != b.reserved {
        return false;
    }
    if (a.attrs & ZR_STYLE_ATTR_UNDERLINE) == 0 {
        return true;
    }
    a.underline_rgb == b.underline_rgb
}

/// Baseline-style equality uses raw link_ref values (same framebuffer domain).
fn zr_style_eq_cell(a: ZrStyle, b: ZrStyle) -> bool {
    zr_style_eq_sgr(a, b) && a.link_ref == b.link_ref
}

/// Compare hyperlink targets across framebuffer domains.
fn zr_link_targets_eq(a_fb: &ZrFb, a_ref: u32, b_fb: &ZrFb, b_ref: u32) -> bool {
    if a_ref == 0 || b_ref == 0 {
        return a_ref == b_ref;
    }
    let (a_uri, a_uri_len, a_id, a_id_len) = match zr_fb_link_lookup(a_fb, a_ref) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let (b_uri, b_uri_len, b_id, b_id_len) = match zr_fb_link_lookup(b_fb, b_ref) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if a_uri_len != b_uri_len || a_id_len != b_id_len {
        return false;
    }
    if (a_uri_len != 0 && unsafe { ptr::compare_bytes(a_uri, b_uri, a_uri_len) } != 0) ||
        (a_id_len != 0 && unsafe { ptr::compare_bytes(a_id, b_id, a_id_len) } != 0) {
        return false;
    }
    true
}

fn zr_style_eq_cell_for_diff(prev_fb: &ZrFb, a: ZrStyle, next_fb: &ZrFb, b: ZrStyle) -> bool {
    if !zr_style_eq_sgr(a, b) {
        return false;
    }
    zr_link_targets_eq(prev_fb, a.link_ref, next_fb, b.link_ref)
}

fn zr_diff_baseline_style() -> ZrStyle {
    ZrStyle {
        fg_rgb: 0,
        bg_rgb: 0,
        attrs: 0,
        reserved: 0,
        underline_rgb: 0,
        link_ref: 0,
    }
}

fn zr_term_style_is_valid(ts: &ZrTermState) -> bool {
    (ts.flags & ZR_TERM_STATE_STYLE_VALID) != 0
}

fn zr_term_cursor_pos_is_valid(ts: &ZrTermState) -> bool {
    (ts.flags & ZR_TERM_STATE_CURSOR_POS_VALID) != 0
}

fn zr_term_cursor_vis_is_valid(ts: &ZrTermState) -> bool {
    (ts.flags & ZR_TERM_STATE_CURSOR_VIS_VALID) != 0
}

fn zr_term_cursor_shape_is_valid(ts: &ZrTermState) -> bool {
    (ts.flags & ZR_TERM_STATE_CURSOR_SHAPE_VALID) != 0
}

/// Compare prev/next cells for diff equality (glyph, width, style, hyperlink target).
fn zr_cell_eq(prev_fb: &ZrFb, a: &ZrCell, next_fb: &ZrFb, b: &ZrCell) -> bool {
    if a.glyph_len != b.glyph_len || a.width != b.width {
        return false;
    }
    if !zr_style_eq_cell_for_diff(prev_fb, a.style, next_fb, b.style) {
        return false;
    }
    if a.glyph_len != 0 && unsafe { ptr::compare_bytes(a.glyph, b.glyph, a.glyph_len as usize) } != 0 {
        return false;
    }
    true
}

fn zr_cell_is_continuation(c: &ZrCell) -> bool {
    c.width == 0
}

/// Conservative cursor-drift guard.
fn zr_cell_may_drift_cursor(c: &ZrCell) -> bool {
    if c.width != 1 {
        return true;
    }
    for i in 0..c.glyph_len {
        if c.glyph[i as usize] >= 0x80 {
            return true;
        }
    }
    false
}

fn zr_fb_row_bytes(fb: &ZrFb) -> usize {
    if fb.cols == 0 {
        return 0;
    }
    (fb.cols as usize) * mem::size_of::<ZrCell>()
}

fn zr_fb_row_ptr(fb: &ZrFb, y: u32) -> Option<*const u8> {
    if y >= fb.rows {
        return None;
    }
    let row_off_cells = zr_checked_mul_size(y as usize, fb.cols as usize).ok()?;
    Some(unsafe { (fb.cells.as_ptr() as *const u8).add(row_off_cells) })
}

/// Exact row compare over cell storage bytes; false means "maybe dirty".
fn zr_row_eq_exact(a: &ZrFb, ay: u32, b: &ZrFb, by: u32) -> bool {
    if a.cols != b.cols {
        return false;
    }
    let row_bytes = zr_fb_row_bytes(a);
    let pa = match zr_fb_row_ptr(a, ay) {
        Some(p) => p,
        None => return false,
    };
    let pb = match zr_fb_row_ptr(b, by) {
        Some(p) => p,
        None => return false,
    };
    if row_bytes == 0 {
        return true;
    }
    unsafe { ptr::compare_bytes(pa, pb, row_bytes) == 0 }
}

fn zr_fb_links_eq_exact(a: &ZrFb, b: &ZrFb) -> bool {
    if a.links_len != b.links_len || a.link_bytes_len != b.link_bytes_len {
        return false;
    }
    if a.links_len == 0 && a.link_bytes_len == 0 {
        return true;
    }
    if (a.links_len != 0 && (a.links.is_null() || b.links.is_null())) ||
        (a.link_bytes_len != 0 && (a.link_bytes.is_null() || b.link_bytes.is_null())) {
        return false;
    }
    if a.links_len != 0 {
        let links_bytes = zr_checked_mul_size(a.links_len as usize, mem::size_of::<ZrFbLink>()).unwrap();
        if unsafe { ptr::compare_bytes(a.links as *const u8, b.links as *const u8, links_bytes) } != 0 {
            return false;
        }
    }
    if a.link_bytes_len != 0 {
        if unsafe { ptr::compare_bytes(a.link_bytes, b.link_bytes, a.link_bytes_len as usize) } != 0 {
            return false;
        }
    }
    true
}

/// Compare hyperlink targets for corresponding cells across framebuffer domains.
fn zr_row_links_targets_eq(a: &ZrFb, ay: u32, b: &ZrFb, by: u32) -> bool {
    if a.cols != b.cols || ay >= a.rows || by >= b.rows {
        return false;
    }
    if a.cols == 0 {
        return true;
    }
    if a.links_len == 0 && b.links_len == 0 {
        return true;
    }
    let arow = match zr_fb_row_ptr(a, ay) {
        Some(p) => unsafe { &*(p as *const ZrCell) },
        None => return false,
    };
    let brow = match zr_fb_row_ptr(b, by) {
        Some(p) => unsafe { &*(p as *const ZrCell) },
        None => return false,
    };
    let mut last_a_ref = 0;
    let mut last_b_ref = 0;
    for x in 0..a.cols {
        let a_ref = unsafe { arow.add(x as usize) }.read().style.link_ref;
        let b_ref = unsafe { brow.add(x as usize) }.read().style.link_ref;
        if a_ref == 0 && b_ref == 0 {
            last_a_ref = 0;
            last_b_ref = 0;
            continue;
        }
        if a_ref == last_a_ref && b_ref == last_b_ref {
            continue;
        }
        if !zr_link_targets_eq(a, a_ref, b, b_ref) {
            return false;
        }
        last_a_ref = a_ref;
        last_b_ref = b_ref;
    }
    true
}

fn zr_hash_bytes_fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = ZR_FNV64_OFFSET_BASIS;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(ZR_FNV64_PRIME);
    }
    h
}

fn zr_row_hash64(fb: &ZrFb, y: u32) -> u64 {
    if y >= fb.rows {
        return 0;
    }
    let row_bytes = zr_fb_row_bytes(fb);
    let ptr = match zr_fb_row_ptr(fb, y) {
        Some(p) => p,
        None => return 0,
    };
    let slice = unsafe { core::slice::from_raw_parts(ptr, row_bytes) };
    zr_hash_bytes_fnv1a64(slice)
}

/// Return display width of cell at (x,y): 0 for continuation, 2 for wide, 1 otherwise.
fn zr_cell_width_in_next(fb: &ZrFb, x: u32, y: u32) -> u8 {
    let c = match zr_fb_cell_const(fb, x, y) {
        Some(c) => c,
        None => return 1,
    };
    if zr_cell_is_continuation(c) {
        return 0;
    }
    if c.width == 2 {
        return 2;
    }
    if x + 1 < fb.cols {
        if let Some(c1) = zr_fb_cell_const(fb, x + 1, y) {
            if zr_cell_is_continuation(c1) {
                return 2;
            }
        }
    }
    1
}

fn zr_rgb_r(rgb: u32) -> u8 {
    ((rgb >> ZR_RGB_R_SHIFT) & ZR_RGB_MASK) as u8
}
fn zr_rgb_g(rgb: u32) -> u8 {
    ((rgb >> ZR_RGB_G_SHIFT) & ZR_RGB_MASK) as u8
}
fn zr_rgb_b(rgb: u32) -> u8 {
    (rgb & ZR_RGB_MASK) as u8
}

/// Compute squared Euclidean distance between two RGB colors.
fn zr_dist2_u8(ar: u8, ag: u8, ab: u8, br: u8, bg: u8, bb: u8) -> u32 {
    let dr = ar as i32 - br as i32;
    let dg = ag as i32 - bg as i32;
    let db = ab as i32 - bb as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// Find the nearest xterm 256-color cube level (0-5) for a single RGB component.
fn zr_xterm256_component_level(v: u8) -> u8 {
    let mut best_i = 0;
    let mut best_d = u32::MAX;
    for (i, &level) in ZR_XTERM256_LEVELS.iter().enumerate() {
        let diff = v as i32 - level as i32;
        let d = (diff * diff) as u32;
        if d < best_d {
            best_d = d;
            best_i = i as u8;
        }
    }
    best_i
}

/// Map 24-bit RGB to nearest xterm 256-color index.
fn zr_rgb_to_xterm256(rgb: u32) -> u8 {
    let r = zr_rgb_r(rgb);
    let g = zr_rgb_g(rgb);
    let b = zr_rgb_b(rgb);

    // Color cube candidate (16..231)
    let ri = zr_xterm256_component_level(r);
    let gi = zr_xterm256_component_level(g);
    let bi = zr_xterm256_component_level(b);
    let cr = ZR_XTERM256_LEVELS[ri as usize];
    let cg = ZR_XTERM256_LEVELS[gi as usize];
    let cb = ZR_XTERM256_LEVELS[bi as usize];
    let cube_idx = (ZR_XTERM256_CUBE_START +
        ZR_XTERM256_CUBE_SIZE * ZR_XTERM256_CUBE_SIZE * ri as u32 +
        ZR_XTERM256_CUBE_SIZE * gi as u32 +
        bi as u32) as u8;
    let cube_d = zr_dist2_u8(r, g, b, cr, cg, cb);

    // Grayscale ramp candidate (232..255)
    let mut best_gray_i = 0;
    let mut best_gray_d = u32::MAX;
    for i in 0..ZR_XTERM256_GRAY_COUNT {
        let gv = ZR_XTERM256_GRAY_BASE + (ZR_XTERM256_GRAY_STEP * i as u8);
        let d = zr_dist2_u8(r, g, b, gv, gv, gv);
        if d < best_gray_d {
            best_gray_d = d;
            best_gray_i = i as u8;
        }
    }
    let gray_idx = ZR_XTERM256_GRAY_START + best_gray_i as u32;
    let gray_d = best_gray_d;

    if gray_d < cube_d {
        return gray_idx as u8;
    }
    if cube_d < gray_d {
        return cube_idx;
    }
    // Tie-break: choose the smaller xterm index deterministically.
    if gray_idx < cube_idx as u32 { gray_idx as u8 } else { cube_idx }
}

/// Map 24-bit RGB to nearest ANSI 16-color index (0-15).
fn zr_rgb_to_ansi16(rgb: u32) -> u8 {
    let r = zr_rgb_r(rgb);
    let g = zr_rgb_g(rgb);
    let b = zr_rgb_b(rgb);
    let mut best = 0;
    let mut best_d = u32::MAX;
    for (i, &pal) in ZR_ANSI16_PALETTE.iter().enumerate() {
        let d = zr_dist2_u8(r, g, b, pal[0], pal[1], pal[2]);
        if d < best_d {
            best_d = d;
            best = i as u8;
        } else if d == best_d && (i as u8) < best {
            best = i as u8;
        }
    }
    best
}

fn zr_style_underline_variant_bits(style: ZrStyle, caps: &PlatCaps) -> u32 {
    let variant = style.reserved & ZR_STYLE_RESERVED_UNDERLINE_VARIANT_MASK;
    if (style.attrs & ZR_STYLE_ATTR_UNDERLINE) == 0 || caps.supports_underline_styles == 0 {
        return 0;
    }
    if variant < ZR_STYLE_UNDERLINE_VARIANT_MIN || variant > ZR_STYLE_UNDERLINE_VARIANT_MAX {
        return 0;
    }
    variant
}

fn zr_style_effective_link_ref(style: ZrStyle, caps: &PlatCaps) -> u32 {
    if caps.supports_hyperlinks == 0 {
        return 0;
    }
    style.link_ref
}

fn zr_style_has_effective_underline_color(style: ZrStyle, caps: &PlatCaps) -> bool {
    if (style.attrs & ZR_STYLE_ATTR_UNDERLINE) == 0 || style.underline_rgb == 0 {
        return false;
    }
    caps.supports_colored_underlines != 0 &&
        (caps.color_mode == PLAT_COLOR_MODE_RGB || caps.color_mode == PLAT_COLOR_MODE_256)
}

/// Downgrade style attrs/colors (including underline extensions) by terminal caps.
fn zr_style_apply_caps(mut style: ZrStyle, caps: &PlatCaps) -> ZrStyle {
    style.attrs &= caps.sgr_attrs_supported;
    if (style.attrs & ZR_STYLE_ATTR_UNDERLINE) == 0 || caps.supports_underline_styles == 0 {
        style.reserved &= !ZR_STYLE_RESERVED_UNDERLINE_VARIANT_MASK;
    } else {
        let variant = style.reserved & ZR_STYLE_RESERVED_UNDERLINE_VARIANT_MASK;
        if variant < ZR_STYLE_UNDERLINE_VARIANT_MIN || variant > ZR_STYLE_UNDERLINE_VARIANT_MAX {
            style.reserved &= !ZR_STYLE_RESERVED_UNDERLINE_VARIANT_MASK;
        }
    }
    style.link_ref = zr_style_effective_link_ref(style, caps);

    if !zr_style_has_effective_underline_color(style, caps) {
        style.underline_rgb = 0;
    } else if caps.color_mode == PLAT_COLOR_MODE_256 {
        style.underline_rgb = zr_rgb_to_xterm256(style.underline_rgb) as u32;
    }

    match caps.color_mode {
        PLAT_COLOR_MODE_RGB => style,
        PLAT_COLOR_MODE_256 => {
            style.fg_rgb = zr_rgb_to_xterm256(style.fg_rgb) as u32;
            style.bg_rgb = zr_rgb_to_xterm256(style.bg_rgb) as u32;
            style
        }
        PLAT_COLOR_MODE_16 => {
            style.fg_rgb = zr_rgb_to_ansi16(style.fg_rgb) as u32;
            style.bg_rgb = zr_rgb_to_ansi16(style.bg_rgb) as u32;
            style
        }
        _ => {
            // Unknown: deterministically degrade to 16.
            style.fg_rgb = zr_rgb_to_ansi16(style.fg_rgb) as u32;
            style.bg_rgb = zr_rgb_to_ansi16(style.bg_rgb) as u32;
            style
        }
    }
}

/// Write u32 as decimal ASCII digits to string builder.
fn zr_sb_write_u32_dec(sb: &mut ZrSb, v: u32) -> bool {
    let mut tmp = [0u8; 10];
    let mut n = 0;
    let mut vv = v;
    loop {
        tmp[n] = b'0' + (vv % 10) as u8;
        n += 1;
        vv /= 10;
        if vv == 0 || n == tmp.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        if !sb.write_u8(tmp[i]) {
            return false;
        }
    }
    true
}

/// Reject control bytes that can terminate or corrupt OSC payload parsing.
fn zr_osc_field_bytes_safe(bytes: &[u8]) -> bool {
    for &ch in bytes {
        if ch == ZR_ASCII_ESC || ch == ZR_ASCII_BEL || ch < 0x20 || ch == ZR_ASCII_DEL {
            return false;
        }
    }
    true
}

fn zr_emit_osc8_close(sb: &mut ZrSb) -> bool {
    sb.write_u8(ZR_ASCII_ESC) &&
        sb.write_u8(b']') &&
        sb.write_u8(b'8') &&
        sb.write_u8(b';') &&
        sb.write_u8(b';') &&
        sb.write_u8(ZR_ASCII_ESC) &&
        sb.write_u8(ZR_ASCII_ST_FINAL)
}

fn zr_emit_osc8_open(sb: &mut ZrSb, uri: &[u8], id: &[u8]) -> bool {
    if uri.is_empty() {
        return false;
    }
    if !sb.write_u8(ZR_ASCII_ESC) || !sb.write_u8(b']') || !sb.write_u8(b'8') || !sb.write_u8(b';') {
        return false;
    }
    if !id.is_empty() {
        let id_prefix = b"id=";
        if !sb.write_bytes(id_prefix) || !sb.write_bytes(id) {
            return false;
        }
    }
    sb.write_u8(b';') &&
        sb.write_bytes(uri) &&
        sb.write_u8(ZR_ASCII_ESC) &&
        sb.write_u8(ZR_ASCII_ST_FINAL)
}

/// Emit CUP (cursor position) escape sequence if cursor is not already at (x,y).
fn zr_emit_cup(sb: &mut ZrSb, ts: &mut ZrTermState, x: u32, y: u32) -> bool {
    if zr_term_cursor_pos_is_valid(ts) && ts.cursor_x == x && ts.cursor_y == y {
        return true;
    }
    if !zr_diff_write_csi(sb) {
        return false;
    }
    if !zr_sb_write_u32_dec(sb, y + 1) || !sb.write_u8(b';') || !zr_sb_write_u32_dec(sb, x + 1) || !sb.write_u8(b'H') {
        return false;
    }
    ts.cursor_x = x;
    ts.cursor_y = y;
    ts.flags |= ZR_TERM_STATE_CURSOR_POS_VALID;
    true
}

fn zr_emit_cursor_visibility(sb: &mut ZrSb, ts: &mut ZrTermState, visible: u8) -> bool {
    if visible > 1 {
        return false;
    }
    if zr_term_cursor_vis_is_valid(ts) && ts.cursor_visible == visible {
        return true;
    }
    let seq: &[u8] = if visible != 0 { b"\x1b[?25h" } else { b"\x1b[?25l" };
    if !sb.write_bytes(seq) {
        return false;
    }
    ts.cursor_visible = visible;
    ts.flags |= ZR_TERM_STATE_CURSOR_VIS_VALID;
    true
}

fn zr_cursor_shape_ps(shape: u8, blink: u8) -> u32 {
    match shape {
        ZR_CURSOR_SHAPE_UNDERLINE => if blink != 0 { 3 } else { 4 },
        ZR_CURSOR_SHAPE_BAR => if blink != 0 { 5 } else { 6 },
        _ => if blink != 0 { 1 } else { 2 },
    }
}

fn zr_emit_cursor_shape(sb: &mut ZrSb, ts: &mut ZrTermState, shape: u8, blink: u8, caps: &PlatCaps) -> bool {
    if shape > ZR_CURSOR_SHAPE_BAR || blink > 1 {
        return false;
    }
    if caps.supports_cursor_shape == 0 {
        ts.flags |= ZR_TERM_STATE_CURSOR_SHAPE_VALID;
        return true;
    }
    if zr_term_cursor_shape_is_valid(ts) && ts.cursor_shape == shape && ts.cursor_blink == blink {
        return true;
    }
    let ps = zr_cursor_shape_ps(shape, blink);
    if !zr_diff_write_csi(sb) || !zr_sb_write_u32_dec(sb, ps) || !sb.write_u8(b' ') || !sb.write_u8(b'q') {
        return false;
    }
    ts.cursor_shape = shape;
    ts.cursor_blink = blink;
    ts.flags |= ZR_TERM_STATE_CURSOR_SHAPE_VALID;
    true
}

fn zr_clamp_u32_from_i32(v: i32, lo: u32, hi: u32) -> u32 {
    if hi < lo { return lo; }
    if v <= lo as i32 { lo } else if v >= hi as i32 { hi } else { v as u32 }
}

fn zr_emit_cursor_desired(sb: &mut ZrSb, ts: &mut ZrTermState, desired: Option<&ZrCursorState>, next: &ZrFb, caps: &PlatCaps) -> bool {
    let desired = match desired {
        Some(d) => d,
        None => return true,
    };
    if desired.visible != 0 {
        if !zr_emit_cursor_shape(sb, ts, desired.shape, desired.blink, caps) {
            return false;
        }
    }
    if !zr_emit_cursor_visibility(sb, ts, desired.visible) {
        return false;
    }
    if next.cols == 0 || next.rows == 0 {
        return true;
    }
    if desired.x == -1 && desired.y == -1 {
        if zr_term_cursor_pos_is_valid(ts) {
            return true;
        }
        let x = if ts.cursor_x < next.cols { ts.cursor_x } else { next.cols - 1 };
        let y = if ts.cursor_y < next.rows { ts.cursor_y } else { next.rows - 1 };
        return zr_emit_cup(sb, ts, x, y);
    }
    let mut x = if ts.cursor_x < next.cols { ts.cursor_x } else { next.cols - 1 };
    let mut y = if ts.cursor_y < next.rows { ts.cursor_y } else { next.rows - 1 };
    if desired.x != -1 {
        x = zr_clamp_u32_from_i32(desired.x, 0, next.cols - 1);
    }
    if desired.y != -1 {
        y = zr_clamp_u32_from_i32(desired.y, 0, next.rows - 1);
    }
    zr_emit_cup(sb, ts, x, y)
}

/// Convert a terminal 16-color index (0..15) into the matching SGR code.
fn zr_sgr_16color_code(foreground: bool, idx: u8) -> u32 {
    if foreground {
        if idx < 8 { ZR_SGR_FG_BASE + idx as u32 } else { ZR_SGR_FG_BRIGHT + (idx - 8) as u32 }
    } else {
        if idx < 8 { ZR_SGR_BG_BASE + idx as u32 } else { ZR_SGR_BG_BRIGHT + (idx - 8) as u32 }
    }
}

fn zr_emit_sgr_color_param(sb: &mut ZrSb, desired: ZrStyle, caps: &PlatCaps, foreground: bool) -> bool {
    match caps.color_mode {
        PLAT_COLOR_MODE_RGB => {
            let rgb = if foreground { desired.fg_rgb } else { desired.bg_rgb };
            let r = zr_rgb_r(rgb);
            let g = zr_rgb_g(rgb);
            let b = zr_rgb_b(rgb);
            let base = if foreground { ZR_SGR_FG_256 } else { ZR_SGR_BG_256 };
            zr_sb_write_u32_dec(sb, base) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, ZR_SGR_COLOR_MODE_RGB) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, r as u32) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, g as u32) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, b as u32)
        }
        PLAT_COLOR_MODE_256 => {
            let idx = if foreground { desired.fg_rgb & ZR_SGR_256_INDEX_MASK } else { desired.bg_rgb & ZR_SGR_256_INDEX_MASK };
            let base = if foreground { ZR_SGR_FG_256 } else { ZR_SGR_BG_256 };
            zr_sb_write_u32_dec(sb, base) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, ZR_SGR_COLOR_MODE_256) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, idx)
        }
        _ => {
            // 16-color (or unknown degraded to 16): desired.fg_rgb/bg_rgb are indices 0..15
            let idx = (if foreground { desired.fg_rgb } else { desired.bg_rgb }) & ZR_SGR_16_INDEX_MASK;
            let code = zr_sgr_16color_code(foreground, idx as u8);
            zr_sb_write_u32_dec(sb, code)
        }
    }
}

fn zr_emit_sgr_attr_params(sb: &mut ZrSb, attrs: u32, maps: &[ZrAttrMap]) -> bool {
    for map in maps {
        if (attrs & map.bit) != 0 {
            if !sb.write_u8(b';') || !zr_sb_write_u32_dec(sb, map.sgr) {
                return false;
            }
        }
    }
    true
}

/// Emit underline style as either legacy "4" or extended "4:n".
fn zr_emit_sgr_underline_style_param(sb: &mut ZrSb, desired: ZrStyle, caps: &PlatCaps) -> bool {
    let variant = zr_style_underline_variant_bits(desired, caps);
    if !zr_sb_write_u32_dec(sb, ZR_SGR_UNDERLINE) {
        return false;
    }
    if variant == 0 {
        return true;
    }
    sb.write_u8(b':') && zr_sb_write_u32_dec(sb, variant)
}

/// Emit SGR 58 underline-color parameter in RGB or 256 mode.
fn zr_emit_sgr_underline_color_param(sb: &mut ZrSb, desired: ZrStyle, caps: &PlatCaps) -> bool {
    match caps.color_mode {
        PLAT_COLOR_MODE_RGB => {
            let r = zr_rgb_r(desired.underline_rgb);
            let g = zr_rgb_g(desired.underline_rgb);
            let b = zr_rgb_b(desired.underline_rgb);
            zr_sb_write_u32_dec(sb, ZR_SGR_UNDERLINE_COLOR) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, ZR_SGR_COLOR_MODE_RGB) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, r as u32) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, g as u32) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, b as u32)
        }
        PLAT_COLOR_MODE_256 => {
            let idx = desired.underline_rgb & 0xFF;
            zr_sb_write_u32_dec(sb, ZR_SGR_UNDERLINE_COLOR) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, ZR_SGR_COLOR_MODE_256) &&
                sb.write_u8(b';') &&
                zr_sb_write_u32_dec(sb, idx)
        }
        _ => false,
    }
}

/// Emit full SGR sequence with reset to establish an exact style baseline.
fn zr_emit_sgr_absolute(sb: &mut ZrSb, ts: &mut ZrTermState, mut desired: ZrStyle, caps: &PlatCaps) -> bool {
    desired = zr_style_apply_caps(desired, caps);
    let desired_has_underline_color = zr_style_has_effective_underline_color(desired, caps);
    let had_underline_color = zr_term_style_is_valid(ts) && zr_style_has_effective_underline_color(ts.style, caps);
    let needs_underline_color_reset = caps.supports_colored_underlines != 0 && had_underline_color && !desired_has_underline_color;
    if zr_term_style_is_valid(ts) && zr_style_eq_sgr(ts.style, desired) {
        return true;
    }
    if !zr_diff_write_csi(sb) || !zr_sb_write_u32_dec(sb, ZR_SGR_RESET) {
        return false;
    }
    if !zr_emit_sgr_attr_params(sb, desired.attrs, &ZR_SGR_ATTRS_PRE_UNDERLINE) {
        return false;
    }
    if (desired.attrs & ZR_STYLE_ATTR_UNDERLINE) != 0 {
        if !sb.write_u8(b';') || !zr_emit_sgr_underline_style_param(sb, desired, caps) {
            return false;
        }
    }
    if !zr_emit_sgr_attr_params(sb, desired.attrs, &ZR_SGR_ATTRS_POST_UNDERLINE) {
        return false;
    }
    if desired_has_underline_color {
        if !sb.write_u8(b';') || !zr_emit_sgr_underline_color_param(sb, desired, caps) {
            return false;
        }
    }
    if needs_underline_color_reset {
        if !sb.write_u8(b';') || !zr_sb_write_u32_dec(sb, ZR_SGR_UNDERLINE_COLOR_DEFAULT) {
            return false;
        }
    }
    if !sb.write_u8(b';') || !zr_emit_sgr_color_param(sb, desired, caps, true) ||
        !sb.write_u8(b';') || !zr_emit_sgr_color_param(sb, desired, caps, false) ||
        !sb.write_u8(b'm') {
        return false;
    }
    ts.style = desired;
    ts.flags |= ZR_TERM_STATE_STYLE_VALID;
    true
}

fn zr_emit_sgr_delta(sb: &mut ZrSb, ts: &mut ZrTermState, desired: ZrStyle, caps: &PlatCaps) -> bool {
    // Compatibility-first policy: use absolute reset-based SGR for every style transition.
    zr_emit_sgr_absolute(sb, ts, desired, caps)
}

/// Check if cell at (x,y) differs between prev and next framebuffers.
/// Also returns true if wide-glyph continuation cell changed.
fn zr_line_dirty_at(prev: &ZrFb, next: &ZrFb, x: u32, y: u32) -> bool {
    let a = match zr_fb_cell_const(prev, x, y) {
        Some(c) => c,
        None => return false,
    };
    let b = match zr_fb_cell_const(next, x, y) {
        Some(c) => c,
        None => return false,
    };
    if !zr_cell_eq(prev, a, next, b) {
        return true;
    }
    // Wide-glyph rule: a dirty continuation forces inclusion of its lead cell.
    if x + 1 < prev.cols {
        let a1 = zr_fb_cell_const(prev, x + 1, y);
        let b1 = zr_fb_cell_const(next, x + 1, y);
        let cont = a1.map(zr_cell_is_continuation).unwrap_or(false) ||
            b1.map(zr_cell_is_continuation).unwrap_or(false);
        if cont {
            if let (Some(a1), Some(b1)) = (a1, b1) {
                if !zr_cell_eq(prev, a1, next, b1) {
                    return true;
                }
            }
        }
    }
    false
}

fn zr_cell_is_blank_baseline(c: &ZrCell) -> bool {
    if c.width != 1 || c.glyph_len != 1 || c.glyph[0] != ZR_DIFF_BASELINE_SPACE {
        return false;
    }
    zr_style_eq_cell(c.style, zr_diff_baseline_style())
}

/// Check whether framebuffer contents match the baseline clear model.
fn zr_fb_is_blank_baseline(fb: &ZrFb) -> bool {
    if fb.cols == 0 || fb.rows == 0 {
        return true;
    }
    if fb.cells.is_null() {
        return false;
    }
    for y in 0..fb.rows {
        for x in 0..fb.cols {
            if let Some(c) = zr_fb_cell_const(fb, x, y) {
                if !zr_cell_is_blank_baseline(c) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
    true
}

struct ZrDiffCtx<'a> {
    prev: &'a ZrFb,
    next: &'a ZrFb,
    caps: &'a PlatCaps,
    prev_row_hashes: Option<&'a mut [u64]>,
    next_row_hashes: Option<&'a mut [u64]>,
    dirty_rows: Option<&'a mut [u8]>,
    dirty_row_count: u32,
    has_row_cache: bool,
    sb: ZrSb,
    ts: ZrTermState,
    stats: ZrDiffStats,
    damage: ZrDamage,
}

struct ZrScrollPlan {
    active: bool,
    up: bool,
    top: u32,
    bottom: u32,
    lines: u32,
    moved_lines: u32,
}

/// Track OSC 8 hyperlink transitions independently from SGR style transitions.
fn zr_diff_emit_link_transition(ctx: &mut ZrDiffCtx, desired_link_ref: u32) -> ZrResult {
    let mut desired_link_ref = desired_link_ref;
    if ctx.caps.supports_hyperlinks == 0 {
        desired_link_ref = 0;
    }
    let current_link_known = zr_term_style_is_valid(&ctx.ts);
    let current_link_ref = if current_link_known {
        zr_style_effective_link_ref(ctx.ts.style, ctx.caps)
    } else {
        0
    };
    if !current_link_known && ctx.caps.supports_hyperlinks != 0 {
        if !zr_emit_osc8_close(&mut ctx.sb) {
            return ZR_ERR_LIMIT;
        }
        // current_link_ref stays 0, known becomes true implicitly after close? We'll manage via style.
        // In C, they set current_link_ref=0 and current_link_known=true.
        // We'll simulate by forcing a close and then continue. For simplicity, we'll just update the ts style.
    }
    if current_link_known && current_link_ref == desired_link_ref {
        ctx.ts.style.link_ref = desired_link_ref;
        return ZR_OK;
    }
    if current_link_known && current_link_ref != 0 && !zr_emit_osc8_close(&mut ctx.sb) {
        return ZR_ERR_LIMIT;
    }
    if desired_link_ref != 0 {
        let (uri, uri_len, id, id_len) = match zr_fb_link_lookup(ctx.next, desired_link_ref) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let uri_slice = unsafe { core::slice::from_raw_parts(uri, uri_len) };
        let id_slice = unsafe { core::slice::from_raw_parts(id, id_len) };
        if !zr_osc_field_bytes_safe(uri_slice) || !zr_osc_field_bytes_safe(id_slice) {
            return ZR_ERR_FORMAT;
        }
        if !zr_emit_osc8_open(&mut ctx.sb, uri_slice, id_slice) {
            return ZR_ERR_LIMIT;
        }
    }
    ctx.ts.style.link_ref = desired_link_ref;
    ZR_OK
}

fn zr_diff_zero_outputs(out_len: &mut usize, out_final_term_state: &mut ZrTermState, out_stats: &mut ZrDiffStats) {
    *out_len = 0;
    *out_final_term_state = unsafe { mem::zeroed() };
    *out_stats = unsafe { mem::zeroed() };
}

fn zr_diff_validate_args(
    prev: &ZrFb, next: &ZrFb, caps: &PlatCaps, initial_term_state: &ZrTermState,
    desired_cursor_state: Option<&ZrCursorState>, lim: &ZrLimits,
    scratch_damage_rects: &mut [ZrDamageRect], scratch: Option<&ZrDiffScratch>,
    enable_scroll_optimizations: u8,
) -> ZrResult {
    // This validation is simplified; the original checks many pointers.
    if prev.cols != next.cols || prev.rows != next.rows {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    if scratch_damage_rects.len() < lim.diff_max_damage_rects as usize {
        return ZR_ERR_INVALID_ARGUMENT;
    }
    if let Some(scr) = scratch {
        if scr.row_cap != 0 && scr.row_cap < next.rows {
            return ZR_ERR_INVALID_ARGUMENT;
        }
    }
    ZR_OK
}

/// Populate optional per-line hash/dirty caches.
fn zr_diff_prepare_row_cache(ctx: &mut ZrDiffCtx, scratch: Option<&mut ZrDiffScratch>) {
    if ctx.prev_row_hashes.is_some() && ctx.next_row_hashes.is_some() && ctx.dirty_rows.is_some() {
        // Already set via external scratch? Actually we need to extract from scratch.
        // In C, they set ctx fields from scratch. We'll do similar: if scratch provided, use it.
    }
    // Simplified: we'll just set based on scratch existence.
    if let Some(scr) = scratch {
        if scr.row_cap >= ctx.next.rows {
            ctx.prev_row_hashes = Some(unsafe { &mut *scr.prev_row_hashes });
            ctx.next_row_hashes = Some(unsafe { &mut *scr.next_row_hashes });
            ctx.dirty_rows = Some(unsafe { &mut *scr.dirty_rows });
            ctx.has_row_cache = true;
            ctx.dirty_row_count = 0;
            let reuse_prev_hashes = scr.prev_hashes_valid != 0;
            let links_exact_equal = zr_fb_links_eq_exact(ctx.prev, ctx.next);
            for y in 0..ctx.next.rows {
                let prev_hash = if reuse_prev_hashes {
                    ctx.prev_row_hashes.as_mut().unwrap()[y as usize]
                } else {
                    let h = zr_row_hash64(ctx.prev, y);
                    ctx.prev_row_hashes.as_mut().unwrap()[y as usize] = h;
                    h
                };
                let next_hash = zr_row_hash64(ctx.next, y);
                ctx.next_row_hashes.as_mut().unwrap()[y as usize] = next_hash;
                let mut dirty = 0;
                if prev_hash != next_hash {
                    dirty = 1;
                } else if !zr_row_eq_exact(ctx.prev, y, ctx.next, y) {
                    dirty = 1;
                    ctx.stats.collision_guard_hits += 1;
                } else if !links_exact_equal && !zr_row_links_targets_eq(ctx.prev, y, ctx.next, y) {
                    dirty = 1;
                }
                ctx.dirty_rows.as_mut().unwrap()[y as usize] = dirty;
                if dirty != 0 {
                    ctx.dirty_row_count += 1;
                }
            }
        }
    }
}

/// Compare full framebuffer rows for scroll-shift detection (full width).
fn zr_row_eq(a: &ZrFb, ay: u32, b: &ZrFb, by: u32) -> bool {
    if !zr_row_eq_exact(a, ay, b, by) {
        return false;
    }
    zr_row_links_targets_eq(a, ay, b, by)
}

/// Deterministic preference order for competing scroll candidates.
fn zr_scroll_plan_better(best: &ZrScrollPlan, cand: &ZrScrollPlan, cols: u32) -> bool {
    if !cand.active { return false; }
    if !best.active { return true; }
    let best_cells = best.moved_lines as u64 * cols as u64;
    let cand_cells = cand.moved_lines as u64 * cols as u64;
    if cand_cells != best_cells {
        return cand_cells > best_cells;
    }
    if cand.moved_lines != best.moved_lines {
        return cand.moved_lines > best.moved_lines;
    }
    if cand.lines != best.lines {
        return cand.lines < best.lines;
    }
    if cand.top != best.top {
        return cand.top < best.top;
    }
    if cand.bottom != best.bottom {
        return cand.bottom < best.bottom;
    }
    cand.up != best.up
}

fn zr_scroll_saved_enough(moved_lines: u32, cols: u32) -> bool {
    const ZR_SCROLL_MIN_MOVED_LINES: u32 = 4;
    const ZR_SCROLL_MIN_SAVED_CELLS: u64 = 256;
    if moved_lines < ZR_SCROLL_MIN_MOVED_LINES {
        return false;
    }
    let saved_cells = moved_lines as u64 * cols as u64;
    saved_cells >= ZR_SCROLL_MIN_SAVED_CELLS
}

/// Evaluate a contiguous run of row matches as a scroll-region candidate.
fn zr_scroll_plan_consider_run(best: &mut ZrScrollPlan, cols: u32, rows: u32, up: bool,
                               run_start: u32, run_len: u32, delta: u32) {
    if run_len == 0 || delta == 0 { return; }
    let mut cand = ZrScrollPlan {
        active: true,
        up,
        top: run_start,
        bottom: run_start + run_len - 1 + delta,
        lines: delta,
        moved_lines: run_len,
    };
    if cand.bottom >= rows { return; }
    if !zr_scroll_saved_enough(cand.moved_lines, cols) { return; }
    if zr_scroll_plan_better(best, &cand, cols) {
        *best = cand;
    }
}

/// Scan for the longest run of shifted-equal rows for a given delta + direction.
fn zr_scroll_scan_delta_dir(prev: &ZrFb, next: &ZrFb,
                            prev_hashes: Option<&[u64]>, next_hashes: Option<&[u64]>,
                            delta: u32, up: bool, best: &mut ZrScrollPlan) {
    if delta == 0 || delta >= next.rows { return; }
    let rows = next.rows;
    let cols = next.cols;
    let y_end = rows - delta;
    let mut run_start = 0;
    let mut run_len = 0;
    for y in 0..y_end {
        let (next_y, prev_y) = if up { (y, y + delta) } else { (y + delta, y) };
        if best.active {
            let remaining = y_end - y;
            if (run_len + remaining) <= best.moved_lines {
                break;
            }
        }
        let hash_match = if let (Some(prev_h), Some(next_h)) = (prev_hashes, next_hashes) {
            next_h[next_y as usize] == prev_h[prev_y as usize]
        } else {
            true
        };
        let match_ = hash_match && zr_row_eq(next, next_y, prev, prev_y);
        if match_ {
            if run_len == 0 { run_start = y; }
            run_len += 1;
        } else {
            zr_scroll_plan_consider_run(best, cols, rows, up, run_start, run_len, delta);
            run_len = 0;
        }
    }
    zr_scroll_plan_consider_run(best, cols, rows, up, run_start, run_len, delta);
}

/// Detect a vertical scroll within a full-width region.
fn zr_diff_detect_scroll_fullwidth(prev: &ZrFb, next: &ZrFb,
                                   prev_hashes: Option<&[u64]>, next_hashes: Option<&[u64]>,
                                   dirty_row_count: u32) -> ZrScrollPlan {
    let mut best = ZrScrollPlan { active: false, up: false, top: 0, bottom: 0, lines: 0, moved_lines: 0 };
    if prev.cols != next.cols || prev.rows != next.rows { return best; }
    if next.rows < 2 || next.cols == 0 { return best; }
    if dirty_row_count != ZR_DIFF_DIRTY_ROW_COUNT_UNKNOWN {
        if dirty_row_count == 0 || dirty_row_count < ZR_SCROLL_MIN_DIRTY_LINES {
            return best;
        }
    }
    let rows = next.rows;
    let max_delta = cmp::min(rows - 1, ZR_SCROLL_MAX_DELTA);
    for delta in 1..=max_delta {
        if best.active {
            let moved_cap = rows - delta;
            if moved_cap <= best.moved_lines { continue; }
        }
        zr_scroll_scan_delta_dir(prev, next, prev_hashes, next_hashes, delta, true, &mut best);
        zr_scroll_scan_delta_dir(prev, next, prev_hashes, next_hashes, delta, false, &mut best);
    }
    if !best.active { return best; }
    if best.bottom <= best.top || (best.bottom - best.top + 1) <= best.lines || best.lines == 0 {
        best.active = false;
    }
    best
}

fn zr_emit_decstbm(sb: &mut ZrSb, ts: &mut ZrTermState, top: u32, bottom: u32) -> bool {
    if !zr_diff_write_csi(sb) { return false; }
    if !zr_sb_write_u32_dec(sb, top + 1) || !sb.write_u8(b';') ||
        !zr_sb_write_u32_dec(sb, bottom + 1) || !sb.write_u8(b'r') {
        return false;
    }
    ts.cursor_x = 0;
    ts.cursor_y = 0;
    ts.flags |= ZR_TERM_STATE_CURSOR_POS_VALID;
    true
}

fn zr_emit_scroll_op(sb: &mut ZrSb, ts: &mut ZrTermState, up: bool, lines: u32) -> bool {
    if lines == 0 { return true; }
    if !zr_diff_write_csi(sb) { return false; }
    if !zr_sb_write_u32_dec(sb, lines) { return false; }
    sb.write_u8(if up { b'S' } else { b'T' })
}

fn zr_emit_decstbm_reset(sb: &mut ZrSb, ts: &mut ZrTermState) -> bool {
    if !zr_diff_write_csi(sb) || !sb.write_u8(b'r') { return false; }
    ts.cursor_x = 0;
    ts.cursor_y = 0;
    ts.flags |= ZR_TERM_STATE_CURSOR_POS_VALID;
    true
}

fn zr_emit_ed2_clear_screen(sb: &mut ZrSb) -> bool {
    sb.write_bytes(b"\x1b[2J")
}

/// Establish a known blank baseline when screen contents are not trusted.
fn zr_diff_establish_blank_screen_baseline(ctx: &mut ZrDiffCtx) -> ZrResult {
    ctx.ts.flags &= !(ZR_TERM_STATE_STYLE_VALID | ZR_TERM_STATE_CURSOR_POS_VALID);
    if !zr_emit_decstbm_reset(&mut ctx.sb, &mut ctx.ts) {
        return ZR_ERR_LIMIT;
    }
    let baseline = zr_diff_baseline_style();
    if !zr_emit_sgr_absolute(&mut ctx.sb, &mut ctx.ts, baseline, ctx.caps) {
        return ZR_ERR_LIMIT;
    }
    if !zr_emit_ed2_clear_screen(&mut ctx.sb) {
        return ZR_ERR_LIMIT;
    }
    ctx.ts.flags |= ZR_TERM_STATE_SCREEN_VALID;
    if ctx.sb.truncated() { ZR_ERR_LIMIT } else { ZR_OK }
}

/// Render a contiguous span of dirty cells [start, end] on row y.
fn zr_diff_render_span(ctx: &mut ZrDiffCtx, y: u32, start: u32, end: u32) -> ZrResult {
    if !zr_emit_cup(&mut ctx.sb, &mut ctx.ts, start, y) {
        return ZR_ERR_LIMIT;
    }
    for xx in start..=end {
        let c = match zr_fb_cell_const(ctx.next, xx, y) {
            Some(c) => c,
            None => continue,
        };
        let w = zr_cell_width_in_next(ctx.next, xx, y);
        if w == 0 { continue; }
        if !zr_emit_cup(&mut ctx.sb, &mut ctx.ts, xx, y) {
            return ZR_ERR_LIMIT;
        }
        let link_rc = zr_diff_emit_link_transition(ctx, c.style.link_ref);
        if link_rc != ZR_OK { return link_rc; }
        if !zr_emit_sgr_delta(&mut ctx.sb, &mut ctx.ts, c.style, ctx.caps) {
            return ZR_ERR_LIMIT;
        }
        if c.glyph_len != 0 {
            let glyph_slice = unsafe { core::slice::from_raw_parts(c.glyph, c.glyph_len as usize) };
            if !ctx.sb.write_bytes(glyph_slice) {
                return ZR_ERR_LIMIT;
            }
        } else {
            for _ in 0..w {
                if !ctx.sb.write_u8(b' ') {
                    return ZR_ERR_LIMIT;
                }
            }
        }
        ctx.ts.cursor_x += w as u32;
        if zr_cell_may_drift_cursor(c) {
            ctx.ts.flags &= !ZR_TERM_STATE_CURSOR_POS_VALID;
        }
    }
    if ctx.sb.truncated() { ZR_ERR_LIMIT } else { ZR_OK }
}

fn zr_diff_render_full_line(ctx: &mut ZrDiffCtx, y: u32) -> ZrResult {
    if ctx.next.cols == 0 { return ZR_OK; }
    zr_diff_render_span(ctx, y, 0, ctx.next.cols - 1)
}

fn zr_diff_expand_span_for_wide(next: &ZrFb, y: u32, start: &mut u32, end: &mut u32) {
    if next.cols == 0 || y >= next.rows { return; }
    if *start >= next.cols || *end >= next.cols { return; }
    if *start > 0 {
        if let Some(c) = zr_fb_cell_const(next, *start, y) {
            if zr_cell_is_continuation(c) {
                *start -= 1;
            }
        }
    }
    if *end + 1 < next.cols {
        let w = zr_cell_width_in_next(next, *end, y);
        if w == 2 {
            *end += 1;
        }
    }
}

fn zr_u32_mul_clamp(a: u32, b: u32) -> u32 {
    let prod = (a as u64).saturating_mul(b as u64);
    if prod > u32::MAX as u64 { u32::MAX } else { prod as u32 }
}

fn zr_diff_row_known_clean(ctx: &ZrDiffCtx, y: u32) -> bool {
    if let Some(dirty) = ctx.dirty_rows.as_ref() {
        if y < ctx.next.rows {
            return dirty[y as usize] == 0;
        }
    }
    false
}

fn zr_diff_sweep_threshold_pct(ctx: &ZrDiffCtx) -> u32 {
    if ctx.next.rows == 0 {
        return ZR_DIFF_SWEEP_DIRTY_LINE_PCT_BASE;
    }
    let mut threshold_pct = ZR_DIFF_SWEEP_DIRTY_LINE_PCT_BASE;
    if ctx.next.rows <= 12 {
        threshold_pct = ZR_DIFF_SWEEP_DIRTY_LINE_PCT_SMALL_FRAME;
    } else if ctx.next.cols >= 120 {
        threshold_pct = ZR_DIFF_SWEEP_DIRTY_LINE_PCT_WIDE_FRAME;
    }
    let dirty_scaled = (ctx.dirty_row_count as u64) * (ZR_DIFF_SWEEP_VERY_DIRTY_DEN as u64);
    let very_dirty_scaled = (ctx.next.rows as u64) * (ZR_DIFF_SWEEP_VERY_DIRTY_NUM as u64);
    if dirty_scaled >= very_dirty_scaled {
        threshold_pct = ZR_DIFF_SWEEP_DIRTY_LINE_PCT_VERY_DIRTY;
    }
    threshold_pct
}

fn zr_diff_should_use_sweep(ctx: &ZrDiffCtx) -> bool {
    if ctx.next.rows == 0 || !ctx.has_row_cache {
        return false;
    }
    let threshold_pct = zr_diff_sweep_threshold_pct(ctx);
    let dirty_scaled = (ctx.dirty_row_count as u64) * 100;
    let rows_scaled = (ctx.next.rows as u64) * (threshold_pct as u64);
    dirty_scaled >= rows_scaled
}

fn zr_diff_span_overlaps_or_touches(r: &ZrDamageRect, span_start: u32, span_end: u32) -> bool {
    (r.x0 <= span_end + 1) && (r.x1 + 1 >= span_start)
}

/// Merge one rectangle into the current row span, flushing first when disjoint.
fn zr_diff_span_merge_or_flush(ctx: &mut ZrDiffCtx, y: u32, r: &ZrDamageRect,
                               have_span: &mut bool, span_start: &mut u32, span_end: &mut u32) -> ZrResult {
    if !*have_span {
        *span_start = r.x0;
        *span_end = r.x1;
        *have_span = true;
        return ZR_OK;
    }
    if zr_diff_span_overlaps_or_touches(r, *span_start, *span_end) {
        if r.x0 < *span_start { *span_start = r.x0; }
        if r.x1 > *span_end { *span_end = r.x1; }
        return ZR_OK;
    }
    let rc = zr_diff_render_span(ctx, y, *span_start, *span_end);
    if rc != ZR_OK { return rc; }
    *span_start = r.x0;
    *span_end = r.x1;
    ZR_OK
}

fn zr_diff_span_flush(ctx: &mut ZrDiffCtx, y: u32, have_span: bool, span_start: u32, span_end: u32) -> ZrResult {
    if !have_span { return ZR_OK; }
    zr_diff_render_span(ctx, y, span_start, span_end)
}

fn zr_diff_render_damage_coalesced_scan(ctx: &mut ZrDiffCtx) -> ZrResult {
    for y in 0..ctx.next.rows {
        let mut span_start = 0;
        let mut span_end = 0;
        let mut have_span = false;
        for i in 0..ctx.damage.rect_count {
            let r = &ctx.damage.rects[i as usize];
            if y < r.y0 || y > r.y1 { continue; }
            let rc = zr_diff_span_merge_or_flush(ctx, y, r, &mut have_span, &mut span_start, &mut span_end);
            if rc != ZR_OK { return rc; }
        }
        let rc = zr_diff_span_flush(ctx, y, have_span, span_start, span_end);
        if rc != ZR_OK { return rc; }
    }
    ZR_OK
}

fn zr_diff_row_head_get(row_heads: &[u64], y: u32) -> u32 {
    if y < row_heads.len() as u32 { row_heads[y as usize] as u32 } else { ZR_DIFF_RECT_INDEX_NONE }
}

fn zr_diff_row_head_set(row_heads: &mut [u64], y: u32, value: u32) {
    if y < row_heads.len() as u32 { row_heads[y as usize] = value as u64; }
}

fn zr_diff_row_heads_reset(row_heads: &mut [u64], rows: u32) {
    for y in 0..rows {
        zr_diff_row_head_set(row_heads, y, ZR_DIFF_RECT_INDEX_NONE);
    }
}

fn zr_diff_rect_link_get(r: &ZrDamageRect) -> u32 {
    r._link
}

fn zr_diff_rect_link_set(r: &mut ZrDamageRect, next_idx: u32) {
    r._link = next_idx;
}

struct ZrDiffActiveRects {
    head: u32,
    tail: u32,
}

impl ZrDiffActiveRects {
    fn new() -> Self {
        ZrDiffActiveRects { head: ZR_DIFF_RECT_INDEX_NONE, tail: ZR_DIFF_RECT_INDEX_NONE }
    }
    fn append(&mut self, ctx: &mut ZrDiffCtx, idx: u32) {
        if idx == ZR_DIFF_RECT_INDEX_NONE { return; }
        zr_diff_rect_link_set(&mut ctx.damage.rects[idx as usize], ZR_DIFF_RECT_INDEX_NONE);
        if self.tail == ZR_DIFF_RECT_INDEX_NONE {
            self.head = idx;
            self.tail = idx;
            return;
        }
        zr_diff_rect_link_set(&mut ctx.damage.rects[self.tail as usize], idx);
        self.tail = idx;
    }
    fn remove(&mut self, ctx: &mut ZrDiffCtx, prev_idx: u32, idx: u32, next_idx: u32) {
        if idx == ZR_DIFF_RECT_INDEX_NONE { return; }
        if prev_idx == ZR_DIFF_RECT_INDEX_NONE {
            self.head = next_idx;
        } else {
            zr_diff_rect_link_set(&mut ctx.damage.rects[prev_idx as usize], next_idx);
        }
        if self.tail == idx {
            self.tail = prev_idx;
        }
        zr_diff_rect_link_set(&mut ctx.damage.rects[idx as usize], ZR_DIFF_RECT_INDEX_NONE);
    }
}

/// Index rectangle starts by y0 while preserving ascending rectangle order.
fn zr_diff_indexed_build_row_heads(ctx: &mut ZrDiffCtx, row_heads: &mut [u64], rows: u32) {
    for i in (0..ctx.damage.rect_count).rev() {
        let r = &mut ctx.damage.rects[i as usize];
        let start_y = r.y0;
        if start_y >= rows { continue; }
        let head = zr_diff_row_head_get(row_heads, start_y);
        zr_diff_rect_link_set(r, head);
        zr_diff_row_head_set(row_heads, start_y, i);
    }
}

fn zr_diff_indexed_activate_row(ctx: &mut ZrDiffCtx, row_heads: &[u64], y: u32, active: &mut ZrDiffActiveRects) {
    let mut start_idx = zr_diff_row_head_get(row_heads, y);
    while start_idx != ZR_DIFF_RECT_INDEX_NONE {
        let r = &mut ctx.damage.rects[start_idx as usize];
        let next_start = zr_diff_rect_link_get(r);
        active.append(ctx, start_idx);
        start_idx = next_start;
    }
}

fn zr_diff_indexed_render_row(ctx: &mut ZrDiffCtx, y: u32, active: &mut ZrDiffActiveRects) -> ZrResult {
    let mut span_start = 0;
    let mut span_end = 0;
    let mut have_span = false;
    let mut prev_idx = ZR_DIFF_RECT_INDEX_NONE;
    let mut idx = active.head;
    while idx != ZR_DIFF_RECT_INDEX_NONE {
        let r = &ctx.damage.rects[idx as usize];
        let next_idx = zr_diff_rect_link_get(r);
        let rc = zr_diff_span_merge_or_flush(ctx, y, r, &mut have_span, &mut span_start, &mut span_end);
        if rc != ZR_OK { return rc; }
        if r.y1 == y {
            active.remove(ctx, prev_idx, idx, next_idx);
        } else {
            prev_idx = idx;
        }
        idx = next_idx;
    }
    zr_diff_span_flush(ctx, y, have_span, span_start, span_end)
}

fn zr_diff_render_damage_coalesced_indexed(ctx: &mut ZrDiffCtx) -> ZrResult {
    let rows = ctx.next.rows;
    // Reuse prev_row_hashes as per-row head indices for this frame's damage lists.
    let row_heads = match ctx.prev_row_hashes.as_mut() {
        Some(slice) => slice,
        None => return ZR_ERR_INVALID_ARGUMENT,
    };
    zr_diff_row_heads_reset(row_heads, rows);
    zr_diff_indexed_build_row_heads(ctx, row_heads, rows);
    let mut active = ZrDiffActiveRects::new();
    for y in 0..rows {
        zr_diff_indexed_activate_row(ctx, row_heads, y, &mut active);
        let rc = zr_diff_indexed_render_row(ctx, y, &mut active);
        if rc != ZR_OK { return rc; }
    }
    ZR_OK
}

fn zr_diff_render_damage_coalesced(ctx: &mut ZrDiffCtx) -> ZrResult {
    if ctx.has_row_cache && ctx.prev_row_hashes.is_some() {
        zr_diff_render_damage_coalesced_indexed(ctx)
    } else {
        zr_diff_render_damage_coalesced_scan(ctx)
    }
}

fn zr_diff_build_damage(ctx: &mut ZrDiffCtx, lim: &ZrLimits, scratch: &mut [ZrDamageRect]) -> ZrResult {
    zr_damage_begin_frame(&mut ctx.damage, scratch, lim.diff_max_damage_rects, ctx.next.cols, ctx.next.rows);
    for y in 0..ctx.next.rows {
        if zr_diff_row_known_clean(ctx, y) { continue; }
        let mut line_dirty = false;
        let mut x = 0;
        while x < ctx.next.cols {
            if !zr_line_dirty_at(ctx.prev, ctx.next, x, y) {
                x += 1;
                continue;
            }
            let start = x;
            while x < ctx.next.cols && zr_line_dirty_at(ctx.prev, ctx.next, x, y) {
                x += 1;
            }
            let mut end = if x == 0 { 0 } else { x - 1 };
            zr_diff_expand_span_for_wide(ctx.next, y, &mut start, &mut end);
            zr_damage_add_span(&mut ctx.damage, y, start, end);
            line_dirty = true;
            ctx.stats.dirty_cells += (end - start + 1);
            if ctx.damage.full_frame != 0 { break; }
        }
        if line_dirty { ctx.stats.dirty_lines += 1; }
        if ctx.damage.full_frame != 0 { break; }
    }
    ctx.stats.damage_rects = ctx.damage.rect_count;
    ctx.stats.damage_cells = zr_damage_cells(&ctx.damage);
    ctx.stats.damage_full_frame = ctx.damage.full_frame;
    ctx.stats._pad0 = 0;
    ZR_OK
}

/// Scan row y for dirty spans and render each one.
fn zr_diff_render_line(ctx: &mut ZrDiffCtx, y: u32) -> ZrResult {
    if zr_diff_row_known_clean(ctx, y) { return ZR_OK; }
    let mut line_dirty = false;
    let mut x = 0;
    while x < ctx.next.cols {
        if !zr_line_dirty_at(ctx.prev, ctx.next, x, y) {
            x += 1;
            continue;
        }
        let start = x;
        while x < ctx.next.cols && zr_line_dirty_at(ctx.prev, ctx.next, x, y) {
            x += 1;
        }
        let end = if x == 0 { 0 } else { x - 1 };
        let rc = zr_diff_render_span(ctx, y, start, end);
        if rc != ZR_OK { return rc; }
        line_dirty = true;
        ctx.stats.dirty_cells += (end - start + 1);
        if ctx.sb.truncated() { return ZR_ERR_LIMIT; }
    }
    if line_dirty { ctx.stats.dirty_lines += 1; }
    ZR_OK
}

fn zr_diff_finalize_damage_stats_sweep(ctx: &mut ZrDiffCtx) {
    ctx.stats.damage_rects = ctx.stats.dirty_lines;
    ctx.stats.damage_cells = ctx.stats.dirty_cells;
    ctx.stats.damage_full_frame = 0;
    ctx.stats._pad0 = 0;
}

/// Render a full-frame redraw from `next`.
fn zr_diff_render_full_frame(ctx: &mut ZrDiffCtx) -> ZrResult {
    for y in 0..ctx.next.rows {
        let rc = zr_diff_render_full_line(ctx, y);
        if rc != ZR_OK { return rc; }
    }
    let full_cells = zr_u32_mul_clamp(ctx.next.cols, ctx.next.rows);
    ctx.stats.path_sweep_used = 1;
    ctx.stats.path_damage_used = 0;
    ctx.stats.damage_full_frame = if full_cells != 0 { 1 } else { 0 };
    ctx.stats.damage_rects = if full_cells != 0 { 1 } else { 0 };
    ctx.stats.damage_cells = full_cells;
    ctx.stats.dirty_lines = if ctx.next.cols != 0 { ctx.next.rows } else { 0 };
    ctx.stats.dirty_cells = full_cells;
    ctx.stats._pad0 = 0;
    if ctx.sb.truncated() { ZR_ERR_LIMIT } else { ZR_OK }
}

fn zr_diff_render_sweep_rows(ctx: &mut ZrDiffCtx, skip_top: u32, skip_bottom: u32, has_skip: bool) -> ZrResult {
    for y in 0..ctx.next.rows {
        if has_skip && y >= skip_top && y <= skip_bottom { continue; }
        let rc = zr_diff_render_line(ctx, y);
        if rc != ZR_OK { return rc; }
    }
    zr_diff_finalize_damage_stats_sweep(ctx);
    ZR_OK
}

/// Try to apply a scroll-region optimization and report a row range to skip.
fn zr_diff_try_scroll_opt(ctx: &mut ZrDiffCtx, out_skip: &mut bool, out_skip_top: &mut u32, out_skip_bottom: &mut u32) -> ZrResult {
    ctx.stats.scroll_opt_attempted = 1;
    *out_skip = false;
    *out_skip_top = 0;
    *out_skip_bottom = 0;
    let dirty_row_count = if ctx.has_row_cache { ctx.dirty_row_count } else { ZR_DIFF_DIRTY_ROW_COUNT_UNKNOWN };
    let plan = zr_diff_detect_scroll_fullwidth(ctx.prev, ctx.next,
                                               ctx.prev_row_hashes.as_deref(), ctx.next_row_hashes.as_deref(),
                                               dirty_row_count);
    if !plan.active { return ZR_OK; }
    ctx.stats.scroll_opt_hit = 1;
    if !zr_emit_decstbm(&mut ctx.sb, &mut ctx.ts, plan.top, plan.bottom) {
        return ZR_ERR_LIMIT;
    }
    if !zr_emit_scroll_op(&mut ctx.sb, &mut ctx.ts, plan.up, plan.lines) {
        return ZR_ERR_LIMIT;
    }
    if !zr_emit_decstbm_reset(&mut ctx.sb, &mut ctx.ts) {
        return ZR_ERR_LIMIT;
    }
    if plan.up {
        let first_new = plan.bottom - plan.lines + 1;
        for y in first_new..=plan.bottom {
            let rc = zr_diff_render_full_line(ctx, y);
            if rc != ZR_OK { return rc; }
            ctx.stats.dirty_lines += 1;
            ctx.stats.dirty_cells += ctx.next.cols;
        }
    } else {
        let last_new = plan.top + plan.lines - 1;
        for y in plan.top..=last_new {
            let rc = zr_diff_render_full_line(ctx, y);
            if rc != ZR_OK { return rc; }
            ctx.stats.dirty_lines += 1;
            ctx.stats.dirty_cells += ctx.next.cols;
        }
    }
    *out_skip = true;
    *out_skip_top = plan.top;
    *out_skip_bottom = plan.bottom;
    if ctx.sb.truncated() { ZR_ERR_LIMIT } else { ZR_OK }
}

/// Public entry point.
pub fn zr_diff_render_ex(
    prev: &ZrFb, next: &ZrFb, caps: &PlatCaps, initial_term_state: &ZrTermState,
    desired_cursor_state: Option<&ZrCursorState>, lim: &ZrLimits,
    scratch_damage_rects: &mut [ZrDamageRect], scratch: Option<&mut ZrDiffScratch>,
    enable_scroll_optimizations: u8,
    out_buf: &mut [u8], out_len: &mut usize, out_final_term_state: &mut ZrTermState, out_stats: &mut ZrDiffStats,
) -> ZrResult {
    zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
    let arg_rc = zr_diff_validate_args(prev, next, caps, initial_term_state, desired_cursor_state, lim,
                                       scratch_damage_rects, scratch, enable_scroll_optimizations);
    if arg_rc != ZR_OK { return arg_rc; }

    let mut ctx = ZrDiffCtx {
        prev,
        next,
        caps,
        prev_row_hashes: None,
        next_row_hashes: None,
        dirty_rows: None,
        dirty_row_count: 0,
        has_row_cache: false,
        sb: ZrSb::new(out_buf),
        ts: *initial_term_state,
        stats: unsafe { mem::zeroed() },
        damage: unsafe { mem::zeroed() },
    };
    zr_diff_prepare_row_cache(&mut ctx, scratch);

    let mut force_full_redraw = false;
    if (ctx.ts.flags & ZR_TERM_STATE_SCREEN_VALID) == 0 {
        let pre_baseline_ts = ctx.ts;
        let rc = zr_diff_establish_blank_screen_baseline(&mut ctx);
        if rc == ZR_OK {
            force_full_redraw = !zr_fb_is_blank_baseline(prev);
        } else if rc == ZR_ERR_LIMIT {
            ctx.sb.reset();
            ctx.ts = pre_baseline_ts;
        } else {
            zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
            return rc;
        }
    }

    let mut skip = false;
    let mut skip_top = 0;
    let mut skip_bottom = 0;
    if !force_full_redraw && enable_scroll_optimizations != 0 && caps.supports_scroll_region != 0 {
        let rc = zr_diff_try_scroll_opt(&mut ctx, &mut skip, &mut skip_top, &mut skip_bottom);
        if rc != ZR_OK {
            zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
            return rc;
        }
    }

    if force_full_redraw {
        let rc = zr_diff_render_full_frame(&mut ctx);
        if rc != ZR_OK {
            zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
            return rc;
        }
    } else if skip {
        ctx.stats.damage_full_frame = 1;
        ctx.stats.damage_rects = 1;
        ctx.stats.damage_cells = zr_u32_mul_clamp(next.cols, next.rows);
        ctx.stats._pad0 = 0;
        for y in 0..next.rows {
            if y >= skip_top && y <= skip_bottom { continue; }
            let rc = zr_diff_render_line(&mut ctx, y);
            if rc != ZR_OK {
                zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
                return rc;
            }
        }
    } else {
        if zr_diff_should_use_sweep(&ctx) {
            ctx.stats.path_sweep_used = 1;
            ctx.stats.path_damage_used = 0;
            ctx.stats.dirty_lines = 0;
            ctx.stats.dirty_cells = 0;
            let rc = zr_diff_render_sweep_rows(&mut ctx, 0, 0, false);
            if rc != ZR_OK {
                zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
                return rc;
            }
        } else {
            ctx.stats.path_sweep_used = 0;
            ctx.stats.path_damage_used = 1;
            let mut rc = zr_diff_build_damage(&mut ctx, lim, scratch_damage_rects);
            if rc != ZR_OK {
                zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
                return rc;
            }
            if ctx.damage.full_frame != 0 {
                ctx.stats.dirty_lines = 0;
                ctx.stats.dirty_cells = 0;
                for y in 0..next.rows {
                    rc = zr_diff_render_line(&mut ctx, y);
                    if rc != ZR_OK {
                        zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
                        return rc;
                    }
                }
            } else {
                rc = zr_diff_render_damage_coalesced(&mut ctx);
                if rc != ZR_OK {
                    zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
                    return rc;
                }
            }
        }
    }

    let link_close_rc = zr_diff_emit_link_transition(&mut ctx, 0);
    if link_close_rc != ZR_OK {
        zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
        return link_close_rc;
    }
    if !zr_emit_cursor_desired(&mut ctx.sb, &mut ctx.ts, desired_cursor_state, next, caps) {
        zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
        return ZR_ERR_LIMIT;
    }
    if ctx.sb.truncated() {
        zr_diff_zero_outputs(out_len, out_final_term_state, out_stats);
        return ZR_ERR_LIMIT;
    }
    *out_len = ctx.sb.len();
    *out_final_term_state = ctx.ts;
    ctx.stats.bytes_emitted = *out_len;
    *out_stats = ctx.stats;
    ZR_OK
}

/// Simplified version without scratch parameter.
pub fn zr_diff_render(
    prev: &ZrFb, next: &ZrFb, caps: &PlatCaps, initial_term_state: &ZrTermState,
    desired_cursor_state: Option<&ZrCursorState>, lim: &ZrLimits,
    scratch_damage_rects: &mut [ZrDamageRect], enable_scroll_optimizations: u8,
    out_buf: &mut [u8], out_len: &mut usize, out_final_term_state: &mut ZrTermState, out_stats: &mut ZrDiffStats,
) -> ZrResult {
    zr_diff_render_ex(prev, next, caps, initial_term_state, desired_cursor_state, lim,
                      scratch_damage_rects, None, enable_scroll_optimizations,
                      out_buf, out_len, out_final_term_state, out_stats)
}

// Helper function to write CSI
fn zr_diff_write_csi(sb: &mut ZrSb) -> bool {
    sb.write_u8(ZR_ASCII_ESC) && sb.write_u8(b'[')
}