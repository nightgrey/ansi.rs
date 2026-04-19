// #![forbid(unsafe_code)]
// //! Presenter: state-tracked ANSI emission.
// //!
// //! The Presenter transforms buffer diffs into minimal terminal output by tracking
// //! the current terminal state and only emitting sequences when changes are needed.
// //!
// //! # Design Principles
// //!
// //! - **State tracking**: Track current style, link, and cursor to avoid redundant output
// //! - **Run grouping**: Use ChangeRuns to minimize cursor positioning
// //! - **Single write**: Buffer all output and flush once per frame
// //! - **Synchronized output**: Use DEC 2026 to prevent flicker on supported terminals
// //!
// //! # Usage
// //!
// //! ```ignore
// //! use ftui_render::presenter::Presenter;
// //! use ftui_render::buffer::Buffer;
// //! use ftui_render::diff::BufferDiff;
// //! use ftui_core::terminal_capabilities::Capabilities;
// //!
// //! let caps = Capabilities::detect();
// //! let mut presenter = Presenter::new(std::io::stdout(), caps);
// //!
// //! let mut current = Buffer::new(80, 24);
// //! let mut next = Buffer::new(80, 24);
// //! // ... render widgets into `next` ...
// //!
// //! let diff = BufferDiff::compute(&current, &next);
// //! presenter.present(&next, &diff)?;
// //! std::mem::swap(&mut current, &mut next);
// //! ```

// use crate::counting::CountingWriter;
// use crate::{Arena, Buffer, BufferDiff, Cell};
// use ansi::io::Write as _;
// use ansi::{Style, SynchronizedOutput, TextCursorEnable};
// use derive_more::{AsMut, AsRef};
// use geometry::Point;
// use std::io::{self, BufWriter, Write};
// use std::time::{Duration, Instant};
// use terminal::Capabilities;

// /// Size of the internal write buffer (64KB).
// const BUFFER_CAPACITY: usize = 64 * 1024;
// /// Maximum hyperlink URL length allowed in OSC 8 payloads.
// const MAX_SAFE_HYPERLINK_URL_BYTES: usize = 4096;

// #[inline]
// fn is_safe_hyperlink_url(url: &str) -> bool {
//     url.len() <= MAX_SAFE_HYPERLINK_URL_BYTES && !url.chars().any(char::is_control)
// }

// // =============================================================================
// // DP Cost Model for ANSI Emission
// // =============================================================================

// /// State-tracked ANSI presenter.
// ///
// /// Transforms buffer diffs into minimal terminal output by tracking
// /// the current terminal state and only emitting necessary escape sequences.
// #[derive(AsMut, AsRef)]
// pub struct Presenter<W: Write> {
//     /// Buffered writer for efficient output, with byte counting.
//     #[as_ref]
//     #[as_mut]
//     writer: CountingWriter<BufWriter<W>>,
//     /// Current style state (None = unknown/reset).
//     current_style: Option<Style>,
//     /// Current cursor X position (0-indexed). None = unknown.
//     cursor: Point<Option<u16>>,
//     /// Viewport Y offset (added to all row coordinates).
//     offset_y: u16,
//     /// Terminal capabilities for conditional output.
//     capabilities: Capabilities,
// }

// impl<W: Write> Presenter<W> {
//     /// Create a new presenter with the given writer and capabilities.
//     pub fn new(writer: W, capabilities: Capabilities) -> Self {
//         Self {
//             writer: CountingWriter::new(BufWriter::with_capacity(BUFFER_CAPACITY, writer)),
//             current_style: None,
//             cursor: Point::new(None, None),
//             offset_y: 0,
//             capabilities,
//         }
//     }

//     /// Get mutable access to the full counting writer stack.
//     ///
//     /// This exposes `CountingWriter<BufWriter<W>>` so callers can access
//     /// byte counting, buffered flush, etc.
//     pub fn counting_writer_mut(&mut self) -> &mut CountingWriter<BufWriter<W>> {
//         &mut self.writer
//     }

//     /// Set the viewport Y offset.
//     ///
//     /// All subsequent render operations will add this offset to row coordinates.
//     /// Useful for inline mode where the UI starts at a specific row.
//     pub fn set_viewport_offset_y(&mut self, offset: u16) {
//         self.offset_y = offset;
//     }

