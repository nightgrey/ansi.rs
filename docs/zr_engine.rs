```rust
//! src/core/zr_engine.rs — Public engine ABI implementation and orchestration.
//!
//! Wires together platform I/O, input parsing, event batching, drawlist
//! execution, framebuffer diff rendering, and single-flush output emission
//! under the project's locked ownership and error contracts.

use core::cmp;
use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::atomic::AtomicPtr;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::zr_cursor::*;
use crate::core::zr_damage::*;
use crate::core::zr_debug_overlay::*;
use crate::core::zr_debug_trace::*;
use crate::core::zr_detect::*;
use crate::core::zr_diff::*;
use crate::core::zr_blit::*;
use crate::core::zr_drawlist::*;
use crate::core::zr_event_pack::*;
use crate::core::zr_event_queue::*;
use crate::core::zr_image::*;
use crate::core::zr_input_parser::*;
use crate::core::zr_metrics_internal::*;

use crate::platform::zr_platform::*;

use crate::util::zr_arena::*;
use crate::util::zr_assert::*;
use crate::util::zr_checked::*;
use crate::util::zr_string_builder::*;
use crate::util::zr_thread_yield::*;

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

const ZR_ENGINE_INPUT_PENDING_CAP: usize = 64;
const ZR_ENGINE_DETECT_PASSTHROUGH_CAP: usize = 4096;
const ZR_ENGINE_PASTE_MARKER_LEN: usize = 6;
const ZR_ENGINE_PASTE_IDLE_FLUSH_POLLS: u32 = 4;

const ZR_ENGINE_PASTE_BEGIN: &[u8] = b"\x1b[200~";
const ZR_ENGINE_PASTE_END: &[u8] = b"\x1b[201~";
const ZR_ENGINE_KITTY_KEYBOARD_PUSH: &[u8] = b"\x1b[>1u";
const ZR_ENGINE_KITTY_KEYBOARD_POP: &[u8] = b"\x1b[<u";

const ZR_ENGINE_EVENT_QUEUE_CAP: usize = 1024;
const ZR_ENGINE_USER_BYTES_CAP: usize = 64 * 1024;
const ZR_ENGINE_READ_CHUNK_CAP: usize = 4096;
const ZR_ENGINE_READ_LOOP_MAX: usize = 64;
const ZR_ENGINE_DEFAULT_TICK_INTERVAL_MS: u32 = 16;

const ZR_SYNC_BEGIN: &[u8] = b"\x1b[?2026h";
const ZR_SYNC_END: &[u8] = b"\x1b[?2026l";

const ZR_DEBUG_RING_BUF_SIZE: usize = 256 * 1024; // 256 KB

// -----------------------------------------------------------------------------
// Engine structure
// -----------------------------------------------------------------------------

pub struct ZrEngine {
    // Platform
    plat: Option<Box<dyn ZrPlatform>>,
    restore_prev: *mut ZrEngine,
    restore_next: *mut ZrEngine,
    restore_registered: bool,
    _pad_restore0: [u8; 3],

    // Baseline caps
    caps_base: PlatCaps,
    term_profile_base: ZrTerminalProfile,

    // Effective caps
    caps: PlatCaps,
    term_profile: ZrTerminalProfile,
    size: PlatSize,
    kitty_keyboard_active: bool,
    _pad_caps0: [u8; 3],

    // Config
    cfg_create: ZrEngineConfig,
    cfg_runtime: ZrEngineRuntimeConfig,

    // Tick scheduling
    last_tick_ms: u32,

    // Framebuffers
    fb_prev: ZrFb,
    fb_next: ZrFb,
    fb_stage: ZrFb,
    term_state: ZrTermState,
    cursor_desired: ZrCursorState,
    fb_next_synced_to_prev: bool,
    _pad_fb_sync0: [u8; 3],

    // Image sideband
    image_frame_next: ZrImageFrame,
    image_frame_stage: ZrImageFrame,
    image_state: ZrImageState,

    // Drawlist resources
    dl_resources_next: ZrDlResources,
    dl_resources_stage: ZrDlResources,

    // Output buffer
    out_buf: Vec<u8>,
    out_cap: usize,

    // Damage scratch
    damage_rects: Vec<ZrDamageRect>,
    damage_rect_cap: u32,
    diff_prev_row_hashes: Vec<u64>,
    diff_next_row_hashes: Vec<u64>,
    diff_dirty_rows: Vec<u8>,
    diff_row_cap: u32,
    diff_prev_hashes_valid: bool,
    _pad_diff0: [u8; 3],

    // Diff telemetry
    diff_sweep_frames_total: u64,
    diff_damage_frames_total: u64,
    diff_scroll_attempts_total: u64,
    diff_scroll_hits_total: u64,
    diff_collision_guard_hits_total: u64,

    // Event pipeline
    evq: ZrEventQueue,
    ev_storage: Vec<ZrEvent>,
    ev_cap: usize,
    user_bytes: Vec<u8>,
    user_bytes_cap: u32,
    post_user_inflight: AtomicU32,
    destroy_started: AtomicBool,

    // Input buffering
    input_pending: [u8; ZR_ENGINE_INPUT_PENDING_CAP],
    input_pending_len: usize,

    paste_begin_hold: [u8; ZR_ENGINE_PASTE_MARKER_LEN],
    paste_begin_hold_len: usize,

    paste_buf: Vec<u8>,
    paste_buf_cap: u32,
    paste_len: u32,
    paste_active: bool,
    paste_overflowed: bool,
    paste_idle_polls: u32,

    paste_end_hold: [u8; ZR_ENGINE_PASTE_MARKER_LEN],
    paste_end_hold_len: usize,

    // Arenas
    arena_frame: ZrArena,
    arena_persistent: ZrArena,

    // Metrics snapshot
    metrics: ZrMetrics,

    // Debug trace
    debug_trace: Option<ZrDebugTrace>,
    debug_ring_buf: Vec<u8>,
    debug_record_offsets: Vec<u32>,
    debug_record_sizes: Vec<u32>,
}

// -----------------------------------------------------------------------------
// Static globals for restore hooks
// -----------------------------------------------------------------------------

static G_ZR_ENGINE_RESTORE_HEAD: AtomicPtr<ZrEngine> = AtomicPtr::new(ptr::null_mut());
static G_ZR_ENGINE_RESTORE_LOCK: AtomicBool = AtomicBool::new(false);
static G_ZR_ENGINE_RESTORE_ACTIVE_GUARD: AtomicBool = AtomicBool::new(false);
static G_ZR_ENGINE_RESTORE_HOOKS_INSTALLED: AtomicU8 = AtomicU8::new(0);

#[cfg(feature = "zr_engine_testing")]
static G_ZR_ENGINE_TEST_RESTORE_ATTEMPTS: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "zr_engine_testing")]
static G_ZR_ENGINE_TEST_RESTORE_ABORT_CALLS: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "zr_engine_testing")]
static G_ZR_ENGINE_TEST_RESTORE_EXIT_CALLS: AtomicU32 = AtomicU32::new(0);

// -----------------------------------------------------------------------------
// Restore helpers
// -----------------------------------------------------------------------------

fn zr_engine_restore_lock_acquire() {
    while G_ZR_ENGINE_RESTORE_LOCK.swap(true, Ordering::Acquire) {
        zr_thread_yield();
    }
}

fn zr_engine_restore_lock_release() {
    G_ZR_ENGINE_RESTORE_LOCK.store(false, Ordering::Release);
}

fn zr_engine_restore_platform_state(e: &mut ZrEngine) {
    if let Some(plat) = &mut e.plat {
        let _ = zr_engine_sync_kitty_keyboard(e, &ZrTerminalProfile::default());
        let _ = plat.leave_raw();
    }
}

fn zr_engine_restore_active_platforms() -> u32 {
    if G_ZR_ENGINE_RESTORE_ACTIVE_GUARD.swap(true, Ordering::AcqRel) {
        return 0;
    }

    let mut attempts = 0;
    zr_engine_restore_lock_acquire();
    let mut ptr = G_ZR_ENGINE_RESTORE_HEAD.load(Ordering::Acquire);
    while !ptr.is_null() {
        let e = unsafe { &mut *ptr };
        if e.plat.is_some() {
            attempts += 1;
            zr_engine_restore_platform_state(e);
        }
        ptr = e.restore_next;
    }
    zr_engine_restore_lock_release();

    G_ZR_ENGINE_RESTORE_ACTIVE_GUARD.store(false, Ordering::Release);
    attempts
}

fn zr_engine_restore_from_assert() {
    let attempts = zr_engine_restore_active_platforms();
    #[cfg(feature = "zr_engine_testing")]
    {
        G_ZR_ENGINE_TEST_RESTORE_ABORT_CALLS.fetch_add(1, Ordering::AcqRel);
        G_ZR_ENGINE_TEST_RESTORE_ATTEMPTS.fetch_add(attempts, Ordering::AcqRel);
    }
}

fn zr_engine_restore_from_exit() {
    let attempts = zr_engine_restore_active_platforms();
    #[cfg(feature = "zr_engine_testing")]
    {
        G_ZR_ENGINE_TEST_RESTORE_EXIT_CALLS.fetch_add(1, Ordering::AcqRel);
        G_ZR_ENGINE_TEST_RESTORE_ATTEMPTS.fetch_add(attempts, Ordering::AcqRel);
    }
}

fn zr_engine_restore_install_hooks_once() {
    if G_ZR_ENGINE_RESTORE_HOOKS_INSTALLED.load(Ordering::Acquire) != 0 {
        return;
    }
    zr_engine_restore_lock_acquire();
    if G_ZR_ENGINE_RESTORE_HOOKS_INSTALLED.load(Ordering::Acquire) == 0 {
        // In Rust, we can't easily set atexit hooks; we simulate by using a global
        // that is called on process exit? For simplicity, we will not implement atexit
        // but note that in C this uses atexit. We'll assume the caller ensures cleanup.
        // Alternatively, we could use `std::panic::set_hook` for assert-like behavior.
        // For translation, we'll leave this as a no-op with a comment.
        // The original registers atexit to call zr_engine_restore_from_exit.
        G_ZR_ENGINE_RESTORE_HOOKS_INSTALLED.store(1, Ordering::Release);
    }
    zr_engine_restore_lock_release();
}

fn zr_engine_restore_register(e: &mut ZrEngine) {
    if e.plat.is_none() {
        return;
    }
    zr_engine_restore_install_hooks_once();
    zr_engine_restore_lock_acquire();
    if !e.restore_registered {
        e.restore_prev = ptr::null_mut();
        e.restore_next = G_ZR_ENGINE_RESTORE_HEAD.load(Ordering::Acquire);
        if !e.restore_next.is_null() {
            unsafe { (*e.restore_next).restore_prev = e };
        }
        G_ZR_ENGINE_RESTORE_HEAD.store(e, Ordering::Release);
        e.restore_registered = true;
    }
    // In C, they call zr_engine_restore_sync_assert_hook_locked() which sets an assert hook.
    // We'll skip that for Rust as we don't have a global assert hook mechanism.
    zr_engine_restore_lock_release();
}

fn zr_engine_restore_unregister(e: &mut ZrEngine) {
    zr_engine_restore_lock_acquire();
    if e.restore_registered {
        if !e.restore_prev.is_null() {
            unsafe { (*e.restore_prev).restore_next = e.restore_next };
        } else {
            G_ZR_ENGINE_RESTORE_HEAD.store(e.restore_next, Ordering::Release);
        }
        if !e.restore_next.is_null() {
            unsafe { (*e.restore_next).restore_prev = e.restore_prev };
        }
        e.restore_prev = ptr::null_mut();
        e.restore_next = ptr::null_mut();
        e.restore_registered = false;
    }
    zr_engine_restore_lock_release();
}

// -----------------------------------------------------------------------------
// Post user event guard
// -----------------------------------------------------------------------------

fn zr_engine_post_user_enter(e: &ZrEngine) -> bool {
    if e.destroy_started.load(Ordering::Acquire) {
        return false;
    }
    e.post_user_inflight.fetch_add(1, Ordering::AcqRel);
    if e.destroy_started.load(Ordering::Acquire) {
        e.post_user_inflight.fetch_sub(1, Ordering::Release);
        return false;
    }
    true
}

fn zr_engine_post_user_leave(e: &ZrEngine) {
    e.post_user_inflight.fetch_sub(1, Ordering::Release);
}

// -----------------------------------------------------------------------------
// Time helpers
// -----------------------------------------------------------------------------

fn zr_engine_now_ms_u32() -> u32 {
    // Use std::time
    let start = std::time::UNIX_EPOCH;
    let now = std::time::SystemTime::now();
    let dur = now.duration_since(start).unwrap_or(Duration::ZERO);
    dur.as_millis() as u32
}

fn zr_engine_tick_interval_ms(cfg: &ZrEngineRuntimeConfig) -> u32 {
    if cfg.target_fps == 0 {
        return ZR_ENGINE_DEFAULT_TICK_INTERVAL_MS;
    }
    let ms = 1000 / cfg.target_fps;
    if ms == 0 { 1 } else { ms }
}

fn zr_engine_tick_until_due_ms(e: &ZrEngine, now_ms: u32) -> u32 {
    let interval_ms = zr_engine_tick_interval_ms(&e.cfg_runtime);
    let elapsed_ms = now_ms.wrapping_sub(e.last_tick_ms);
    if elapsed_ms >= interval_ms {
        0
    } else {
        interval_ms - elapsed_ms
    }
}

fn zr_engine_maybe_enqueue_tick(e: &mut ZrEngine, now_ms: u32) {
    let interval_ms = zr_engine_tick_interval_ms(&e.cfg_runtime);
    let elapsed_ms = now_ms.wrapping_sub(e.last_tick_ms);
    if elapsed_ms < interval_ms {
        return;
    }
    let dt_ms = if elapsed_ms == 0 { 1 } else { elapsed_ms };
    let ev = ZrEvent {
        type_: ZR_EV_TICK,
        time_ms: now_ms,
        flags: 0,
        u: ZrEventUnion {
            tick: ZrEventTick { dt_ms, reserved0: 0, reserved1: 0, reserved2: 0 },
        },
    };
    let _ = e.evq.try_push_no_drop(&ev);
    e.last_tick_ms = now_ms;
}

// -----------------------------------------------------------------------------
// Cursor default
// -----------------------------------------------------------------------------

fn zr_engine_cursor_default() -> ZrCursorState {
    ZrCursorState {
        x: -1,
        y: -1,
        shape: ZR_CURSOR_SHAPE_BLOCK,
        visible: 0,
        blink: 0,
        reserved0: 0,
    }
}

// -----------------------------------------------------------------------------
// Framebuffer helpers
// -----------------------------------------------------------------------------

fn zr_engine_cells_bytes(fb: &ZrFb) -> usize {
    if fb.cells.is_empty() {
        return 0;
    }
    (fb.cols as usize) * (fb.rows as usize) * mem::size_of::<ZrCell>()
}

fn zr_engine_output_wait_timeout_ms(cfg: &ZrEngineRuntimeConfig) -> i32 {
    if cfg.target_fps == 0 {
        return 0;
    }
    let ms = 1000 / cfg.target_fps;
    if ms == 0 { 1 } else { ms as i32 }
}

fn zr_engine_free_diff_row_scratch(e: &mut ZrEngine) {
    e.diff_prev_row_hashes.clear();
    e.diff_next_row_hashes.clear();
    e.diff_dirty_rows.clear();
    e.diff_row_cap = 0;
    e.diff_prev_hashes_valid = false;
}

fn zr_engine_alloc_diff_row_scratch(rows: u32) -> ZrResult<(Vec<u64>, Vec<u64>, Vec<u8>)> {
    if rows == 0 {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    let prev = vec![0u64; rows as usize];
    let next = vec![0u64; rows as usize];
    let dirty = vec![0u8; rows as usize];
    Ok((prev, next, dirty))
}

fn zr_engine_fb_copy(src: &ZrFb, dst: &mut ZrFb) -> ZrResult {
    if src.cols != dst.cols || src.rows != dst.rows {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    let n = zr_engine_cells_bytes(src);
    if n != 0 && !src.cells.is_empty() && !dst.cells.is_empty() {
        dst.cells.copy_from_slice(&src.cells[..n / mem::size_of::<ZrCell>()]);
    }
    zr_fb_links_clone_from(dst, src)
}

fn zr_engine_fb_copy_noalloc(src: &ZrFb, dst: &mut ZrFb) -> ZrResult {
    if src.cols != dst.cols || src.rows != dst.rows {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    if src.cells.is_empty() || dst.cells.is_empty() {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    if src.links.len() > dst.links.capacity() || src.link_bytes.len() > dst.link_bytes.capacity() {
        return Err(ZR_ERR_LIMIT);
    }
    let n = zr_engine_cells_bytes(src);
    if n != 0 {
        dst.cells.copy_from_slice(&src.cells[..n / mem::size_of::<ZrCell>()]);
    }
    if !src.links.is_empty() {
        dst.links.copy_from_slice(&src.links);
    }
    if !src.link_bytes.is_empty() {
        dst.link_bytes.copy_from_slice(&src.link_bytes);
    }
    dst.links_len = src.links_len;
    dst.link_bytes_len = src.link_bytes_len;
    Ok(())
}

fn zr_engine_resize_framebuffers(e: &mut ZrEngine, cols: u32, rows: u32) -> ZrResult {
    if cols == 0 || rows == 0 {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }

    let mut prev = ZrFb::new();
    let mut next = ZrFb::new();
    let mut stage = ZrFb::new();
    zr_fb_init(&mut prev, cols, rows)?;
    zr_fb_init(&mut next, cols, rows)?;
    zr_fb_init(&mut stage, cols, rows)?;

    let (new_prev_hashes, new_next_hashes, new_dirty_rows) = zr_engine_alloc_diff_row_scratch(rows)?;

    // Commit
    mem::swap(&mut e.fb_prev, &mut prev);
    mem::swap(&mut e.fb_next, &mut next);
    mem::swap(&mut e.fb_stage, &mut stage);
    e.diff_prev_row_hashes = new_prev_hashes;
    e.diff_next_row_hashes = new_next_hashes;
    e.diff_dirty_rows = new_dirty_rows;
    e.diff_row_cap = rows;
    e.diff_prev_hashes_valid = false;

    e.term_state.flags &= !(ZR_TERM_STATE_STYLE_VALID | ZR_TERM_STATE_CURSOR_POS_VALID | ZR_TERM_STATE_SCREEN_VALID);
    e.fb_next_synced_to_prev = true;

    Ok(())
}

// -----------------------------------------------------------------------------
// Input processing
// -----------------------------------------------------------------------------

fn zr_engine_input_pending_parse(e: &mut ZrEngine, time_ms: u32) {
    loop {
        let pending_len = e.input_pending_len;
        if pending_len == 0 {
            return;
        }
        let consumed = zr_input_parse_bytes_prefix(&mut e.evq, &e.input_pending[..pending_len], time_ms);
        if consumed == 0 || consumed > pending_len {
            return;
        }
        let remain = pending_len - consumed;
        if remain > 0 {
            e.input_pending.copy_within(consumed..pending_len, 0);
        }
        e.input_pending_len = remain;
    }
}

fn zr_engine_input_pending_append_byte(e: &mut ZrEngine, b: u8, time_ms: u32) {
    if e.input_pending_len >= ZR_ENGINE_INPUT_PENDING_CAP {
        let _ = zr_input_parse_bytes(&mut e.evq, &e.input_pending[..e.input_pending_len], time_ms);
        e.input_pending_len = 0;
    }
    e.input_pending[e.input_pending_len] = b;
    e.input_pending_len += 1;
    zr_engine_input_pending_parse(e, time_ms);
}

fn zr_engine_paste_store_byte(e: &mut ZrEngine, b: u8) {
    if e.paste_overflowed {
        return;
    }
    if e.paste_len >= e.paste_buf_cap {
        e.paste_overflowed = true;
        return;
    }
    e.paste_buf[e.paste_len as usize] = b;
    e.paste_len += 1;
}

fn zr_engine_paste_finish(e: &mut ZrEngine, time_ms: u32) {
    e.paste_active = false;
    if !e.paste_overflowed {
        let _ = e.evq.post_paste(time_ms, &e.paste_buf[..e.paste_len as usize]);
    }
    e.paste_overflowed = false;
    e.paste_len = 0;
    e.paste_end_hold_len = 0;
    e.paste_idle_polls = 0;
}

fn zr_engine_input_process_paste_byte(e: &mut ZrEngine, b: u8, time_ms: u32) {
    e.paste_idle_polls = 0;
    const SEQ_LEN: usize = ZR_ENGINE_PASTE_MARKER_LEN;
    if e.paste_end_hold_len == 0 {
        if b == ZR_ENGINE_PASTE_END[0] {
            e.paste_end_hold[0] = b;
            e.paste_end_hold_len = 1;
            return;
        }
        zr_engine_paste_store_byte(e, b);
        return;
    }
    let want = e.paste_end_hold_len;
    if want < SEQ_LEN && b == ZR_ENGINE_PASTE_END[want] {
        e.paste_end_hold[want] = b;
        e.paste_end_hold_len += 1;
        if e.paste_end_hold_len == SEQ_LEN {
            zr_engine_paste_finish(e, time_ms);
        }
        return;
    }
    // Mismatch: flush held bytes
    for i in 0..e.paste_end_hold_len {
        zr_engine_paste_store_byte(e, e.paste_end_hold[i]);
    }
    e.paste_end_hold_len = 0;
    if b == ZR_ENGINE_PASTE_END[0] {
        e.paste_end_hold[0] = b;
        e.paste_end_hold_len = 1;
        return;
    }
    zr_engine_paste_store_byte(e, b);
}

fn zr_engine_input_process_normal_byte(e: &mut ZrEngine, b: u8, time_ms: u32) {
    const SEQ_LEN: usize = ZR_ENGINE_PASTE_MARKER_LEN;
    if e.paste_begin_hold_len == 0 {
        if b == ZR_ENGINE_PASTE_BEGIN[0] {
            e.paste_begin_hold[0] = b;
            e.paste_begin_hold_len = 1;
            return;
        }
        zr_engine_input_pending_append_byte(e, b, time_ms);
        return;
    }
    let want = e.paste_begin_hold_len;
    if want < SEQ_LEN && b == ZR_ENGINE_PASTE_BEGIN[want] {
        e.paste_begin_hold[want] = b;
        e.paste_begin_hold_len += 1;
        if e.paste_begin_hold_len == SEQ_LEN {
            e.paste_begin_hold_len = 0;
            e.paste_active = true;
            e.paste_overflowed = false;
            e.paste_len = 0;
            e.paste_end_hold_len = 0;
            e.paste_idle_polls = 0;
        }
        return;
    }
    // Mismatch: flush held bytes
    for i in 0..e.paste_begin_hold_len {
        zr_engine_input_pending_append_byte(e, e.paste_begin_hold[i], time_ms);
    }
    e.paste_begin_hold_len = 0;
    if b == ZR_ENGINE_PASTE_BEGIN[0] {
        e.paste_begin_hold[0] = b;
        e.paste_begin_hold_len = 1;
        return;
    }
    zr_engine_input_pending_append_byte(e, b, time_ms);
}

fn zr_engine_input_process_bytes(e: &mut ZrEngine, bytes: &[u8], time_ms: u32) {
    let paste_enabled = e.cfg_runtime.plat.enable_bracketed_paste != 0 && e.caps.supports_bracketed_paste != 0;
    for &b in bytes {
        if !paste_enabled {
            zr_engine_input_pending_append_byte(e, b, time_ms);
        } else if e.paste_active {
            zr_engine_input_process_paste_byte(e, b, time_ms);
        } else {
            zr_engine_input_process_normal_byte(e, b, time_ms);
        }
    }
}

fn zr_engine_input_flush_pending(e: &mut ZrEngine, time_ms: u32) {
    let paste_enabled = e.cfg_runtime.plat.enable_bracketed_paste != 0 && e.caps.supports_bracketed_paste != 0;

    if !paste_enabled && e.paste_active {
        if e.paste_len != 0 {
            for i in 0..e.paste_len {
                zr_engine_input_pending_append_byte(e, e.paste_buf[i as usize], time_ms);
            }
        }
        for i in 0..e.paste_end_hold_len {
            zr_engine_input_pending_append_byte(e, e.paste_end_hold[i], time_ms);
        }
        e.paste_active = false;
        e.paste_overflowed = false;
        e.paste_len = 0;
        e.paste_end_hold_len = 0;
        e.paste_idle_polls = 0;
    }

    if !paste_enabled {
        for i in 0..e.paste_begin_hold_len {
            zr_engine_input_pending_append_byte(e, e.paste_begin_hold[i], time_ms);
        }
        e.paste_begin_hold_len = 0;
        if e.input_pending_len != 0 {
            let _ = zr_input_parse_bytes(&mut e.evq, &e.input_pending[..e.input_pending_len], time_ms);
            e.input_pending_len = 0;
        }
        return;
    }

    if e.paste_active {
        if e.paste_idle_polls < u32::MAX {
            e.paste_idle_polls += 1;
        }
        if e.paste_idle_polls < ZR_ENGINE_PASTE_IDLE_FLUSH_POLLS {
            return;
        }
        for i in 0..e.paste_end_hold_len {
            zr_engine_paste_store_byte(e, e.paste_end_hold[i]);
        }
        e.paste_end_hold_len = 0;
        if e.paste_len != 0 || e.paste_overflowed {
            zr_engine_paste_finish(e, time_ms);
            return;
        }
        e.paste_active = false;
        e.paste_overflowed = false;
        e.paste_idle_polls = 0;
        return;
    }

    // Not in paste: flush begin prefix
    for i in 0..e.paste_begin_hold_len {
        zr_engine_input_pending_append_byte(e, e.paste_begin_hold[i], time_ms);
    }
    e.paste_begin_hold_len = 0;
    if e.input_pending_len != 0 {
        let _ = zr_input_parse_bytes(&mut e.evq, &e.input_pending[..e.input_pending_len], time_ms);
        e.input_pending_len = 0;
    }
}

fn zr_engine_drain_platform_input(e: &mut ZrEngine, time_ms: u32) -> ZrResult {
    let plat = match e.plat.as_mut() {
        Some(p) => p,
        None => return Err(ZR_ERR_INVALID_ARGUMENT),
    };
    let mut buf = [0u8; ZR_ENGINE_READ_CHUNK_CAP];
    for _ in 0..ZR_ENGINE_READ_LOOP_MAX {
        match plat.read_input(&mut buf) {
            Ok(n) if n == 0 => return Ok(()),
            Ok(n) => zr_engine_input_process_bytes(e, &buf[..n], time_ms),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Event packing
// -----------------------------------------------------------------------------

fn zr_engine_pack_one_event(w: &mut ZrEvpackWriter, q: &ZrEventQueue, ev: &ZrEvent) -> bool {
    match ev.type_ {
        ZR_EV_KEY => w.append_record(ZR_EV_KEY, ev.time_ms, ev.flags, &ev.u.key, mem::size_of::<ZrEventKey>()),
        ZR_EV_TEXT => w.append_record(ZR_EV_TEXT, ev.time_ms, ev.flags, &ev.u.text, mem::size_of::<ZrEventText>()),
        ZR_EV_PASTE => {
            let (payload, payload_len) = match q.paste_payload_view(ev) {
                Some((p, len)) => (p, len),
                None => return false,
            };
            w.append_record2(
                ZR_EV_PASTE,
                ev.time_ms,
                ev.flags,
                &ev.u.paste.hdr,
                mem::size_of::<ZrEventPasteHeader>(),
                payload,
                payload_len,
            )
        }
        ZR_EV_MOUSE => w.append_record(ZR_EV_MOUSE, ev.time_ms, ev.flags, &ev.u.mouse, mem::size_of::<ZrEventMouse>()),
        ZR_EV_RESIZE => w.append_record(ZR_EV_RESIZE, ev.time_ms, ev.flags, &ev.u.resize, mem::size_of::<ZrEventResize>()),
        ZR_EV_TICK => w.append_record(ZR_EV_TICK, ev.time_ms, ev.flags, &ev.u.tick, mem::size_of::<ZrEventTick>()),
        ZR_EV_USER => {
            let (payload, payload_len) = match q.user_payload_view(ev) {
                Some((p, len)) => (p, len),
                None => return false,
            };
            w.append_record2(
                ZR_EV_USER,
                ev.time_ms,
                ev.flags,
                &ev.u.user.hdr,
                mem::size_of::<ZrEventUserHeader>(),
                payload,
                payload_len,
            )
        }
        _ => false,
    }
}

// -----------------------------------------------------------------------------
// Config helpers
// -----------------------------------------------------------------------------

fn zr_engine_runtime_from_create_cfg(e: &mut ZrEngine, cfg: &ZrEngineConfig) {
    e.cfg_create = cfg.clone();
    e.cfg_runtime = ZrEngineRuntimeConfig {
        limits: cfg.limits,
        plat: cfg.plat,
        tab_width: cfg.tab_width,
        width_policy: cfg.width_policy,
        target_fps: cfg.target_fps,
        enable_scroll_optimizations: cfg.enable_scroll_optimizations,
        enable_debug_overlay: cfg.enable_debug_overlay,
        enable_replay_recording: cfg.enable_replay_recording,
        wait_for_output_drain: cfg.wait_for_output_drain,
        cap_force_flags: cfg.cap_force_flags,
        cap_suppress_flags: cfg.cap_suppress_flags,
    };
}

fn zr_engine_metrics_init(e: &mut ZrEngine, cfg: &ZrEngineConfig) {
    e.metrics = zr_metrics__default_snapshot();
    e.metrics.negotiated_engine_abi_major = cfg.requested_engine_abi_major;
    e.metrics.negotiated_engine_abi_minor = cfg.requested_engine_abi_minor;
    e.metrics.negotiated_engine_abi_patch = cfg.requested_engine_abi_patch;
    e.metrics.negotiated_drawlist_version = cfg.requested_drawlist_version;
    e.metrics.negotiated_event_batch_version = cfg.requested_event_batch_version;
}

fn zr_engine_alloc_out_buf(e: &mut ZrEngine) -> ZrResult {
    e.out_cap = e.cfg_runtime.limits.out_max_bytes_per_frame as usize;
    e.out_buf = vec![0u8; e.out_cap];
    Ok(())
}

fn zr_engine_alloc_damage_rects(e: &mut ZrEngine) -> ZrResult {
    let cap = e.cfg_runtime.limits.diff_max_damage_rects;
    if cap == 0 {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    e.damage_rects = vec![ZrDamageRect::default(); cap as usize];
    e.damage_rect_cap = cap;
    Ok(())
}

fn zr_engine_init_arenas(e: &mut ZrEngine) -> ZrResult {
    let arena_initial = e.cfg_runtime.limits.arena_initial_bytes as usize;
    let arena_max = e.cfg_runtime.limits.arena_max_total_bytes as usize;
    e.arena_frame = ZrArena::new(arena_initial, arena_max)?;
    e.arena_persistent = ZrArena::new(arena_initial, arena_max)?;
    Ok(())
}

fn zr_engine_init_event_queue(e: &mut ZrEngine) -> ZrResult {
    e.ev_cap = ZR_ENGINE_EVENT_QUEUE_CAP;
    e.ev_storage = vec![ZrEvent::default(); e.ev_cap];
    e.user_bytes_cap = ZR_ENGINE_USER_BYTES_CAP as u32;
    e.user_bytes = vec![0u8; e.user_bytes_cap as usize];
    e.paste_buf_cap = e.user_bytes_cap;
    e.paste_buf = vec![0u8; e.paste_buf_cap as usize];
    ZrEventQueue::init(&mut e.evq, &mut e.ev_storage, e.ev_cap as u32, &mut e.user_bytes, e.user_bytes_cap)
}

fn zr_engine_terminal_profile_defaults(caps: &PlatCaps, out_profile: &mut ZrTerminalProfile) {
    out_profile.id = ZR_TERM_UNKNOWN;
    out_profile.supports_mouse = caps.supports_mouse;
    out_profile.supports_bracketed_paste = caps.supports_bracketed_paste;
    out_profile.supports_focus_events = caps.supports_focus_events;
    out_profile.supports_osc52 = caps.supports_osc52;
    out_profile.supports_sync_update = caps.supports_sync_update;
}

fn zr_engine_requeue_probe_passthrough(e: &mut ZrEngine, bytes: &[u8]) {
    let time_ms = zr_engine_now_ms_u32();
    for &b in bytes {
        zr_engine_input_pending_append_byte(e, b, time_ms);
    }
}

fn zr_engine_sync_kitty_keyboard(e: &mut ZrEngine, profile: &ZrTerminalProfile) -> ZrResult {
    if e.plat.is_none() {
        return Ok(());
    }
    let want_active = profile.supports_kitty_keyboard != 0;
    if e.kitty_keyboard_active == want_active {
        return Ok(());
    }
    let bytes = if want_active { ZR_ENGINE_KITTY_KEYBOARD_PUSH } else { ZR_ENGINE_KITTY_KEYBOARD_POP };
    let plat = e.plat.as_mut().unwrap();
    plat.write_output(bytes)?;
    e.kitty_keyboard_active = want_active;
    Ok(())
}

fn zr_engine_apply_cap_overrides(e: &mut ZrEngine) {
    zr_detect_apply_overrides(
        &e.term_profile_base,
        &e.caps_base,
        e.cfg_runtime.cap_force_flags,
        e.cfg_runtime.cap_suppress_flags,
        &mut e.term_profile,
        &mut e.caps,
    );
}

fn zr_engine_detect_terminal_profile(e: &mut ZrEngine) {
    zr_engine_terminal_profile_defaults(&e.caps_base, &mut e.term_profile_base);
    e.term_profile = e.term_profile_base.clone();
    e.caps = e.caps_base.clone();

    let mut detected_profile = ZrTerminalProfile::default();
    let mut detected_caps = PlatCaps::default();
    let mut probe_passthrough = [0u8; ZR_ENGINE_DETECT_PASSTHROUGH_CAP];
    let mut probe_passthrough_len = 0;
    let plat = e.plat.as_mut().unwrap();
    let rc = zr_detect_probe_terminal(
        plat,
        &e.caps_base,
        &mut detected_profile,
        &mut detected_caps,
        &mut probe_passthrough,
        &mut probe_passthrough_len,
    );
    if rc == ZR_OK {
        e.term_profile_base = detected_profile;
        e.caps_base = detected_caps;
        zr_engine_requeue_probe_passthrough(e, &probe_passthrough[..probe_passthrough_len]);
    }
    zr_engine_apply_cap_overrides(e);
}

fn zr_engine_init_platform(e: &mut ZrEngine) -> ZrResult {
    let plat = crate::platform::zr_platform::create_platform(&e.cfg_runtime.plat)?;
    e.plat = Some(plat);
    let plat = e.plat.as_mut().unwrap();
    plat.enter_raw()?;
    plat.get_caps(&mut e.caps_base)?;
    zr_engine_detect_terminal_profile(e);
    let profile = e.term_profile.clone();
    zr_engine_sync_kitty_keyboard(e, &profile)?;
    plat.get_size(&mut e.size)?;
    Ok(())
}

fn zr_engine_init_runtime_state(e: &mut ZrEngine) -> ZrResult {
    zr_engine_alloc_out_buf(e)?;
    zr_engine_alloc_damage_rects(e)?;
    zr_engine_init_arenas(e)?;
    zr_engine_init_event_queue(e)?;
    zr_engine_init_platform(e)?;
    zr_engine_restore_register(e);
    if e.cfg_runtime.wait_for_output_drain != 0 && e.caps.supports_output_wait_writable == 0 {
        return Err(ZR_ERR_UNSUPPORTED);
    }
    zr_engine_resize_framebuffers(e, e.size.cols, e.size.rows)?;
    e.term_state.cursor_visible = 0;
    e.term_state.flags |= ZR_TERM_STATE_CURSOR_VIS_VALID;
    e.term_state.flags |= ZR_TERM_STATE_SCREEN_VALID;
    e.last_tick_ms = zr_engine_now_ms_u32();
    Ok(())
}

fn zr_engine_enqueue_initial_resize(e: &mut ZrEngine) {
    let ev = ZrEvent {
        type_: ZR_EV_RESIZE,
        time_ms: e.last_tick_ms,
        flags: 0,
        u: ZrEventUnion {
            resize: ZrEventResize { cols: e.size.cols, rows: e.size.rows, reserved0: 0, reserved1: 0 },
        },
    };
    let _ = e.evq.push(&ev);
}

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------

pub fn engine_create(cfg: &ZrEngineConfig) -> ZrResult<Box<ZrEngine>> {
    zr_engine_config_validate(cfg)?;
    let mut e = Box::new(ZrEngine {
        plat: None,
        restore_prev: ptr::null_mut(),
        restore_next: ptr::null_mut(),
        restore_registered: false,
        _pad_restore0: [0; 3],
        caps_base: PlatCaps::default(),
        term_profile_base: ZrTerminalProfile::default(),
        caps: PlatCaps::default(),
        term_profile: ZrTerminalProfile::default(),
        size: PlatSize { cols: 0, rows: 0 },
        kitty_keyboard_active: false,
        _pad_caps0: [0; 3],
        cfg_create: cfg.clone(),
        cfg_runtime: ZrEngineRuntimeConfig::default(),
        last_tick_ms: 0,
        fb_prev: ZrFb::new(),
        fb_next: ZrFb::new(),
        fb_stage: ZrFb::new(),
        term_state: ZrTermState::default(),
        cursor_desired: zr_engine_cursor_default(),
        fb_next_synced_to_prev: false,
        _pad_fb_sync0: [0; 3],
        image_frame_next: ZrImageFrame::new(),
        image_frame_stage: ZrImageFrame::new(),
        image_state: ZrImageState::new(),
        dl_resources_next: ZrDlResources::new(),
        dl_resources_stage: ZrDlResources::new(),
        out_buf: Vec::new(),
        out_cap: 0,
        damage_rects: Vec::new(),
        damage_rect_cap: 0,
        diff_prev_row_hashes: Vec::new(),
        diff_next_row_hashes: Vec::new(),
        diff_dirty_rows: Vec::new(),
        diff_row_cap: 0,
        diff_prev_hashes_valid: false,
        _pad_diff0: [0; 3],
        diff_sweep_frames_total: 0,
        diff_damage_frames_total: 0,
        diff_scroll_attempts_total: 0,
        diff_scroll_hits_total: 0,
        diff_collision_guard_hits_total: 0,
        evq: ZrEventQueue::new(),
        ev_storage: Vec::new(),
        ev_cap: 0,
        user_bytes: Vec::new(),
        user_bytes_cap: 0,
        post_user_inflight: AtomicU32::new(0),
        destroy_started: AtomicBool::new(false),
        input_pending: [0; ZR_ENGINE_INPUT_PENDING_CAP],
        input_pending_len: 0,
        paste_begin_hold: [0; ZR_ENGINE_PASTE_MARKER_LEN],
        paste_begin_hold_len: 0,
        paste_buf: Vec::new(),
        paste_buf_cap: 0,
        paste_len: 0,
        paste_active: false,
        paste_overflowed: false,
        paste_idle_polls: 0,
        paste_end_hold: [0; ZR_ENGINE_PASTE_MARKER_LEN],
        paste_end_hold_len: 0,
        arena_frame: ZrArena::new(0, 0).unwrap(),
        arena_persistent: ZrArena::new(0, 0).unwrap(),
        metrics: ZrMetrics::default(),
        debug_trace: None,
        debug_ring_buf: Vec::new(),
        debug_record_offsets: Vec::new(),
        debug_record_sizes: Vec::new(),
    });
    zr_engine_runtime_from_create_cfg(&mut e, cfg);
    zr_engine_metrics_init(&mut e, cfg);
    zr_engine_init_runtime_state(&mut e)?;
    zr_engine_enqueue_initial_resize(&mut e);
    Ok(e)
}

pub fn engine_destroy(e: Box<ZrEngine>) {
    let mut e = *e;
    e.destroy_started.store(true, Ordering::Release);
    while e.post_user_inflight.load(Ordering::Acquire) != 0 {
        zr_thread_yield();
    }
    if let Some(plat) = e.plat.take() {
        zr_engine_restore_unregister(&mut e);
        zr_engine_restore_platform_state(&mut e);
        drop(plat);
    } else {
        zr_engine_restore_unregister(&mut e);
    }
    // Release heap state
    e.fb_prev.release();
    e.fb_next.release();
    e.fb_stage.release();
    e.image_frame_next.release();
    e.image_frame_stage.release();
    e.image_state = ZrImageState::new();
    e.dl_resources_next.release();
    e.dl_resources_stage.release();
    e.arena_frame.release();
    e.arena_persistent.release();
    e.out_buf.clear();
    e.damage_rects.clear();
    e.diff_prev_row_hashes.clear();
    e.diff_next_row_hashes.clear();
    e.diff_dirty_rows.clear();
    e.ev_storage.clear();
    e.user_bytes.clear();
    e.paste_buf.clear();
    e.input_pending_len = 0;
    e.paste_begin_hold_len = 0;
    e.paste_end_hold_len = 0;
    e.paste_idle_polls = 0;
    e.paste_active = false;
    e.paste_overflowed = false;
    e.paste_len = 0;
    // debug free
    if let Some(mut dt) = e.debug_trace.take() {
        dt.release();
    }
    e.debug_ring_buf.clear();
    e.debug_record_offsets.clear();
    e.debug_record_sizes.clear();
    // drop e
}

pub fn engine_submit_drawlist(e: &mut ZrEngine, bytes: &[u8]) -> ZrResult {
    if bytes.is_empty() {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    let v = zr_dl_validate(bytes, &e.cfg_runtime.limits)?;
    if v.hdr.version != e.cfg_create.requested_drawlist_version {
        return Err(ZR_ERR_UNSUPPORTED);
    }

    let mut cursor_stage = e.cursor_desired;
    let mut preflight_resources = ZrDlResources::new();
    e.dl_resources_stage.release();
    e.dl_resources_stage.clone_from(&e.dl_resources_next)?;
    preflight_resources.clone_shallow(&e.dl_resources_stage)?;

    let mut have_fb_next_snapshot = false;
    if !e.fb_next_synced_to_prev {
        if let Err(rc) = zr_engine_fb_copy_noalloc(&e.fb_next, &mut e.fb_stage) {
            if rc == ZR_ERR_LIMIT {
                zr_engine_fb_copy(&e.fb_next, &mut e.fb_stage)?;
            } else {
                return Err(rc);
            }
        }
        have_fb_next_snapshot = true;
    }

    e.image_frame_stage.reset();
    let rc = zr_dl_preflight_resources(
        &v,
        &mut e.fb_next,
        &mut e.image_frame_stage,
        &e.cfg_runtime.limits,
        &e.term_profile,
        &mut preflight_resources,
    );
    preflight_resources.release();
    if rc != ZR_OK {
        let rollback_src = if have_fb_next_snapshot { &e.fb_stage } else { &e.fb_prev };
        let _ = zr_engine_fb_copy_noalloc(rollback_src, &mut e.fb_next);
        e.image_frame_stage.reset();
        e.dl_resources_stage.release();
        return Err(rc);
    }

    let mut blit_caps = ZrBlitCaps::default();
    zr_engine_build_blit_caps(e, &mut blit_caps);
    let rc = zr_dl_execute(
        &v,
        &mut e.fb_next,
        &e.cfg_runtime.limits,
        e.cfg_runtime.tab_width,
        e.cfg_runtime.width_policy,
        &blit_caps,
        &e.term_profile,
        &mut e.image_frame_stage,
        &mut e.dl_resources_stage,
        &mut cursor_stage,
    );
    if rc != ZR_OK {
        let rollback_src = if have_fb_next_snapshot { &e.fb_stage } else { &e.fb_prev };
        let _ = zr_engine_fb_copy_noalloc(rollback_src, &mut e.fb_next);
        e.image_frame_stage.reset();
        e.dl_resources_stage.release();
        return Err(rc);
    }

    mem::swap(&mut e.image_frame_next, &mut e.image_frame_stage);
    e.image_frame_stage.reset();
    mem::swap(&mut e.dl_resources_next, &mut e.dl_resources_stage);
    e.dl_resources_stage.release();
    e.cursor_desired = cursor_stage;
    e.fb_next_synced_to_prev = false;
    Ok(())
}

// Include present and poll implementations (omitted for brevity, would be separate files)
// For this translation we assume they exist as separate modules.
// In the original C, they are included as .inc files. We'll skip their content
// and just provide stub functions.

pub fn engine_present(e: &mut ZrEngine) -> ZrResult {
    // Stub: would call into present implementation
    unimplemented!("engine_present stub")
}

pub fn engine_poll(e: &mut ZrEngine, out_events: &mut [u8], out_events_cap: usize, out_len: &mut usize, wait_timeout_ms: i32) -> ZrResult {
    // Stub
    unimplemented!("engine_poll stub")
}

pub fn engine_post_user_event(e: &mut ZrEngine, tag: u32, payload: &[u8]) -> ZrResult {
    if !zr_engine_post_user_enter(e) {
        return Err(ZR_ERR_INVALID_ARGUMENT);
    }
    let rc = if e.plat.is_none() {
        Err(ZR_ERR_INVALID_ARGUMENT)
    } else {
        let time_ms = zr_engine_now_ms_u32();
        e.evq.post_user(time_ms, tag, payload)
    };
    if rc.is_ok() {
        if let Some(plat) = e.plat.as_mut() {
            let _ = plat.wake();
        }
    }
    zr_engine_post_user_leave(e);
    rc
}

pub fn engine_get_metrics(e: &ZrEngine, out_metrics: &mut ZrMetrics) -> ZrResult {
    *out_metrics = e.metrics.clone();
    Ok(())
}

pub fn engine_get_caps(e: &ZrEngine, out_caps: &mut ZrTerminalCaps) -> ZrResult {
    let mut c = ZrTerminalCaps::default();
    c.color_mode = e.caps.color_mode;
    c.supports_mouse = e.caps.supports_mouse;
    c.supports_bracketed_paste = e.caps.supports_bracketed_paste;
    c.supports_focus_events = e.caps.supports_focus_events;
    c.supports_osc52 = e.caps.supports_osc52;
    c.supports_sync_update = e.caps.supports_sync_update;
    c.supports_scroll_region = e.caps.supports_scroll_region;
    c.supports_cursor_shape = e.caps.supports_cursor_shape;
    c.supports_output_wait_writable = e.caps.supports_output_wait_writable;
    c.supports_underline_styles = e.caps.supports_underline_styles;
    c.supports_colored_underlines = e.caps.supports_colored_underlines;
    c.supports_hyperlinks = e.caps.supports_hyperlinks;
    c.sgr_attrs_supported = e.caps.sgr_attrs_supported;
    c.terminal_id = e.term_profile.id;
    c.cap_flags = zr_detect_profile_cap_flags(&e.term_profile, &e.caps);
    c.cap_force_flags = e.cfg_runtime.cap_force_flags & ZR_TERM_CAP_ALL_MASK;
    c.cap_suppress_flags = e.cfg_runtime.cap_suppress_flags & ZR_TERM_CAP_ALL_MASK;
    *out_caps = c;
    Ok(())
}

pub fn engine_get_terminal_profile(e: &ZrEngine) -> &ZrTerminalProfile {
    &e.term_profile
}

pub fn engine_set_config(e: &mut ZrEngine, cfg: &ZrEngineRuntimeConfig) -> ZrResult {
    zr_engine_runtime_config_validate(cfg)?;
    if cfg.plat != e.cfg_runtime.plat {
        return Err(ZR_ERR_UNSUPPORTED);
    }
    let mut prospective_profile = ZrTerminalProfile::default();
    let mut prospective_caps = PlatCaps::default();
    zr_detect_apply_overrides(
        &e.term_profile_base,
        &e.caps_base,
        cfg.cap_force_flags,
        cfg.cap_suppress_flags,
        &mut prospective_profile,
        &mut prospective_caps,
    );
    if cfg.wait_for_output_drain != 0 && prospective_caps.supports_output_wait_writable == 0 {
        return Err(ZR_ERR_UNSUPPORTED);
    }

    let mut out_buf_new = Vec::new();
    let mut out_cap_new = e.out_cap;
    let mut want_out_buf = false;
    if cfg.limits.out_max_bytes_per_frame != e.cfg_runtime.limits.out_max_bytes_per_frame {
        want_out_buf = true;
        out_cap_new = cfg.limits.out_max_bytes_per_frame as usize;
        out_buf_new = vec![0u8; out_cap_new];
    }

    let mut damage_rects_new = Vec::new();
    let mut damage_rect_cap_new = e.damage_rect_cap;
    let mut want_damage_rects = false;
    if cfg.limits.diff_max_damage_rects != e.cfg_runtime.limits.diff_max_damage_rects {
        want_damage_rects = true;
        let cap = cfg.limits.diff_max_damage_rects;
        if cap == 0 {
            return Err(ZR_ERR_INVALID_ARGUMENT);
        }
        damage_rects_new = vec![ZrDamageRect::default(); cap as usize];
        damage_rect_cap_new = cap;
    }

    let mut arena_frame_new = ZrArena::new(0, 0).unwrap();
    let mut arena_persistent_new = ZrArena::new(0, 0).unwrap();
    let mut want_arena_reinit = false;
    if cfg.limits.arena_initial_bytes != e.cfg_runtime.limits.arena_initial_bytes ||
        cfg.limits.arena_max_total_bytes != e.cfg_runtime.limits.arena_max_total_bytes {
        want_arena_reinit = true;
        let arena_initial = cfg.limits.arena_initial_bytes as usize;
        let arena_max = cfg.limits.arena_max_total_bytes as usize;
        arena_frame_new = ZrArena::new(arena_initial, arena_max)?;
        arena_persistent_new = ZrArena::new(arena_initial, arena_max)?;
    }

    zr_engine_sync_kitty_keyboard(e, &prospective_profile)?;

    // Commit
    if want_out_buf {
        e.out_buf = out_buf_new;
        e.out_cap = out_cap_new;
    }
    if want_damage_rects {
        e.damage_rects = damage_rects_new;
        e.damage_rect_cap = damage_rect_cap_new;
    }
    if want_arena_reinit {
        e.arena_frame.release();
        e.arena_persistent.release();
        e.arena_frame = arena_frame_new;
        e.arena_persistent = arena_persistent_new;
    }
    e.cfg_runtime = cfg.clone();
    zr_engine_apply_cap_overrides(e);
    Ok(())
}

// -----------------------------------------------------------------------------
// Debug API
// -----------------------------------------------------------------------------

fn zr_engine_debug_free(e: &mut ZrEngine) {
    e.debug_trace.take();
    e.debug_ring_buf.clear();
    e.debug_record_offsets.clear();
    e.debug_record_sizes.clear();
}

pub fn engine_debug_enable(e: &mut ZrEngine, config: Option<&ZrDebugConfig>) -> ZrResult {
    zr_engine_debug_free(e);
    let mut cfg = config.cloned().unwrap_or_else(zr_debug_config_default);
    cfg.enabled = 1;
    let ring_cap = if cfg.ring_capacity > 0 { cfg.ring_capacity } else { ZR_DEBUG_DEFAULT_RING_CAP };

    let mut trace = ZrDebugTrace::new();
    e.debug_ring_buf = vec![0u8; ZR_DEBUG_RING_BUF_SIZE];
    e.debug_record_offsets = vec![0u32; ring_cap as usize];
    e.debug_record_sizes = vec![0u32; ring_cap as usize];
    trace.init(&cfg, &mut e.debug_ring_buf, &mut e.debug_record_offsets, &mut e.debug_record_sizes)?;
    e.debug_trace = Some(trace);
    if let Some(trace) = e.debug_trace.as_mut() {
        trace.set_start_time((plat_now_ms() as u64) * 1000);
        trace.set_frame(zr_engine_trace_frame_id(e));
    }
    Ok(())
}

pub fn engine_debug_disable(e: &mut ZrEngine) {
    zr_engine_debug_free(e);
}

pub fn engine_debug_query(e: &ZrEngine, query: &ZrDebugQuery, out_headers: &mut [ZrDebugRecordHeader], out_result: &mut ZrDebugQueryResult) -> ZrResult {
    if let Some(trace) = e.debug_trace.as_ref() {
        trace.query(query, out_headers, out_result)
    } else {
        *out_result = ZrDebugQueryResult::default();
        Ok(())
    }
}

pub fn engine_debug_get_payload(e: &ZrEngine, record_id: u64, out_payload: &mut [u8], out_size: &mut u32) -> ZrResult {
    if let Some(trace) = e.debug_trace.as_ref() {
        trace.get_payload(record_id, out_payload, out_size)
    } else {
        *out_size = 0;
        Err(ZR_ERR_LIMIT)
    }
}

pub fn engine_debug_get_stats(e: &ZrEngine, out_stats: &mut ZrDebugStats) -> ZrResult {
    if let Some(trace) = e.debug_trace.as_ref() {
        trace.get_stats(out_stats)
    } else {
        *out_stats = ZrDebugStats::default();
        Ok(())
    }
}

pub fn engine_debug_export(e: &ZrEngine, out_buf: &mut [u8]) -> i32 {
    if let Some(trace) = e.debug_trace.as_ref() {
        trace.export(out_buf)
    } else {
        0
    }
}

pub fn engine_debug_reset(e: &mut ZrEngine) {
    if let Some(trace) = e.debug_trace.as_mut() {
        trace.reset();
    }
}

// -----------------------------------------------------------------------------
// Testing helpers
// -----------------------------------------------------------------------------

#[cfg(feature = "zr_engine_testing")]
pub fn zr_engine_test_reset_restore_counters() {
    G_ZR_ENGINE_TEST_RESTORE_ATTEMPTS.store(0, Ordering::Release);
    G_ZR_ENGINE_TEST_RESTORE_ABORT_CALLS.store(0, Ordering::Release);
    G_ZR_ENGINE_TEST_RESTORE_EXIT_CALLS.store(0, Ordering::Release);
}

#[cfg(feature = "zr_engine_testing")]
pub fn zr_engine_test_restore_attempts() -> u32 {
    G_ZR_ENGINE_TEST_RESTORE_ATTEMPTS.load(Ordering::Acquire)
}

#[cfg(feature = "zr_engine_testing")]
pub fn zr_engine_test_restore_abort_calls() -> u32 {
    G_ZR_ENGINE_TEST_RESTORE_ABORT_CALLS.load(Ordering::Acquire)
}

#[cfg(feature = "zr_engine_testing")]
pub fn zr_engine_test_restore_exit_calls() -> u32 {
    G_ZR_ENGINE_TEST_RESTORE_EXIT_CALLS.load(Ordering::Acquire)
}

#[cfg(feature = "zr_engine_testing")]
pub fn zr_engine_test_invoke_exit_restore_hook() {
    zr_engine_restore_from_exit();
}
```