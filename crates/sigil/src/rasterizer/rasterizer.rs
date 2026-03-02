use std::borrow::Cow;
use std::io;
use derive_more::{Deref, DerefMut};
use rustix::path::Arg;
use ansi::{Escape, Style};
use geometry::{Position, Row};

use crate::buffer::{Buffer, Cell};

use super::sequences as seq;

#[derive(Clone,  Copy, Debug, Deref, DerefMut)]
struct Cursor {
    #[deref]
    #[deref_mut]
    pos: Position,
    style: Style,
}

impl Cursor {
    pub const ZERO: Self = Self {
        pos: Position::ZERO,
        style: Style::EMPTY,
    };

    pub fn new() -> Self {
        Self::ZERO
    }

    pub fn to(&mut self, row: usize, col: usize) {
        self.pos.row = row;
        self.pos.col = col;
    }

    pub fn reset(&mut self) {
        *self = Self::ZERO;
    }

    pub fn is_unstyled(&self) -> bool {
        self.style.is_empty()
    }

    pub fn clear_style(&mut self) {
        self.style.clear();
    }
}

#[derive(Clone, Debug)]
pub struct Rasterizer {
    buffer: Vec<u8>,
    frame: Buffer,
    cursor: Cursor,
    is_fullscreen: bool,
    clear: bool,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            frame: Buffer::new(width, height),
            cursor: Cursor::new(),
            is_fullscreen: false,
            clear: false,
        }
    }

    /// Render a buffer, diffing against the previous frame.
    pub fn render(&mut self, buffer: &Buffer) {
        let prev = &mut self.frame;
        let next = buffer;

        match self.clear {
            true => {
                seq::home(&mut self.buffer);
                seq::ed_all(&mut self.buffer);
                self.cursor.reset();
                self.clear = false;
                prev.clear_and_resize(next.width, next.height);
            },
            false => if prev.width != next.width && prev.height != next.height {
                prev.resize(next.width, next.height);
            }
        }

        seq::hide_cursor(&mut self.buffer);

        for y in 0..next.height {
            self.render_line(next, y);
        }

        let cursor = &mut self.cursor;

        // Reset pen if dirty.
        if !cursor.is_unstyled() {
            seq::sgr_reset(&mut self.buffer);
            cursor.clear_style();
        }

        seq::show_cursor(&mut self.buffer);

        // Swap prev ← new (clone the next for next diff).
        self.frame = next.clone();
    }

    /// Flush internal buffer to the underlying writer.
    pub fn flush(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        if !self.buffer.is_empty() {
            w.write_all(&self.buffer)?;
            self.buffer.clear();
        }
        w.flush()
    }

    /// Mark the screen for a full clear on next render.
    pub fn redraw(&mut self) {
        self.clear = true;
    }

    /// Enter alternate screen buffer.
    pub fn enter_alt_screen(&mut self) {
        seq::enter_alt_screen(&mut self.buffer);
        self.is_fullscreen = true;
        self.clear = true;
    }

    /// Exit alternate screen buffer.
    pub fn exit_alt_screen(&mut self) {
        seq::sgr_reset(&mut self.buffer);
        seq::exit_alt_screen(&mut self.buffer);
        self.is_fullscreen = false;
        self.frame.clear();
        self.cursor.reset();
    }

    /// Invalidate the previous buffer, forcing a full redraw.
    pub fn resize(&mut self) {
        self.frame.clear();
        self.clear = true;
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub fn clear_frame(&mut self) {
        self.frame.clear();
    }
    pub fn clear_cursor(&mut self) {
        self.cursor.reset();
    }

    pub fn clear(&mut self) {
        self.clear_frame();
        self.clear_buffer();
        self.clear_cursor();
    }

    // ── Internal ────────────────────────────────────────────────────

    /// Diff and render a single line.
    fn render_line(&mut self, buffer: &Buffer, y: usize) {
        let width = buffer.width;

        // Compute diff bounds and trailing-blank state up front, before
        // borrowing self mutably.
        let (first_diff, last_diff, need_eol) = {
            let prev = &self.frame;
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
                seq::sgr_reset(&mut self.buffer);
                self.cursor.style = Style::EMPTY;
            }
            seq::el(&mut self.buffer);
        }
    }

    /// Update the pen (SGR state) to match the target style.
    fn update_pen(&mut self, style: &Style) {
        if self.cursor.style == *style {
            return;
        }

        let diff = self.cursor.style.diff(*style);
        if !diff.is_empty() {
            diff.escape(&mut self.buffer).ok();
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
            seq::cr(&mut self.buffer);
        } else if dr == 0 && dc > 0 && dc <= 4 {
            seq::cuf(&mut self.buffer, dc as usize);
        } else if dr == 0 && dc < 0 && (-dc) <= 4 {
            seq::cub(&mut self.buffer, (-dc) as usize);
        } else if dc == 0 && dr > 0 && dr <= 4 {
            seq::cud(&mut self.buffer, dr as usize);
        } else if dc == 0 && dr < 0 && (-dr) <= 4 {
            seq::cuu(&mut self.buffer, (-dr) as usize);
        } else {
            seq::cup(&mut self.buffer, row, col);
        }

        self.cursor.to(row, col);
    }

    /// Write a cell's content after updating the pen.
    fn put_cell(&mut self, buffer: &Buffer, cell: &Cell) {
        self.update_pen(cell.style());

        if cell.is_empty() {
            self.buffer.push(b' ');
        } else {
            let s = cell.as_str(&buffer.arena);
            self.buffer.extend_from_slice(s.as_bytes());
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    fn as_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.buffer)
    }
}
#[cfg(test)]
mod tests {
    use ansi::{Attribute, Color};