//     /// Get the terminal capabilities.
//     #[inline]
//     pub fn capabilities(&self) -> &Capabilities {
//         &self.capabilities
//     }

//     /// Present a frame using the given buffer and diff.
//     ///
//     /// This is the main entry point for rendering. It:
//     /// 1. Begins synchronized output (if supported)
//     /// 2. Emits changes based on the diff
//     /// 3. Resets style and closes links
//     /// 4. Ends synchronized output
//     /// 5. Flushes all buffered output
//     pub fn present(&mut self, buffer: &Buffer, diff: &BufferDiff) -> io::Result<()> {
//         self.present_with_pool(buffer, diff, None)
//     }

//     /// Present a frame with grapheme pool and link registry.
//     pub fn present_with_pool(
//         &mut self,
//         buffer: &Buffer,
//         diff: &BufferDiff,
//         pool: Option<&Arena>,
//     ) -> io::Result<()> {
//         let bracket_supported = self.capabilities.use_sync_output();

//         // Calculate runs upfront for stats, reusing the runs buffer.
//         let cells_changed = diff.len();

//         // Start stats collection
//         self.writer.reset();

//         // Begin synchronized output to prevent flicker.
//         // When sync brackets are supported, use DEC 2026 for atomic frame display.
//         // Otherwise, fall back to cursor-hiding to reduce visual flicker.
//         if bracket_supported {
//             if let Err(err) = self.writer.escape(SynchronizedOutput::Set) {
//                 // Begin writes can fail after partial bytes; best-effort close
//                 // avoids leaving the terminal parser in sync-output mode.
//                 let _ = self.writer.escape(SynchronizedOutput::Reset);
//                 let _ = self.writer.flush();
//                 return Err(err);
//             }
//         } else {
//             self.writer.escape(TextCursorEnable::Reset)?;
//         }

//         // Emit diff using run grouping for efficiency.
//         let emit_result = self.emit_diff_runs(buffer, pool);

//         // Always attempt to restore terminal state, even if diff emission failed.
//         let frame_end_result = self.finish_frame();

//         let bracket_end_result = if bracket_supported {
//             self.writer.escape(SynchronizedOutput::Reset)
//         } else {
//             self.writer.escape(TextCursorEnable::Set)
//         };

//         let flush_result = self.writer.flush();

//         // Prioritize terminal-state restoration errors over emission errors:
//         // if cleanup fails (reset/link-close/sync-end/flush), callers need that
//         // failure surfaced immediately to avoid leaving the terminal wedged.
//         let cleanup_error = frame_end_result
//             .err()
//             .or_else(|| bracket_end_result.err())
//             .or_else(|| flush_result.err());
//         if let Some(err) = cleanup_error {
//             return Err(err);
//         }
//         emit_result?;

//         Ok(())
//     }

//     /// Emit diff runs using the cost model and internal buffers.
//     ///
//     /// This allows advanced callers (like TerminalWriter) to drive the emission
//     /// phase manually while still benefiting from the optimization logic.
//     /// The caller must populate `self.runs_buf` before calling this (e.g. via `diff.runs_into`).
//     pub fn emit_diff_runs(&mut self, buffer: &Buffer, pool: Option<&Arena>) -> io::Result<()> {
//         // Group runs by row and apply cost model per row
//         let mut i = 0;
//         while i < self.runs.len() {
//             let row_y = self.runs[i].y;

//             // Collect all runs on this row
//             let row_start = i;
//             while i < self.runs.len() && self.runs[i].y == row_y {
//                 i += 1;
//             }
//             let row_runs = &self.runs[row_start..i];

//             let row = buffer.iter_row(row_y);
//             for span in plan.spans() {
//                 self.move_cursor_optimal(span.x0, span.y)?;
//                 // Hot path: avoid recomputing `y * width + x` for every cell.
//                 let start = span.x0 as usize;
//                 let end = span.x1 as usize;
//                 debug_assert!(start <= end);
//                 debug_assert!(end < row.len());
//                 let mut idx = start;
//                 while idx <= end {
//                     let cell = &row[idx];
//                     self.emit_cell(idx as u16, cell, pool)?;

