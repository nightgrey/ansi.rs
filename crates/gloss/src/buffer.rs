use ansi::io::Write as _;
use std::fmt::Debug;
use std::io::{Cursor, Write};
use derive_more::{AsMut, AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};
use ansi::{Escape, Reset, SelectGraphicRendition, Style};
use grid::{Grid};
use super::{Cell, GraphemeArena};

#[derive(Clone, Index, IndexMut, Deref, DerefMut, AsRef, AsMut, IntoIterator)]
#[as_ref(forward)]
#[as_mut(forward)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Buffer(Grid<Cell>);

impl Buffer {
    pub const EMPTY: Self = Self(Grid::EMPTY);

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self(Grid::new(width, height))
    }

    /// Create a buffer from a slice of fixed elements.
    ///
    /// A convenience constructor mostly used for tests.
    pub fn from_chars(width: usize, height: usize, chars: &[(usize, usize, char, Style)]) -> Self {
        let mut buffer = Self::new(width, height);
        for &(row, col, ch, style) in chars {
            buffer[(row, col)] = Cell::from_char(ch, style);
        }
        buffer
    }

    pub fn copy_from_area(&mut self, area: &Area) -> Self {
        Self(self.0.copy_from_area(area))
    }

    /// Insert `n` lines at row `y`, shifting remaining lines down (ANSI IL).
    /// Operates on the full buffer width.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.insert_line_area(y, n, cell, self.area());
    }

    /// Delete `n` lines at row `y`, shifting remaining lines up (ANSI DL).
    /// Operates on the full buffer width.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.delete_line_area(y, n, cell, &self.area());
    }

    /// Insert `n` cells at `(x, y)`, shifting cells right (ANSI ICH).
    /// Operates on the full buffer width.
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.insert_cell_area(row, col, n, cell, &self.area());
    }

    /// Delete `n` cells at `(x, y)`, shifting cells left (ANSI DCH).
    /// Operates on the full buffer width.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.delete_cell_area(row, col, n, cell, &self.area());
    }

    /// Insert `n` lines at row `y` within specific bounds.
    /// Lines at `y` and below are shifted down; lines pushed beyond `bounds.max.row` are lost.
    /// New lines are filled with `cell`.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: impl Spatial) {
        if n == 0 {
            return;
        }

        // Clip to buffer bounds and ensure y is within bounds
        let bounds = self.clip(&bounds);
        let y = y.clamp(bounds.min.row, bounds.max.row);
        let n = n.min(bounds.max.row - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.row {
            return;
        }


        // Move lines down (backwards to prevent overwriting)
        // Source: [y, max-n) -> Dest: [y+n, max)
        for row in (y..(bounds.max.row - n)).rev() {
            let src_start = row * self.width + bounds.min.col;
            let dst_start = (row + n) * self.width + bounds.min.col;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Fill new lines with the provided cell
        for row in y..(y + n) {
            let start = row * self.width + bounds.min.col;
            self[start..start + width].fill(cell);
        }
    }

    /// Delete `n` lines at row `y` within specific bounds.
    /// Lines below shift up; new blank lines appear at bottom of bounds.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: &impl Spatial) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(bounds);
        let y = y.clamp(bounds.min.row, bounds.max.row);
        let n = n.min(bounds.max.row - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.row {
            return;
        }

        let row_stride = self.width;

        // Move lines up
        // Source: [y+n, max) -> Dest: [y, max-n)
        for row in y..(bounds.max.row - n) {
            let src_start = (row + n) * row_stride + bounds.min.col;
            let dst_start = row * row_stride + bounds.min.col;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Clear bottom n lines
        for row in (bounds.max.row - n)..bounds.max.row {
            let start = row * row_stride + bounds.min.col;
            self[start..start + width].fill(cell);
        }
    }

    /// Insert `n` cells at `(x, y)` within specific bounds (ANSI ICH).
    /// Cells shift right; cells pushed beyond right margin are lost.
    pub fn insert_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: &impl Spatial) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(bounds);

        // Validate y is within vertical bounds
        if row < bounds.min.row || row >= bounds.max.row {
            return;
        }

        let x = col.clamp(bounds.min.col, bounds.max.col);
        let n = n.min(bounds.max.col - x);

        if n == 0 {
            return;
        }

        let row_offset = row * self.width;

        // Shift cells right: [x, max-n) -> [x+n, max)
        if x + n < bounds.max.col {
            let src_start = row_offset + x;
            let src_end = row_offset + bounds.max.col - n;
            let dst_start = row_offset + x + n;
            self.copy_within(src_start..src_end, dst_start);
        }

        // Fill insertion point
        let fill_start = row_offset + x;
        let fill_end = fill_start + n;
        self[fill_start..fill_end].fill(cell);
    }

    /// Delete `n` cells at `(x, y)` within specific bounds (ANSI DCH).
    /// Cells shift left; new blank cells appear at right margin.
    pub fn delete_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: &impl Spatial) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(bounds);

        if row < bounds.min.row || row >= bounds.max.row {
            return;
        }

        let x = col.clamp(bounds.min.col, bounds.max.col);
        let n = n.min(bounds.max.col - x);

        if n == 0 {
            return;
        }

        let fill_cell = cell;
        let row_offset = row * self.width;

        // Shift cells left: [x+n, max) -> [x, max-n)
        if x + n < bounds.max.col {
            let src_start = row_offset + x + n;
            let src_end = row_offset + bounds.max.col;
            let dst_start = row_offset + x;
            self.copy_within(src_start..src_end, dst_start);
        }

        // Clear rightmost cells
        let clear_start = row_offset + bounds.max.col - n;
        let clear_end = row_offset + bounds.max.col;
        self[clear_start..clear_end].fill(fill_cell);
    }
    pub fn to_string(&self, arena: &GraphemeArena) -> String {
        self.rows().map(|row| row.iter().map(|cell| {
            cell.as_str(arena)
        }).collect::<String>()).intersperse(String::from("\n")).collect()
    }

}
impl Spatial for Buffer {
    fn min(&self) -> Position { Position::ZERO }
    fn max(&self) -> Position { Position::new(self.height(), self.width()) }

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Buffer")
            .field(&self.as_slice())
            .field(&self.size())
            .finish()
    }
}

