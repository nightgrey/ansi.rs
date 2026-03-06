use std::hash::{Hash, Hasher};
use std::io;
use ansi::escape;
use ansi::io::Write;
use ansi::sequences::*;
use geometry::Row;

use crate::buffer::Buffer;

use super::capabilities::Capabilities;
use super::cursor::Cursor;
use super::line::{transform_line, transform_line_relative};

/// Whether the rasterizer operates in fullscreen or inline mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterizerMode {
    /// Traditional fullscreen mode using alternate screen buffer.
    Fullscreen,
    /// Inline/CLI mode: renders within the normal scrollback region.
    Inline,
}

/// State for inline rendering.
struct InlineState {
    /// Number of rows the rasterizer "owns" in the terminal.
    owned_rows: usize,
    /// Whether this is the first render call.
    first_render: bool,
}

pub struct Rasterizer {
    output: Vec<u8>,
    prev: Buffer,
    cursor: Cursor,
    caps: Capabilities,
    is_fullscreen: bool,
    force_clear: bool,
    row_hashes: Vec<u64>,
    mode: RasterizerMode,
    inline: Option<InlineState>,
}

impl Rasterizer {
    /// Create a new rasterizer with the given screen dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            output: Vec::with_capacity(4096),
            prev: Buffer::new(width, height),
            cursor: Cursor::new(),
            caps: Capabilities::default(),
            is_fullscreen: false,
            force_clear: true,
            row_hashes: Vec::new(),
            mode: RasterizerMode::Fullscreen,
            inline: None,
        }
    }

    /// Create a rasterizer with explicit capabilities.
    pub fn with_capabilities(width: usize, height: usize, caps: Capabilities) -> Self {
        Self {
            caps,
            ..Self::new(width, height)
        }
    }

    /// Create a rasterizer in inline mode.
    pub fn inline(width: usize, height: usize) -> Self {
        Self {
            mode: RasterizerMode::Inline,
            inline: Some(InlineState {
                owned_rows: 0,
                first_render: true,
            }),
            ..Self::new(width, height)
        }
    }

    /// Create an inline rasterizer with explicit capabilities.
    pub fn inline_with_capabilities(width: usize, height: usize, caps: Capabilities) -> Self {
        Self {
            caps,
            ..Self::inline(width, height)
        }
    }

    /// Render a buffer, diffing against the previous frame.
    pub fn render(&mut self, buffer: &Buffer) {
        if self.mode == RasterizerMode::Inline {
            return self.render_inline(buffer);
        }
        let width = buffer.width;
        let height = buffer.height;

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape(&mut self.output, SynchronizedOutput::Enable);
        }

        // Handle dimension change or forced clear.
        if self.prev.width != width || self.prev.height != height || self.force_clear {
            escape(&mut self.output, Home);
            escape(&mut self.output, EraseDisplay);
            self.cursor.reset();
            self.prev = Buffer::new(width, height);
            self.row_hashes.clear();
            self.row_hashes.resize(height, u64::MAX);
            self.force_clear = false;
        }

        // Compute row hashes for the new buffer.
        let mut new_hashes = Vec::with_capacity(height);
        for y in 0..height {
            new_hashes.push(Self::hash_row(&buffer[Row(y)]));
        }

        // Ensure row_hashes is sized correctly (handles first render after init).
        if self.row_hashes.len() != height {
            self.row_hashes.resize(height, u64::MAX);
        }

        // Scroll optimization: detect uniform vertical scroll.
        if self.caps.contains(Capabilities::SCROLL_REGION | Capabilities::SCROLL) && height >= 3 {
            if let Some(scroll) = self.detect_scroll(&new_hashes, height) {
                self.apply_scroll(scroll, width, height, &mut new_hashes);
            }
        }

        escape(&mut self.output, TextCursor::Enable);

        // Clear-to-bottom optimization: scan from bottom upward for contiguous
        // all-empty new lines where old has content.
        let ed_row = self.find_ed_row(buffer, width, height);

        for y in 0..height {
            if y >= ed_row {
                // Everything from ed_row down can be erased in one shot.
                self.cursor.move_to(&mut self.output, ed_row, 0, self.caps);
                self.cursor.reset_pen(&mut self.output);
                escape(&mut self.output, EraseDisplayToEnd);
                break;
            }

            // Skip rows whose hash is unchanged.
            if new_hashes[y] == self.row_hashes[y] {
                continue;
            }

            transform_line(
                &mut self.output,
                &mut self.cursor,
                buffer,
                &self.prev,
                y,
                width,
                self.caps,
            );
        }

        self.row_hashes = new_hashes;

        self.cursor.reset_pen(&mut self.output);
        escape(&mut self.output, TextCursor::Enable);

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Disable).unwrap();
        }

        // Swap prev ← new.
        self.prev = buffer.clone();
    }

    /// Flush the accumulated output to a writer and clear the buffer.
    pub fn flush(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        if !self.output.is_empty() {
            w.write_all(&self.output)?;
            self.output.clear();
        }
        w.flush()
    }

    /// Mark the screen for a full clear on next render.
    pub fn invalidate(&mut self) {
        self.force_clear = true;
    }

    /// Handle a terminal resize.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.prev = Buffer::new(width, height);
        self.row_hashes.clear();
        self.force_clear = true;
    }

    /// Enter alternate screen buffer.
    pub fn enter_alt_screen(&mut self) {
        escape!(&mut self.output, AlternateScreen::Enable);
        self.is_fullscreen = true;
        self.force_clear = true;
    }

    /// Exit alternate screen buffer.
    pub fn exit_alt_screen(&mut self) {
        escape!(&mut self.output, Reset, AlternateScreen::Disable);
        self.is_fullscreen = false;
        self.cursor.reset();
        self.force_clear = true;
    }

    /// Access the internal output buffer (for testing).
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    /// Clear the output buffer without flushing.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    // ── Inline rendering ───────────────────────────────────────────

    /// Render in inline mode (no alternate screen, relative cursor only).
    fn render_inline(&mut self, buffer: &Buffer) {
        let width = buffer.width;
        let height = buffer.height;

        let inline = self.inline.as_mut().expect("inline state required");

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Enable);
        }

        escape(&mut self.output, TextCursor::Disable);

        if inline.first_render {
            // First render: emit each row with \n separators and EL.
            inline.first_render = false;
            inline.owned_rows = height;
            self.prev = Buffer::new(width, height);

            for y in 0..height {
                if y > 0 {
                    self.output.push(b'\n');
                }
                let row = &buffer[Row(y)];
                for col in 0..width {
                    let cell = &row[col];
                    self.cursor.update_pen(&mut self.output, cell.style());
                    if cell.is_empty() {
                        self.output.push(b' ');
                    } else {
                        let s = cell.as_str(&buffer.arena);
                        self.output.extend_from_slice(s.as_bytes());
                    }
                }
                self.cursor.reset_pen(&mut self.output);
                escape(&mut self.output, EraseLineToEnd);
            }

            self.cursor.row = height - 1;
            self.cursor.col = width;
        } else {
            // Subsequent renders: move up to the top of our owned region, diff rows.
            let old_owned = inline.owned_rows;

            if height > old_owned {
                // Growing: emit newlines to claim more rows.
                let extra = height - old_owned;
                for _ in 0..extra {
                    self.output.push(b'\n');
                }
                self.cursor.row += extra;
                inline.owned_rows = height;
            }

            // Move to the top of our owned region.
            if self.cursor.row > 0 {
                escape(&mut self.output, CursorUp(self.cursor.row));
            }
            escape(&mut self.output, CarriageReturn);
            self.cursor.row = 0;
            self.cursor.col = 0;

            // Resize prev if needed.
            if self.prev.width != width || self.prev.height != height {
                self.prev = Buffer::new(width, height);
            }

            // Diff each row using relative movement.
            for y in 0..height {
                transform_line_relative(
                    &mut self.output,
                    &mut self.cursor,
                    buffer,
                    &self.prev,
                    y,
                    width,
                    self.caps,
                );
            }

            // If we shrank, clear extra rows.
            if height < old_owned {
                for _ in height..old_owned {
                    self.cursor.move_to_relative(&mut self.output, self.cursor.row + 1, 0);
                    escape(&mut self.output, EraseLineToEnd);
                }
                // Move back to last row of content.
                if self.cursor.row > height - 1 {
                    let up = self.cursor.row - (height - 1);
                    escape(&mut self.output, CursorUp(up));
                    self.cursor.row = height - 1;
                }
                inline.owned_rows = height;
            }
        }

        self.cursor.reset_pen(&mut self.output);
        escape(&mut self.output, TextCursor::Enable);

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Disable);
        }

        self.prev = buffer.clone();
    }

    // ── Internal ────────────────────────────────────────────────────

    /// Detect a uniform vertical scroll by comparing old and new row hashes.
    ///
    /// Returns the scroll offset (positive = scroll up, negative = scroll down)
    /// if a sufficient majority of rows match at that offset.
    fn detect_scroll(&self, new_hashes: &[u64], height: usize) -> Option<isize> {
        // Pick the middle row as a probe.
        let probe_row = height / 2;
        let probe_hash = new_hashes[probe_row];

        // Don't probe sentinel hashes.
        if probe_hash == u64::MAX {
            return None;
        }

        // Search old hashes for where this hash appeared.
        let old_row = self.row_hashes.iter().position(|&h| h == probe_hash)?;

        // Candidate offset: new[probe_row] == old[old_row] → offset = old_row - probe_row
        let offset = old_row as isize - probe_row as isize;

        if offset == 0 {
            return None; // No scroll.
        }

        // Verify: count how many rows match at this offset.
        let mut matches = 0usize;
        for y in 0..height {
            let src = y as isize + offset;
            if src >= 0 && (src as usize) < height && new_hashes[y] == self.row_hashes[src as usize] {
                matches += 1;
            }
        }

        // Require 2/3 of rows to match.
        if matches >= height * 2 / 3 {
            Some(offset)
        } else {
            None
        }
    }

    /// Apply a detected scroll: emit DECSTBM + SU/SD, then update prev and row_hashes.
    fn apply_scroll(
        &mut self,
        offset: isize,
        width: usize,
        height: usize,
        new_hashes: &mut Vec<u64>,
    ) {
        let _ = new_hashes; // hashes are updated below

        // Set scroll region to full screen.
        escape!(&mut self.output, SetTopBottomMargins::Some(0, height - 1));

        // Move cursor into the scroll region.
        if offset > 0 {
            // Content moved up → scroll up (old rows shifted up by `offset`).
            escape(&mut self.output, CursorPosition(0, 0));
            escape(&mut self.output, ScrollUp(offset as usize));
        } else {
            // Content moved down → scroll down.
            escape(&mut self.output, CursorPosition(0, 0));
            escape(&mut self.output, ScrollDown((-offset) as usize));
        }

        // Reset scroll region.
        escape(&mut self.output, SetTopBottomMargins::None);

        // Update self.prev to reflect what the terminal now shows.
        let abs_offset = offset.unsigned_abs();
        let row_stride = width;

        if offset > 0 {
            // Scroll up: rows shifted up by `offset`.
            // copy_within: [offset*stride..] → [0..]
            self.prev.inner.copy_within(abs_offset * row_stride.., 0);
            // Clear the bottom `abs_offset` rows.
            let clear_start = (height - abs_offset) * row_stride;
            self.prev.inner[clear_start..].fill(crate::buffer::Cell::EMPTY);
            // Shift row_hashes similarly.
            self.row_hashes.copy_within(abs_offset.., 0);
            for h in &mut self.row_hashes[height - abs_offset..] {
                *h = u64::MAX;
            }
        } else {
            // Scroll down: rows shifted down by `abs_offset`.
            let len = height * row_stride;
            self.prev.inner.copy_within(..len - abs_offset * row_stride, abs_offset * row_stride);
            // Clear the top `abs_offset` rows.
            self.prev.inner[..abs_offset * row_stride].fill(crate::buffer::Cell::EMPTY);
            // Shift row_hashes.
            self.row_hashes.copy_within(..height - abs_offset, abs_offset);
            for h in &mut self.row_hashes[..abs_offset] {
                *h = u64::MAX;
            }
        }

        // Reset cursor state since we moved it.
        self.cursor.row = 0;
        self.cursor.col = 0;
    }

    /// Compute a hash for a row of cells.
    fn hash_row(row: &[crate::buffer::Cell]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for cell in row {
            cell.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Find the first row from which all remaining rows in the new buffer are
    /// empty but the old buffer has content — suitable for ED (erase down).
    fn find_ed_row(&self, buffer: &Buffer, width: usize, height: usize) -> usize {
        let mut ed_row = height; // Default: no ED optimization.

        for y in (0..height).rev() {
            let new_row = &buffer[Row(y)];
            let old_row = &self.prev[Row(y)];

            let new_empty = (0..width).all(|x| new_row[x].is_empty());
            let old_has_content = (0..width).any(|x| !old_row[x].is_empty());

            if new_empty && old_has_content {
                ed_row = y;
            } else {
                break;
            }
        }

        ed_row
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Color, Style};

    use crate::buffer::Cell;

    use super::*;


    #[test]
    fn render_styled_cells_emits_sgr() {
        let style = Style::new().bold().foreground(Color::Rgb(255, 0, 0));

        let buffer = Buffer::from_chars(5, 1, &[(0, 0, 'H', style), (0, 1, 'i', style)]);

        let mut r = Rasterizer::new(5, 1);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        assert!(output.contains("\x1B["), "expected SGR sequence: {output}");
        assert!(output.contains('H'), "expected 'H': {output}");
        assert!(output.contains('i'), "expected 'i': {output}");
    }

    #[test]
    fn render_identical_buffer_produces_no_diff() {
        let style = Style::new().foreground(Color::Index(2));
        let buffer = Buffer::from_chars(
            3,
            1,
            &[
                (0, 0, 'A', style),
                (0, 1, 'B', style),
                (0, 2, 'C', style),
            ],
        );

        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer);
        r.clear_output();

        // Render same buffer again.
        r.render(&buffer);

        let output_str = String::from_utf8_lossy(r.output());
        assert!(!output_str.contains('A'), "should not re-emit 'A': {output_str}");
        assert!(!output_str.contains('B'), "should not re-emit 'B': {output_str}");
        assert!(!output_str.contains('C'), "should not re-emit 'C': {output_str}");
    }

    #[test]
    fn render_single_cell_change_emits_only_that_cell() {
        let style = Style::new().foreground(Color::Index(3));
        let buf1 = Buffer::from_chars(
            3,
            1,
            &[
                (0, 0, 'A', style),
                (0, 1, 'B', style),
                (0, 2, 'C', style),
            ],
        );

        let mut r = Rasterizer::new(3, 1);
        r.render(&buf1);
        r.clear_output();

        // Change only middle cell.
        let buf2 = Buffer::from_chars(
            3,
            1,
            &[
                (0, 0, 'A', style),
                (0, 1, 'X', style),
                (0, 2, 'C', style),
            ],
        );

        r.render(&buf2);

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains('X'), "should emit 'X': {output_str}");
        assert!(!output_str.contains('A'), "should not re-emit 'A': {output_str}");
        assert!(!output_str.contains('C'), "should not re-emit 'C': {output_str}");
    }

    #[test]
    fn invalidate_forces_full_redraw() {
        let buffer = Buffer::from_chars(2, 1, &[(0, 0, 'Z', Style::EMPTY)]);

        let mut r = Rasterizer::new(2, 1);
        r.render(&buffer);
        r.clear_output();

        r.invalidate();
        r.render(&buffer);

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains("\x1B[2J"), "should contain ED2: {output_str}");
        assert!(output_str.contains('Z'), "should re-emit 'Z': {output_str}");
    }

    #[test]
    fn trailing_el_optimization() {
        let style = Style::new().foreground(Color::Index(1));

        // First frame: full row of content.
        let buf1 = Buffer::from_chars(
            5,
            1,
            &[
                (0, 0, 'A', style),
                (0, 1, 'B', style),
                (0, 2, 'C', style),
                (0, 3, 'D', style),
                (0, 4, 'E', style),
            ],
        );

        let mut r = Rasterizer::new(5, 1);
        r.render(&buf1);
        r.clear_output();

        // Second frame: change cell 1 and clear cells 2-4.
        // Cell 0 stays the same, cell 1 changes (triggers diff), cells 2-4
        // are empty while old had content → trailing EL.
        let buf2 = Buffer::from_chars(5, 1, &[(0, 0, 'A', style), (0, 1, 'X', style)]);

        r.render(&buf2);

        let output_str = String::from_utf8_lossy(r.output());
        // The diff region is [1..1] (only cell 1 changed: B→X).
        // Trailing cells [2..5]: new=empty, old=content → EL should fire.
        assert!(output_str.contains("\x1B[K"), "should contain EL: {output_str}");
    }

    #[test]
    fn ed_optimization_clears_empty_bottom() {
        let style = Style::new().foreground(Color::Index(1));

        // First frame: content on all 3 rows.
        let buf1 = Buffer::from_chars(
            3,
            3,
            &[
                (0, 0, 'A', style),
                (1, 0, 'B', style),
                (2, 0, 'C', style),
            ],
        );

        let mut r = Rasterizer::new(3, 3);
        r.render(&buf1);
        r.clear_output();

        // Second frame: only first row has content.
        let buf2 = Buffer::from_chars(3, 3, &[(0, 0, 'A', style)]);

        r.render(&buf2);

        let output_str = String::from_utf8_lossy(r.output());
        // Should use ED to clear from row 1 downward.
        assert!(output_str.contains("\x1B[J"), "should contain ED: {output_str}");
    }

    // ── Synchronized Output ──────────────────────────────────────────

    #[test]
    fn sync_output_wraps_render() {
        let caps = Capabilities::DEFAULT | Capabilities::SYNC_OUTPUT;
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::with_capabilities(3, 1, caps);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        assert!(output.starts_with("\x1B[?2026h"), "should start with begin_sync: {output}");
        assert!(output.ends_with("\x1B[?2026l"), "should end with end_sync: {output}");
    }

    #[test]
    fn no_sync_without_cap() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        assert!(!output.contains("\x1B[?2026h"), "should not contain begin_sync: {output}");
        assert!(!output.contains("\x1B[?2026l"), "should not contain end_sync: {output}");
    }

    // ── Row Hashing ─────────────────────────────────────────────────────

    #[test]
    fn row_hash_skips_identical_frame() {
        let style = Style::new().foreground(Color::Index(2));
        let buffer = Buffer::from_chars(
            3,
            2,
            &[
                (0, 0, 'A', style),
                (0, 1, 'B', style),
                (1, 0, 'C', style),
            ],
        );

        let mut r = Rasterizer::new(3, 2);
        r.render(&buffer);
        r.clear_output();

        // Second render — identical buffer.
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        // Only hide/show cursor, no cell content.
        assert!(!output.contains('A'), "should skip row 0: {output}");
        assert!(!output.contains('C'), "should skip row 1: {output}");
    }

    #[test]
    fn row_hash_only_emits_changed_row() {
        let style = Style::new().foreground(Color::Index(3));
        let buf1 = Buffer::from_chars(
            3,
            3,
            &[
                (0, 0, 'a', style),
                (1, 0, 'b', style),
                (2, 0, 'c', style),
            ],
        );

        let mut r = Rasterizer::new(3, 3);
        r.render(&buf1);
        r.clear_output();

        // Change only row 1.
        let buf2 = Buffer::from_chars(
            3,
            3,
            &[
                (0, 0, 'a', style),
                (1, 0, 'X', style),
                (2, 0, 'c', style),
            ],
        );

        r.render(&buf2);

        let output = String::from_utf8_lossy(r.output());
        assert!(output.contains('X'), "should emit changed cell: {output}");
        // Use lowercase to avoid matching escape sequences like \x1B[A.
        assert!(!output.contains('a'), "should skip unchanged row 0: {output}");
        assert!(!output.contains('c'), "should skip unchanged row 2: {output}");
    }

    // ── Scroll Optimization ──────────────────────────────────────────────

    #[test]
    fn scroll_up_detected() {
        let caps = Capabilities::DEFAULT | Capabilities::SCROLL_REGION | Capabilities::SCROLL;
        let style = Style::EMPTY;

        // Frame 1: rows A, B, C, D, E
        let buf1 = Buffer::from_chars(
            3, 5,
            &[
                (0, 0, 'a', style),
                (1, 0, 'b', style),
                (2, 0, 'c', style),
                (3, 0, 'd', style),
                (4, 0, 'e', style),
            ],
        );

        let mut r = Rasterizer::with_capabilities(3, 5, caps);
        r.render(&buf1);
        r.clear_output();

        // Frame 2: rows B, C, D, E, F (scroll up by 1)
        let buf2 = Buffer::from_chars(
            3, 5,
            &[
                (0, 0, 'b', style),
                (1, 0, 'c', style),
                (2, 0, 'd', style),
                (3, 0, 'e', style),
                (4, 0, 'f', style),
            ],
        );

        r.render(&buf2);

        let output = String::from_utf8_lossy(r.output());
        // Should contain SU (scroll up) sequence: \x1B[S
        assert!(output.contains("\x1B[S") || output.contains("\x1B[1S"),
            "should contain SU: {output}");
    }

    #[test]
    fn scroll_down_detected() {
        let caps = Capabilities::DEFAULT | Capabilities::SCROLL_REGION | Capabilities::SCROLL;
        let style = Style::EMPTY;

        // Frame 1: rows A, B, C, D, E
        let buf1 = Buffer::from_chars(
            3, 5,
            &[
                (0, 0, 'a', style),
                (1, 0, 'b', style),
                (2, 0, 'c', style),
                (3, 0, 'd', style),
                (4, 0, 'e', style),
            ],
        );

        let mut r = Rasterizer::with_capabilities(3, 5, caps);
        r.render(&buf1);
        r.clear_output();

        // Frame 2: rows F, A, B, C, D (scroll down by 1)
        let buf2 = Buffer::from_chars(
            3, 5,
            &[
                (0, 0, 'f', style),
                (1, 0, 'a', style),
                (2, 0, 'b', style),
                (3, 0, 'c', style),
                (4, 0, 'd', style),
            ],
        );

        r.render(&buf2);

        let output = String::from_utf8_lossy(r.output());
        // Should contain SD (scroll down) sequence: \x1B[T
        assert!(output.contains("\x1B[T") || output.contains("\x1B[1T"),
            "should contain SD: {output}");
    }

    #[test]
    fn no_scroll_for_completely_different_content() {
        let caps = Capabilities::DEFAULT | Capabilities::SCROLL_REGION | Capabilities::SCROLL;
        let style = Style::EMPTY;

        let buf1 = Buffer::from_chars(3, 5, &[
            (0, 0, 'a', style), (1, 0, 'b', style), (2, 0, 'c', style),
            (3, 0, 'd', style), (4, 0, 'e', style),
        ]);

        let mut r = Rasterizer::with_capabilities(3, 5, caps);
        r.render(&buf1);
        r.clear_output();

        // Completely different content.
        let buf2 = Buffer::from_chars(3, 5, &[
            (0, 0, 'v', style), (1, 0, 'w', style), (2, 0, 'x', style),
            (3, 0, 'y', style), (4, 0, 'z', style),
        ]);

        r.render(&buf2);

        let output = String::from_utf8_lossy(r.output());
        // No scroll should be detected.
        assert!(!output.contains("\x1B[S"), "should not contain SU: {output}");
        assert!(!output.contains("\x1B[T"), "should not contain SD: {output}");
    }

    #[test]
    fn no_scroll_without_caps() {
        let style = Style::EMPTY;

        let buf1 = Buffer::from_chars(3, 5, &[
            (0, 0, 'a', style), (1, 0, 'b', style), (2, 0, 'c', style),
            (3, 0, 'd', style), (4, 0, 'e', style),
        ]);

        // Default caps don't include SCROLL_REGION.
        let mut r = Rasterizer::new(3, 5);
        r.render(&buf1);
        r.clear_output();

        let buf2 = Buffer::from_chars(3, 5, &[
            (0, 0, 'b', style), (1, 0, 'c', style), (2, 0, 'd', style),
            (3, 0, 'e', style), (4, 0, 'f', style),
        ]);

        r.render(&buf2);

        let output = String::from_utf8_lossy(r.output());
        assert!(!output.contains("\x1B[S"), "should not use scroll without caps: {output}");
    }

    // ── REP Optimization ────────────────────────────────────────────────

    #[test]
    fn rep_emits_for_long_runs() {
        let caps = Capabilities::DEFAULT | Capabilities::REP;
        let style = Style::EMPTY;

        // 20 identical 'X' cells.
        let cells: Vec<_> = (0..20).map(|col| (0usize, col, 'X', style)).collect();
        let buffer = Buffer::from_chars(20, 1, &cells);

        let mut r = Rasterizer::with_capabilities(20, 1, caps);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        // Should contain a REP sequence (\x1B[...b).
        assert!(output.contains('b'), "should contain REP sequence: {output}");
    }

    #[test]
    fn rep_not_emitted_for_short_runs() {
        let caps = Capabilities::DEFAULT | Capabilities::REP;
        let style = Style::EMPTY;

        // Only 3 identical cells — below threshold.
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'X', style), (0, 1, 'X', style), (0, 2, 'X', style)]);

        let mut r = Rasterizer::with_capabilities(3, 1, caps);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        // REP needs 4+ repeats, so no REP here.
        let rep_count = output.matches("\x1B[").count();
        // The output should not contain any REP 'b' sequences.
        assert!(!output.contains("b\x1B"), "should not contain REP for short runs: {output}");
    }

    #[test]
    fn rep_not_emitted_without_cap() {
        let style = Style::EMPTY;
        let cells: Vec<_> = (0..20).map(|col| (0usize, col, 'X', style)).collect();
        let buffer = Buffer::from_chars(20, 1, &cells);

        let mut r = Rasterizer::new(20, 1);
        r.render(&buffer);

        let output = r.output();
        let has_rep = output.windows(2).any(|w| w[1] == b'b' && w[0].is_ascii_digit());
        assert!(!has_rep, "should not use REP without cap: {}",
            String::from_utf8_lossy(output));
    }

    #[test]
    fn rep_not_emitted_for_wide_chars() {
        let caps = Capabilities::DEFAULT | Capabilities::REP;
        let style = Style::EMPTY;

        // Wide character '中' (width 2) repeated — should not trigger REP.
        let buffer = Buffer::from_chars(
            10,
            1,
            &[
                (0, 0, '中', style),
                (0, 2, '中', style),
                (0, 4, '中', style),
                (0, 6, '中', style),
                (0, 8, '中', style),
            ],
        );

        let mut r = Rasterizer::with_capabilities(10, 1, caps);
        r.render(&buffer);

        let output = r.output();
        // REP sequence ends with 'b'. Count how many output bytes are 'b'
        // that follow a digit (part of \x1B[Nb pattern).
        let has_rep = output.windows(2).any(|w| w[1] == b'b' && w[0].is_ascii_digit());
        assert!(!has_rep, "should not use REP for wide chars: {}",
            String::from_utf8_lossy(output));
    }

    // ── Pen Elision ────────────────────────────────────────────────────

    #[test]
    fn pen_elision_no_redundant_sgr() {
        let style = Style::new().foreground(Color::Rgb(0, 255, 0));

        // Two adjacent cells with the same style.
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', style), (0, 1, 'B', style)]);

        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer);

        let output = r.output();
        let output_str = String::from_utf8_lossy(output);

        // Count SGR sequences (excluding reset and hide/show cursor).
        // There should be exactly one SGR for the style, not two.
        let sgr_count = output_str.matches("\x1B[38;2;").count();
        assert_eq!(sgr_count, 1, "should emit SGR only once: {output_str}");
    }

    // ── Inline Mode ────────────────────────────────────────────────────

    #[test]
    fn inline_first_render_no_cup() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(5, 2, &[
            (0, 0, 'h', style), (0, 1, 'i', style),
            (1, 0, 'l', style), (1, 1, 'o', style),
        ]);

        let mut r = Rasterizer::inline(5, 2);
        r.render(&buffer);

        let output = r.output();
        // Check for CUP sequences (\x1B[row;colH) — look for `;` followed
        // eventually by `H` within an escape sequence. Exclude \x1B[?25h.
        let has_cup = output.windows(2).enumerate().any(|(i, w)| {
            w == b"\x1B[" && {
                let rest = &output[i + 2..];
                // CUP has a `;` before `H`
                rest.iter().position(|&b| b == b'H').map_or(false, |h_pos| {
                    rest[..h_pos].contains(&b';') && rest[..h_pos].iter().all(|b| b.is_ascii_digit() || *b == b';')
                })
            }
        });
        let output_str = String::from_utf8_lossy(output);
        assert!(!has_cup, "should not contain CUP: {output_str}");

        // Should contain the content.
        assert!(output_str.contains('h'), "should contain 'h': {output_str}");
        assert!(output_str.contains('l'), "should contain 'l': {output_str}");
    }

    #[test]
    fn inline_second_render_starts_with_cuu() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(5, 3, &[
            (0, 0, 'a', style),
            (1, 0, 'b', style),
            (2, 0, 'c', style),
        ]);

        let mut r = Rasterizer::inline(5, 3);
        r.render(&buffer);
        r.clear_output();

        // Change middle row.
        let buf2 = Buffer::from_chars(5, 3, &[
            (0, 0, 'a', style),
            (1, 0, 'X', style),
            (2, 0, 'c', style),
        ]);

        r.render(&buf2);

        let output = String::from_utf8_lossy(r.output());
        // Should start with hide_cursor then CUU (to move back up to our owned region).
        // Hide cursor is \x1B[?25l, then CUU is \x1B[nA.
        assert!(output.contains("\x1B[") && output.contains('A'),
            "should contain CUU: {output}");
    }

    #[test]
    fn inline_no_alt_screen_sequences() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'z', style)]);

        let mut r = Rasterizer::inline(3, 1);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        assert!(!output.contains("\x1B[?1049h"), "should not enter alt screen: {output}");
        assert!(!output.contains("\x1B[?1049l"), "should not exit alt screen: {output}");
    }

    #[test]
    fn inline_no_ed_on_first_render() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(3, 2, &[
            (0, 0, 'x', style),
            (1, 0, 'y', style),
        ]);

        let mut r = Rasterizer::inline(3, 2);
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.output());
        // Inline first render should not clear screen.
        assert!(!output.contains("\x1B[2J"), "should not contain ED2: {output}");
        assert!(!output.contains("\x1B[H"), "should not contain home: {output}");
    }
}