//                     // Repair invalid wide-char tails.
//                     //
//                     // Direct wide chars are always safe to repair because they can
//                     // only span a small, fixed number of cells. Grapheme-pool refs
//                     // may encode much wider payloads (up to 15 cells), so blindly
//                     // repairing all missing tails can erase unrelated content later in
//                     // the row. We only extend the repair to width-2 grapheme refs,
//                     // where clearing a single orphan tail cell is still bounded.
//                     let mut advance = 1usize;
//                     let width = cell.content.width();
//                     let should_repair_invalid_tail = cell.content.as_char().is_some()
//                         || (cell.content.is_grapheme() && width == 2);
//                     if width > 1 && should_repair_invalid_tail {
//                         for off in 1..width {
//                             let tx = idx + off;
//                             if tx >= row.len() {
//                                 break;
//                             }
//                             if row[tx].is_continuation() {
//                                 if tx <= end {
//                                     advance = advance.max(off + 1);
//                                 }
//                                 continue;
//                             }
//                             // Orphan detected: repair with a space.
//                             self.move_cursor_optimal(tx as u16, span.y)?;
//                             self.emit_orphan_continuation_space(tx as u16, links)?;
//                             if tx <= end {
//                                 advance = advance.max(off + 1);
//                             }
//                         }
//                     }

//                     idx = idx.saturating_add(advance);
//                 }
//             }
//         }
//         Ok(())
//     }

//     /// Finish a frame by restoring neutral SGR state and closing any open link.
//     ///
//     /// Callers that drive emission manually through [`emit_diff_runs`] must
//     /// invoke this before returning control to non-UI terminal output.
//     pub fn finish_frame(&mut self) -> io::Result<()> {
//         let reset_result = ansi::sgr_reset(&mut self.writer);
//         self.current_style = None;

//         let hyperlink_close_result = if self.current_link.is_some() {
//             let res = ansi::hyperlink_end(&mut self.writer);
//             if res.is_ok() {
//                 self.current_link = None;
//             }
//             Some(res)
//         } else {
//             None
//         };

//         if let Some(err) = reset_result
//             .err()
//             .or_else(|| hyperlink_close_result.and_then(Result::err))
//         {
//             return Err(err);
//         }

//         Ok(())
//     }

//     /// Best-effort frame cleanup used on error and drop paths.
//     pub fn finish_frame_best_effort(&mut self) {
//         let _ = ansi::sgr_reset(&mut self.writer);
//         self.current_style = None;

//         if self.current_link.is_some() {
//             let _ = ansi::hyperlink_end(&mut self.writer);
//             self.current_link = None;
//         }
//     }

//     /// Emit a single cell.
//     fn emit_cell(&mut self, x: u16, cell: &Cell, pool: Option<&Arena>) -> io::Result<()> {
//         // Drift protection: Ensure cursor is synchronized before emitting content.
//         // This catches cases where the previous emission (e.g. a wide char) advanced
//         // the cursor further than the buffer index advanced (e.g. because the
//         // continuation cell was missing/overwritten in an invalid buffer state).
//         //
//         // If we detect drift, we force a re-synchronization.
//         if let Some(cx) = self.cursor_x {
//             if cx != x && !cell.is_continuation() {
//                 // Re-sync. We assume cursor_y is set because we are in a run.
//                 if let Some(y) = self.cursor_y {
//                     self.move_cursor_optimal(x, y)?;
//                 }
//             }
//         } else {
//             // No known cursor position: must sync.
//             if let Some(y) = self.cursor_y {
//                 self.move_cursor_optimal(x, y)?;
//             }
//         }

