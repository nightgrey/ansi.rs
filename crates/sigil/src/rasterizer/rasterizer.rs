use std::io;

use geometry::Row;

use crate::buffer::Buffer;

use super::capabilities::Capabilities;
use super::cursor::Cursor;
use super::line::transform_line;
use super::sequences as seq;

pub struct Rasterizer {
    output: Vec<u8>,
    prev: Buffer,
    cursor: Cursor,
    caps: Capabilities,
    is_fullscreen: bool,
    force_clear: bool,
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
        }
    }

    /// Create a rasterizer with explicit capabilities.
    pub fn with_capabilities(width: usize, height: usize, caps: Capabilities) -> Self {
        Self {
            caps,
            ..Self::new(width, height)
        }
    }

    /// Render a buffer, diffing against the previous frame.
    pub fn render(&mut self, buffer: &Buffer) {
        let width = buffer.width;
        let height = buffer.height;

        // Handle dimension change or forced clear.
        if self.prev.width != width || self.prev.height != height || self.force_clear {
            seq::home(&mut self.output);
            seq::ed_all(&mut self.output);
            self.cursor.reset();
            self.prev = Buffer::new(width, height);
            self.force_clear = false;
        }

        seq::hide_cursor(&mut self.output);

        // Clear-to-bottom optimization: scan from bottom upward for contiguous
        // all-empty new lines where old has content.
        let ed_row = self.find_ed_row(buffer, width, height);

        for y in 0..height {
            if y >= ed_row {
                // Everything from ed_row down can be erased in one shot.
                self.cursor.move_to(&mut self.output, ed_row, 0, self.caps);
                self.cursor.reset_pen(&mut self.output);
                seq::ed(&mut self.output);
                break;
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

        self.cursor.reset_pen(&mut self.output);
        seq::show_cursor(&mut self.output);

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
        self.force_clear = true;
    }

    /// Enter alternate screen buffer.
    pub fn enter_alt_screen(&mut self) {
        seq::enter_alt_screen(&mut self.output);
        self.is_fullscreen = true;
        self.force_clear = true;
    }

    /// Exit alternate screen buffer.
    pub fn exit_alt_screen(&mut self) {
        seq::sgr_reset(&mut self.output);
        seq::exit_alt_screen(&mut self.output);
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

    // ── Internal ────────────────────────────────────────────────────

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
    use geometry::Position;

    use crate::buffer::Cell;

    use super::*;

    fn make_buffer(width: usize, height: usize, cells: &[(usize, usize, char, Style)]) -> Buffer {
        let mut buf = Buffer::new(width, height);
        for &(row, col, ch, style) in cells {
            buf[(row, col)] = Cell::from_char(ch, style);
        }
        buf
    }

    #[test]
    fn render_styled_cells_emits_sgr() {
        let style = Style::new().bold().foreground(Color::Rgb(255, 0, 0));

        let buffer = make_buffer(5, 1, &[(0, 0, 'H', style), (0, 1, 'i', style)]);

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
        let buffer = make_buffer(
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
        let buf1 = make_buffer(
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
        let buf2 = make_buffer(
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
        let buffer = make_buffer(2, 1, &[(0, 0, 'Z', Style::EMPTY)]);

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
        let buf1 = make_buffer(
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
        let buf2 = make_buffer(5, 1, &[(0, 0, 'A', style), (0, 1, 'X', style)]);

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
        let buf1 = make_buffer(
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
        let buf2 = make_buffer(3, 3, &[(0, 0, 'A', style)]);

        r.render(&buf2);

        let output_str = String::from_utf8_lossy(r.output());
        // Should use ED to clear from row 1 downward.
        assert!(output_str.contains("\x1B[J"), "should contain ED: {output_str}");
    }

    #[test]
    fn pen_elision_no_redundant_sgr() {
        let style = Style::new().foreground(Color::Rgb(0, 255, 0));

        // Two adjacent cells with the same style.
        let buffer = make_buffer(3, 1, &[(0, 0, 'A', style), (0, 1, 'B', style)]);

        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer);

        let output = r.output();
        let output_str = String::from_utf8_lossy(output);

        // Count SGR sequences (excluding reset and hide/show cursor).
        // There should be exactly one SGR for the style, not two.
        let sgr_count = output_str.matches("\x1B[38;2;").count();
        assert_eq!(sgr_count, 1, "should emit SGR only once: {output_str}");
    }
}
