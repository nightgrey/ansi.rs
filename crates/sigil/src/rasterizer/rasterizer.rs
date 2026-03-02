use std::io;

use ansi::{Escape, Style};
use geometry::Row;

use crate::buffer::{Buffer, Cell};

use super::cursor::Cursor;
use super::sequences as seq;

pub struct Rasterizer<W: io::Write> {
    w: W,
    terminal: Vec<u8>,
    prev: Option<Buffer>,
    cursor: Cursor,
    is_fullscreen: bool,
    force_clear: bool,
}

impl<W: io::Write> Rasterizer<W> {
    /// Create a new renderer writing to `writer`.
    pub fn new(writer: W) -> Self {
        Self {
            w: writer,
            terminal: Vec::with_capacity(4096),
            prev: None,
            cursor: Cursor::new(),
            is_fullscreen: false,
            force_clear: false,
        }
    }

    /// Render a buffer, diffing against the previous frame.
    pub fn render(&mut self, buffer: &Buffer) {
        let width = buffer.width;
        let height = buffer.height;

        // Allocate or reallocate prev buffer if dimensions changed.
        let need_full = match &self.prev {
            Some(prev) if prev.width == width && prev.height == height && !self.force_clear => false,
            _ => true,
        };

        if need_full {
            if self.force_clear {
                seq::home(&mut self.terminal);
                seq::ed_all(&mut self.terminal);
                self.cursor.reset();
                self.force_clear = false;
            }
            self.prev = Some(Buffer::new(width, height));
        }

        seq::hide_cursor(&mut self.terminal);

        for y in 0..height {
            self.render_line(buffer, y);
        }

        // Reset pen if dirty.
        if !self.cursor.style.is_empty() {
            seq::sgr_reset(&mut self.terminal);
            self.cursor.style = Style::EMPTY;
        }

        seq::show_cursor(&mut self.terminal);

        // Swap prev ← new (clone the buffer for next diff).
        self.prev = Some(buffer.clone());
    }