//         // Continuation cells are the tail cells of wide glyphs. Emitting the
//         // head glyph already advanced the terminal cursor by the full width, so
//         // we normally skip emitting these cells.
//         //
//         // If we ever start emitting at a continuation cell (e.g. a run begins
//         // mid-wide-character), we must still advance the terminal cursor by one
//         // cell to keep subsequent emissions aligned. We write a space to clear
//         // any potential garbage (orphan cleanup) rather than just skipping with CUF.
//         if cell.is_continuation() {
//             match self.cursor_x {
//                 // Cursor already advanced past this cell by a previously-emitted wide head.
//                 Some(cx) if cx > x => return Ok(()),
//                 Some(cx) => {
//                     // Cursor is positioned at (or before) this continuation cell:
//                     // Treat as orphan and overwrite with space to ensure clean state.
//                     if cx < x
//                         && let Some(y) = self.cursor_y
//                     {
//                         self.move_cursor_optimal(x, y)?;
//                     }
//                     return self.emit_orphan_continuation_space(x, links);
//                 }
//                 // Defensive: move_cursor_optimal should always set cursor_x before emit_cell is called.
//                 None => {
//                     if let Some(y) = self.cursor_y {
//                         self.move_cursor_optimal(x, y)?;
//                     }
//                     return self.emit_orphan_continuation_space(x, links);
//                 }
//             }
//         }

//         // Emit style changes if needed
//         self.emit_style_changes(cell)?;

//         // Emit link changes if needed
//         self.emit_link_changes(cell, links)?;

//         let (prepared_content, raw_width) = PreparedContent::from_cell(cell);

//         // Calculate effective width and check for zero-width content (e.g. combining marks)
//         // stored as standalone cells. These must be replaced to maintain grid alignment.
//         let is_zero_width_content = raw_width == 0 && !cell.is_empty() && !cell.is_continuation();

//         if is_zero_width_content {
//             // Replace with U+FFFD Replacement Character (width 1)
//             self.writer.write_all(b"\xEF\xBF\xBD")?;
//         } else {
//             // Emit normal content
//             self.emit_content(prepared_content, raw_width, pool)?;
//         }

//         // Update cursor position (character output advances cursor)
//         if let Some(cx) = self.cursor_x {
//             // Empty cells are emitted as spaces (width 1).
//             // Zero-width content replaced by U+FFFD is width 1.
//             let width = if cell.is_empty() || is_zero_width_content {
//                 1
//             } else {
//                 raw_width
//             };
//             self.cursor_x = Some(cx.saturating_add(width as u16));
//         }

//         Ok(())
//     }

//     /// Clear a continuation cell with a visually neutral blank.
//     ///
//     /// This path intentionally resets style and closes hyperlinks first so the
//     /// cleanup space cannot inherit stale state from the previous emitted cell.
//     fn emit_orphan_continuation_space(
//         &mut self,
//         x: u16,
//         links: Option<&LinkRegistry>,
//     ) -> io::Result<()> {
//         let blank = Cell::default();
//         self.emit_style_changes(&blank)?;
//         self.emit_link_changes(&blank, links)?;
//         self.writer.write_all(b" ")?;
//         self.cursor_x = Some(x.saturating_add(1));
//         Ok(())
//     }

//     /// Emit style changes if the cell style differs from current.
//     ///
//     /// Uses SGR delta: instead of resetting and re-applying all style properties,
//     /// we compute the minimal set of changes needed (fg delta, bg delta, attr
//     /// toggles). Falls back to reset+apply only when a full reset would be cheaper.
//     fn emit_style_changes(&mut self, cell: &Cell) -> io::Result<()> {
//         let new_style = CellStyle::from_cell(cell);

//         // Check if style changed
//         if self.current_style == Some(new_style) {
//             return Ok(());
//         }

//         match self.current_style {
//             None => {
//                 // No known style state: re-establish a full terminal style baseline.
//                 self.emit_style_full(new_style)?;
//             }
//             Some(old_style) => {
//                 self.emit_style_delta(old_style, new_style)?;
//             }
//         }

//         self.current_style = Some(new_style);
//         Ok(())
//     }

//     /// Full style apply (reset + set all properties). Used when previous state is unknown.
//     fn emit_style_full(&mut self, style: CellStyle) -> io::Result<()> {
//         ansi::sgr_reset(&mut self.writer)?;
//         if style.fg.a() > 0 {
//             ansi::sgr_fg_packed(&mut self.writer, style.fg)?;
//         }
//         if style.bg.a() > 0 {
//             ansi::sgr_bg_packed(&mut self.writer, style.bg)?;
//         }
//         if !style.attrs.is_empty() {
//             ansi::sgr_flags(&mut self.writer, style.attrs)?;
//         }
//         Ok(())
//     }