    use super::*;

    #[test]
    fn render_styled_cells_emits_sgr() {
        let mut output = Vec::new();
        let style = Style::new()
            .bold()
            .foreground(Color::Rgb(255, 0, 0));
        let buffer = Buffer::from_chars(5, 1, &[
            (0, 0, 'H', style),
            (0, 1, 'i', style),
        ]);

        let mut rasterizer = Rasterizer::new(5, 1);
        rasterizer.render(&buffer);
        rasterizer.flush(&mut output).unwrap();

        let str = String::from_utf8_lossy(&output);
        assert!(str.contains("\x1B["), "expected SGR sequence in output: {str}");
        assert!(str.contains('H'), "expected 'H' in output: {str}");
        assert!(str.contains('i'), "expected 'i' in output: {str}");
    }

    #[test]
    fn render_identical_buffer_produces_no_diff() {
        let mut output = io::Cursor::new(Vec::default());
        let style = Style::new().foreground(Color::Index(2));
        let buffer = Buffer::from_chars(3, 1, &[
            (0, 0, 'A', style),
            (0, 1, 'B', style),
            (0, 2, 'C', style),
        ]);

        let mut rasterizer = Rasterizer::new(3, 1);
        rasterizer.render(&buffer);
        rasterizer.flush(&mut output).unwrap();

        let str = String::from_utf8_lossy(&output.get_ref());
        assert!(str.contains('A'), "should emit 'A': {str}");
        assert!(str.contains('B'), "should emit 'B': {str}");
        assert!(str.contains('C'), "should emit 'C': {str}");

        // Render same buffer again.
        rasterizer.render(&buffer);
        let mut next = io::Cursor::new(Vec::default());
        rasterizer.flush(&mut next).unwrap();

        // Output should only contain hide/show cursor, reset — no cell data.
        let str = String::from_utf8_lossy(&next.get_ref());
        assert!(!str.contains('A'), "should not re-emit 'A': {str}");
        assert!(!str.contains('B'), "should not re-emit 'B': {str}");
        assert!(!str.contains('C'), "should not re-emit 'C': {str}");
    }

    #[test]
    fn render_single_cell_change_emits_only_that_cell() {
        let mut output = io::Cursor::new(Vec::default());

        let style = Style::new().foreground(Color::Index(3));
        let mut buffer = Buffer::from_chars(3, 1, &[
            (0, 0, 'A', style),
            (0, 1, 'B', style),
            (0, 2, 'C', style),
        ]);

        let mut rasterizer = Rasterizer::new(3, 1);
        rasterizer.render(&buffer);
        rasterizer.flush(&mut output).unwrap();

        let str = String::from_utf8_lossy(&output.get_ref());
        assert!(str.contains('A'), "should emit 'A': {str}");
        assert!(str.contains('B'), "should emit 'B': {str}");
        assert!(str.contains('C'), "should emit 'C': {str}");

        // Change only middle cell.
        buffer[(0, 1)] = Cell::from_char('X', style);

        rasterizer.render(&buffer);


        let mut next = io::Cursor::new(Vec::default());
        rasterizer.flush(&mut next).unwrap();

        let str = String::from_utf8_lossy(&next.get_ref());
        // Should contain the new cell value.
        assert!(str.contains('X'), "should emit changed cell 'X': {str}");
        // Should not contain unchanged cells.
        assert!(!str.contains('A'), "should not re-emit 'A': {str}");
        assert!(!str.contains('C'), "should not re-emit 'C': {str}");
    }

    #[test]
    fn redraw() {
        let mut output = io::Cursor::new(Vec::default());
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(2, 1, &[(0, 0, 'Z', style)]);
        let mut rasterizer = Rasterizer::new(2, 1);

        rasterizer.render(&buffer);
        rasterizer.flush(&mut output).unwrap();

        rasterizer.redraw();

        rasterizer.render(&buffer);
        rasterizer.flush(&mut output).unwrap();

        let str = String::from_utf8_lossy(&output.get_ref());
        dbg!(&str);
        // Should contain clear screen sequence and the cell.
        assert!(str.contains("\x1B[2J"), "should contain ED2: {str}");
        assert!(str.contains('Z'), "should re-emit 'Z': {str}");
    }
}