    /// Flush internal buffer to the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        if !self.terminal.is_empty() {
            self.w.write_all(&self.terminal)?;
            self.terminal.clear();
        }
        self.w.flush()
    }

    /// Mark the screen for a full clear on next render.
    pub fn erase(&mut self) {
        self.force_clear = true;
    }

    /// Enter alternate screen buffer.
    pub fn enter_alt_screen(&mut self) {
        seq::enter_alt_screen(&mut self.terminal);
        self.is_fullscreen = true;
        self.force_clear = true;
    }

    /// Exit alternate screen buffer.
    pub fn exit_alt_screen(&mut self) {
        seq::sgr_reset(&mut self.terminal);
        seq::exit_alt_screen(&mut self.terminal);
        self.is_fullscreen = false;
        self.prev = None;
        self.cursor.reset();
    }

    /// Move cursor to absolute position.
    pub fn move_to(&mut self, row: usize, col: usize) {
        self.move_cursor(row, col);
    }

    /// Invalidate the previous buffer, forcing a full redraw.
    pub fn resize(&mut self) {
        self.prev = None;
        self.force_clear = true;
    }

    /// Access the internal write buffer (for testing).
    pub fn buffer(&self) -> &[u8] {
        &self.terminal
    }

    // ── Internal ────────────────────────────────────────────────────

    /// Diff and render a single line.
    fn render_line(&mut self, buffer: &Buffer, y: usize) {
        let width = buffer.width;

        // Compute diff bounds and trailing-blank state up front, before
        // borrowing self mutably.
        let (first_diff, last_diff, need_eol) = {
            let prev = self.prev.as_ref().expect("prev buffer must exist");
            let new_row: &[Cell] = &buffer[Row(y)];
            let old_row: &[Cell] = &prev[Row(y)];

            let first = match (0..width).find(|&x| new_row[x] != old_row[x]) {
                Some(col) => col,
                None => return,
            };
            let last = (0..width)
                .rev()
                .find(|&x| new_row[x] != old_row[x])
                .unwrap();

            let new_trail_blank = (last + 1..width).all(|x| new_row[x].is_empty());
            let old_trail_dirty = (last + 1..width).any(|x| !old_row[x].is_empty());

            (first, last, new_trail_blank && old_trail_dirty)
        };

        // Move cursor to start of changed region.
        self.move_cursor(y, first_diff);

        // Emit changed cells.
        let mut col = first_diff;
        while col <= last_diff {
            let cell = buffer[Row(y)][col];
            self.put_cell(buffer, &cell);
            let w = cell.columns() as usize;
            col += w;
            self.cursor.col += w;
        }

        // Clear to end of line if new line has trailing blanks where old didn't.
        if need_eol {
            if !self.cursor.style.is_empty() {
                seq::sgr_reset(&mut self.terminal);
                self.cursor.style = Style::EMPTY;
            }
            seq::el(&mut self.terminal);
        }
    }

    /// Update the pen (SGR state) to match the target style.
    fn update_pen(&mut self, style: &Style) {
        if self.cursor.style == *style {
            return;
        }

        let diff = self.cursor.style.diff(*style);
        if !diff.is_empty() {
            diff.escape(&mut self.terminal).ok();
        }

        self.cursor.style = *style;
    }

    /// Emit the shortest cursor movement sequence.
    fn move_cursor(&mut self, row: usize, col: usize) {
        if self.cursor.row == row && self.cursor.col == col {
            return;
        }

        let dr = row as isize - self.cursor.row as isize;
        let dc = col as isize - self.cursor.col as isize;

        // If moving to column 0, CR is cheapest.
        if col == 0 && dr == 0 {
            seq::cr(&mut self.terminal);
        } else if dr == 0 && dc > 0 && dc <= 4 {
            seq::cuf(&mut self.terminal, dc as usize);
        } else if dr == 0 && dc < 0 && (-dc) <= 4 {
            seq::cub(&mut self.terminal, (-dc) as usize);
        } else if dc == 0 && dr > 0 && dr <= 4 {
            seq::cud(&mut self.terminal, dr as usize);
        } else if dc == 0 && dr < 0 && (-dr) <= 4 {
            seq::cuu(&mut self.terminal, (-dr) as usize);
        } else {
            seq::cup(&mut self.terminal, row, col);
        }

        self.cursor.move_to(row, col);
    }

    /// Write a cell's content after updating the pen.
    fn put_cell(&mut self, buffer: &Buffer, cell: &Cell) {
        self.update_pen(cell.style());

        if cell.is_empty() {
            self.terminal.push(b' ');
        } else {
            let s = cell.as_str(&buffer.arena);
            self.terminal.extend_from_slice(s.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Attribute, Color};

    use super::*;

    fn make_buffer(width: usize, height: usize, cells: &[(usize, usize, char, Style)]) -> Buffer {
        let mut buf = Buffer::new(width, height);
        for &(row, col, ch, style) in cells {
            let pos = geometry::Position::new(row, col);
            buf[pos] = Cell::from_char(ch, 1, style);
        }
        buf
    }

    #[test]
    fn render_styled_cells_emits_sgr() {
        let style = Style::new()
            .bold()
            .foreground(Color::Rgb(255, 0, 0));

        let buffer = make_buffer(5, 1, &[
            (0, 0, 'H', style),
            (0, 1, 'i', style),
        ]);

        let mut r = Rasterizer::new(Vec::new());
        r.render(&buffer);

        let output = String::from_utf8_lossy(r.buffer());

        // Should contain SGR for bold + red fg.
        assert!(output.contains("\x1B["), "expected SGR sequence in output: {output}");
        // Should contain the text.
        assert!(output.contains('H'), "expected 'H' in output: {output}");
        assert!(output.contains('i'), "expected 'i' in output: {output}");
    }

    #[test]
    fn render_identical_buffer_produces_no_diff() {
        let style = Style::new().foreground(Color::Index(2));
        let buffer = make_buffer(3, 1, &[
            (0, 0, 'A', style),
            (0, 1, 'B', style),
            (0, 2, 'C', style),
        ]);

        let mut r = Rasterizer::new(Vec::new());
        r.render(&buffer);
        r.terminal.clear(); // Clear first render output.

        // Render same buffer again.
        r.render(&buffer);

        let output = &r.terminal;
        // Output should only contain hide/show cursor, reset — no cell data.
        let output_str = String::from_utf8_lossy(output);
        assert!(!output_str.contains('A'), "should not re-emit 'A': {output_str}");
        assert!(!output_str.contains('B'), "should not re-emit 'B': {output_str}");
        assert!(!output_str.contains('C'), "should not re-emit 'C': {output_str}");
    }

    #[test]
    fn render_single_cell_change_emits_only_that_cell() {
        let style = Style::new().foreground(Color::Index(3));
        let buf1 = make_buffer(3, 1, &[
            (0, 0, 'A', style),
            (0, 1, 'B', style),
            (0, 2, 'C', style),
        ]);

        let mut r = Rasterizer::new(Vec::new());
        r.render(&buf1);
        r.terminal.clear();

        // Change only middle cell.
        let buf2 = make_buffer(3, 1, &[
            (0, 0, 'A', style),
            (0, 1, 'X', style),
            (0, 2, 'C', style),
        ]);

        r.render(&buf2);

        let output_str = String::from_utf8_lossy(&r.terminal);
        // Should contain the new cell value.
        assert!(output_str.contains('X'), "should emit changed cell 'X': {output_str}");
        // Should not contain unchanged cells.
        assert!(!output_str.contains('A'), "should not re-emit 'A': {output_str}");
        assert!(!output_str.contains('C'), "should not re-emit 'C': {output_str}");
    }

    #[test]
    fn erase_forces_full_redraw() {
        let style = Style::EMPTY;
        let buffer = make_buffer(2, 1, &[(0, 0, 'Z', style)]);

        let mut r = Rasterizer::new(Vec::new());
        r.render(&buffer);
        r.terminal.clear();

        r.erase();
        r.render(&buffer);

        let output_str = String::from_utf8_lossy(&r.terminal);
        // Should contain clear screen sequence and the cell.
        assert!(output_str.contains("\x1B[2J"), "should contain ED2: {output_str}");
        assert!(output_str.contains('Z'), "should re-emit 'Z': {output_str}");
    }
}