//     #[inline]
//     fn dec_len_u8(value: u8) -> u32 {
//         if value >= 100 {
//             3
//         } else if value >= 10 {
//             2
//         } else {
//             1
//         }
//     }

//     #[inline]
//     fn sgr_code_len(code: u8) -> u32 {
//         2 + Self::dec_len_u8(code) + 1
//     }

//     #[inline]
//     fn sgr_flags_len(flags: StyleFlags) -> u32 {
//         if flags.is_empty() {
//             return 0;
//         }
//         let mut count = 0u32;
//         let mut digits = 0u32;
//         for (flag, codes) in ansi::FLAG_TABLE {
//             if flags.contains(flag) {
//                 count += 1;
//                 digits += Self::dec_len_u8(codes.on);
//             }
//         }
//         if count == 0 {
//             return 0;
//         }
//         3 + digits + (count - 1)
//     }

//     #[inline]
//     fn sgr_flags_off_len(flags: StyleFlags) -> u32 {
//         if flags.is_empty() {
//             return 0;
//         }
//         let mut len = 0u32;
//         for (flag, codes) in ansi::FLAG_TABLE {
//             if flags.contains(flag) {
//                 len += Self::sgr_code_len(codes.off);
//             }
//         }
//         len
//     }

//     #[inline]
//     fn sgr_rgb_len(color: PackedRgba) -> u32 {
//         10 + Self::dec_len_u8(color.r()) + Self::dec_len_u8(color.g()) + Self::dec_len_u8(color.b())
//     }

//     /// Emit minimal SGR delta between old and new styles.
//     ///
//     /// Computes which properties changed and emits only those.
//     /// Falls back to reset+apply when that would produce fewer bytes.
//     fn emit_style_delta(&mut self, old: CellStyle, new: CellStyle) -> io::Result<()> {
//         let attrs_removed = old.attrs & !new.attrs;
//         let attrs_added = new.attrs & !old.attrs;
//         let fg_changed = old.fg != new.fg;
//         let bg_changed = old.bg != new.bg;

//         // Hot path for VFX-style workloads: attributes are unchanged and only
//         // colors vary. In this case, delta emission is always no worse than a
//         // reset+reapply baseline, so skip cost estimation and flag diff logic.
//         if old.attrs == new.attrs {
//             if fg_changed {
//                 ansi::sgr_fg_packed(&mut self.writer, new.fg)?;
//             }
//             if bg_changed {
//                 ansi::sgr_bg_packed(&mut self.writer, new.bg)?;
//             }
//             return Ok(());
//         }

//         let mut collateral = StyleFlags::empty();
//         if attrs_removed.contains(StyleFlags::BOLD) && new.attrs.contains(StyleFlags::DIM) {
//             collateral |= StyleFlags::DIM;
//         }
//         if attrs_removed.contains(StyleFlags::DIM) && new.attrs.contains(StyleFlags::BOLD) {
//             collateral |= StyleFlags::BOLD;
//         }

//         let mut delta_len = 0u32;
//         delta_len += Self::sgr_flags_off_len(attrs_removed);
//         delta_len += Self::sgr_flags_len(collateral);
//         delta_len += Self::sgr_flags_len(attrs_added);
//         if fg_changed {
//             delta_len += if new.fg.a() == 0 {
//                 5
//             } else {
//                 Self::sgr_rgb_len(new.fg)
//             };
//         }
//         if bg_changed {
//             delta_len += if new.bg.a() == 0 {
//                 5
//             } else {
//                 Self::sgr_rgb_len(new.bg)
//             };
//         }

//         let mut baseline_len = 4u32;
//         if new.fg.a() > 0 {
//             baseline_len += Self::sgr_rgb_len(new.fg);
//         }
//         if new.bg.a() > 0 {
//             baseline_len += Self::sgr_rgb_len(new.bg);
//         }
//         baseline_len += Self::sgr_flags_len(new.attrs);

//         if delta_len > baseline_len {
//             return self.emit_style_full(new);
//         }

//         // Handle attr removal: emit individual off codes
//         if !attrs_removed.is_empty() {
//             let collateral = ansi::sgr_flags_off(&mut self.writer, attrs_removed, new.attrs)?;
//             // Re-enable any collaterally disabled flags
//             if !collateral.is_empty() {
//                 ansi::sgr_flags(&mut self.writer, collateral)?;
//             }
//         }

//         // Handle attr addition: emit on codes for newly added flags
//         if !attrs_added.is_empty() {
//             ansi::sgr_flags(&mut self.writer, attrs_added)?;
//         }

//         // Handle fg color change
//         if fg_changed {
//             ansi::sgr_fg_packed(&mut self.writer, new.fg)?;
//         }

//         // Handle bg color change
//         if bg_changed {
//             ansi::sgr_bg_packed(&mut self.writer, new.bg)?;
//         }

//         Ok(())
//     }

//     /// Emit hyperlink changes if the cell link differs from current.
//     fn emit_link_changes(&mut self, cell: &Cell, links: Option<&LinkRegistry>) -> io::Result<()> {
//         // Respect capability policy so callers running in mux contexts don't
//         // emit OSC 8 sequences even if the raw capability flag is set.
//         if !self.hyperlinks_enabled {
//             if self.current_link.is_none() {
//                 return Ok(());
//             }
//             if self.current_link.is_some() {
//                 ansi::hyperlink_end(&mut self.writer)?;
//             }
//             self.current_link = None;
//             return Ok(());
//         }

//         let raw_link_id = cell.attrs.link_id();
//         let new_link = if raw_link_id == CellAttrs::LINK_ID_NONE {
//             None
//         } else {
//             Some(raw_link_id)
//         };

//         // Check if link changed
//         if self.current_link == new_link {
//             return Ok(());
//         }

//         // Close current link if open
//         if self.current_link.is_some() {
//             ansi::hyperlink_end(&mut self.writer)?;
//         }

//         // Open new link if present and resolvable
//         let actually_opened = if let (Some(link_id), Some(registry)) = (new_link, links)
//             && let Some(url) = registry.get(link_id)
//             && is_safe_hyperlink_url(url)
//         {
//             ansi::hyperlink_start(&mut self.writer, url)?;
//             true
//         } else {
//             false
//         };

//         // Only track as current if we actually opened it
//         self.current_link = if actually_opened { new_link } else { None };
//         Ok(())
//     }

//     /// Emit cell content after width/content classification.
//     fn emit_content(
//         &mut self,
//         content: PreparedContent,
//         raw_width: usize,
//         pool: Option<&Arena>,
//     ) -> io::Result<()> {
//         match content {
//             PreparedContent::Grapheme(grapheme_id) => {
//                 if let Some(pool) = pool
//                     && let Some(text) = pool.get(grapheme_id)
//                 {
//                     let safe = sanitize(text);
//                     if !safe.is_empty() && display_width(safe.as_ref()) == raw_width {
//                         return self.writer.write_all(safe.as_bytes());
//                     }
//                 }
//                 // Fallback when sanitization strips bytes or changes display width:
//                 // emit width-1 placeholders so the terminal cursor advances by the
//                 // exact number of cells encoded in the grapheme ID.
//                 if raw_width > 0 {
//                     for _ in 0..raw_width {
//                         self.writer.write_all(b"?")?;
//                     }
//                 }
//                 Ok(())
//             }
//             PreparedContent::Char(ch) => {
//                 if ch.is_ascii() {
//                     // Width-0 ASCII controls are filtered earlier via the
//                     // replacement-character path. The remaining ASCII controls
//                     // here are width-1 (`\n`/`\r`) and must still sanitize to
//                     // a visually neutral single cell.
//                     let byte = if ch.is_ascii_control() {
//                         b' '
//                     } else {
//                         ch as u8
//                     };
//                     return self.writer.write_all(&[byte]);
//                 }
//                 // Sanitize control characters that would break the grid.
//                 let safe_ch = if ch.is_control() { ' ' } else { ch };
//                 let mut buf = [0u8; 4];
//                 let encoded = safe_ch.encode_utf8(&mut buf);
//                 self.writer.write_all(encoded.as_bytes())
//             }
//             PreparedContent::Empty => {
//                 // Empty cell - emit space
//                 self.writer.write_all(b" ")
//             }
//         }
//     }

//     /// Move cursor to the specified position.
//     fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
//         // Skip if already at position
//         if self.cursor_x == Some(x) && self.cursor_y == Some(y) {
//             return Ok(());
//         }

//         // Use CUP (cursor position) for absolute positioning
//         ansi::cup(&mut self.writer, y.saturating_add(self.offset_y), x)?;
//         self.cursor_x = Some(x);
//         self.cursor_y = Some(y);
//         Ok(())
//     }

//     /// Move cursor using the cheapest available operation.
//     ///
//     /// Compares CUP (absolute), CHA (column-only), and CUF/CUB (relative)
//     /// to select the minimum-cost cursor movement.
//     fn move_cursor_optimal(&mut self, x: u16, y: u16) -> io::Result<()> {
//         // Skip if already at position
//         if self.cursor_x == Some(x) && self.cursor_y == Some(y) {
//             return Ok(());
//         }

//         // Decide cheapest move
//         let same_row = self.cursor_y == Some(y);
//         let actual_y = y.saturating_add(self.offset_y);

//         if same_row {
//             if let Some(cx) = self.cursor_x {
//                 if x > cx {
//                     // Forward
//                     let dx = x - cx;
//                     let cuf = cost_model::cuf_cost(dx);
//                     let cha = cost_model::cha_cost(x);
//                     let cup = cost_model::cup_cost(actual_y, x);

//                     if cuf <= cha && cuf <= cup {
//                         ansi::cuf(&mut self.writer, dx)?;
//                     } else if cha <= cup {
//                         ansi::cha(&mut self.writer, x)?;
//                     } else {
//                         ansi::cup(&mut self.writer, actual_y, x)?;
//                     }
//                 } else if x < cx {
//                     // Backward
//                     let dx = cx - x;
//                     let cub = cost_model::cub_cost(dx);
//                     let cha = cost_model::cha_cost(x);
//                     let cup = cost_model::cup_cost(actual_y, x);

//                     if cha <= cub && cha <= cup {
//                         ansi::cha(&mut self.writer, x)?;
//                     } else if cub <= cup {
//                         ansi::cub(&mut self.writer, dx)?;
//                     } else {
//                         ansi::cup(&mut self.writer, actual_y, x)?;
//                     }
//                 } else {
//                     // Same column (should have been caught by early check, but for safety)
//                 }
//             } else {
//                 // Unknown x, same row (unlikely but possible if we only tracked y?)
//                 // Fallback to absolute
//                 ansi::cup(&mut self.writer, actual_y, x)?;
//             }
//         } else {
//             // Different row: CUP is the only option
//             ansi::cup(&mut self.writer, actual_y, x)?;
//         }

//         self.cursor_x = Some(x);
//         self.cursor_y = Some(y);
//         Ok(())
//     }

//     /// Clear the entire screen.
//     pub fn clear_screen(&mut self) -> io::Result<()> {
//         ansi::erase_display(&mut self.writer, ansi::EraseDisplayMode::All)?;
//         ansi::cup(&mut self.writer, 0, 0)?;
//         self.cursor_x = Some(0);
//         self.cursor_y = Some(0);
//         self.writer.flush()
//     }

//     /// Clear a single line.
//     pub fn clear_line(&mut self, y: u16) -> io::Result<()> {
//         self.move_cursor_to(0, y)?;
//         ansi::erase_line(&mut self.writer, EraseLineMode::All)?;
//         self.writer.flush()
//     }

//     /// Hide the cursor.
//     pub fn hide_cursor(&mut self) -> io::Result<()> {
//         ansi::cursor_hide(&mut self.writer)?;
//         self.writer.flush()
//     }

//     /// Show the cursor.
//     pub fn show_cursor(&mut self) -> io::Result<()> {
//         ansi::cursor_show(&mut self.writer)?;
//         self.writer.flush()
//     }

//     /// Position the cursor at the specified coordinates.
//     pub fn position_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
//         self.move_cursor_to(x, y)?;
//         self.writer.flush()
//     }

//     /// Reset the presenter state.
//     ///
//     /// Useful after resize or when terminal state is unknown.
//     pub fn reset(&mut self) {
//         self.current_style = None;
//         self.current_link = None;
//         self.cursor_x = None;
//         self.cursor_y = None;
//     }

//     /// Flush any buffered output.
//     pub fn flush(&mut self) -> io::Result<()> {
//         self.writer.flush()
//     }

//     /// Get the inner writer (consuming the presenter).
//     ///
//     /// Flushes any buffered data before returning the writer.
//     pub fn into_inner(self) -> Result<W, io::Error> {
//         self.writer
//             .into_inner() // CountingWriter -> BufWriter<W>
//             .into_inner() // BufWriter<W> -> Result<W, IntoInnerError>
//             .map_err(|e| e.into_error())
//     }
// }

// /// Statistics from a present() operation.
// ///
// /// Captures metrics for verifying O(changes) output size and detecting
// /// performance regressions.
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct PresentStats {
//     /// Bytes emitted for this frame.
//     pub bytes_emitted: u64,
//     /// Number of cells changed.
//     pub cells_changed: usize,
//     /// Number of runs (groups of consecutive changes).
//     pub run_count: usize,
//     /// Time spent in present().
//     pub duration: Duration,
// }

// impl PresentStats {
//     /// Create new stats with the given values.
//     #[inline]
//     pub fn new(
//         bytes_emitted: u64,
//         cells_changed: usize,
//         run_count: usize,
//         duration: Duration,
//     ) -> Self {
//         Self {
//             bytes_emitted,
//             cells_changed,
//             run_count,
//             duration,
//         }
//     }

//     /// Calculate bytes per cell changed.
//     ///
//     /// Returns 0.0 if no cells were changed.
//     #[inline]
//     pub fn bytes_per_cell(&self) -> f64 {
//         if self.cells_changed == 0 {
//             0.0
//         } else {
//             self.bytes_emitted as f64 / self.cells_changed as f64
//         }
//     }

//     /// Calculate bytes per run.
//     ///
//     /// Returns 0.0 if no runs.
//     #[inline]
//     pub fn bytes_per_run(&self) -> f64 {
//         if self.run_count == 0 {
//             0.0
//         } else {
//             self.bytes_emitted as f64 / self.run_count as f64
//         }
//     }

//     /// Check if output is within the expected budget.
//     ///
//     /// Uses conservative estimates for worst-case bytes per cell.
//     #[inline]
//     pub fn within_budget(&self) -> bool {
//         let budget = expected_max_bytes(self.cells_changed, self.run_count);
//         self.bytes_emitted <= budget
//     }

//     /// Log stats at debug level (requires tracing feature).
//     #[cfg(feature = "tracing")]
//     pub fn log(&self) {
//         tracing::debug!(
//             bytes = self.bytes_emitted,
//             cells_changed = self.cells_changed,
//             runs = self.run_count,
//             duration_us = self.duration.as_micros() as u64,
//             bytes_per_cell = format!("{:.1}", self.bytes_per_cell()),
//             "Present stats"
//         );
//     }

//     /// Log stats at debug level (no-op without tracing feature).
//     #[cfg(not(feature = "tracing"))]
//     pub fn log(&self) {
//         // No-op without tracing
//     }
// }

// impl Default for PresentStats {
//     fn default() -> Self {
//         Self {
//             bytes_emitted: 0,
//             cells_changed: 0,
//             run_count: 0,
//             duration: Duration::ZERO,
//         }
//     }
// }

// /// Expected bytes per cell change (approximate worst case).
// ///
// /// Worst case: cursor move (10) + full SGR reset+apply (25) + 4-byte UTF-8 char
// pub const BYTES_PER_CELL_MAX: u64 = 40;

// /// Bytes for sync output wrapper.
// pub const SYNC_OVERHEAD: u64 = 20;

// /// Bytes for cursor move sequence (CUP).
// pub const BYTES_PER_CURSOR_MOVE: u64 = 10;

// /// Calculate expected maximum bytes for a frame with given changes.
// ///
// /// This is a conservative budget for regression testing.
// #[inline]
// pub fn expected_max_bytes(cells_changed: usize, runs: usize) -> u64 {
//     // cursor move per run + cells * max_per_cell + sync overhead
//     (runs as u64 * BYTES_PER_CURSOR_MOVE)
//         + (cells_changed as u64 * BYTES_PER_CELL_MAX)
//         + SYNC_OVERHEAD
// }
