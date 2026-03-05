use std::fmt::Debug;
use derive_more::{AsMut, AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};
use ansi::Style;
use geometry::{Grid, Position, Bounds, Context, };
use super::{Cell, GraphemeArena};

#[derive(Clone, Index, IndexMut, Deref, DerefMut, AsRef, AsMut, IntoIterator)]
pub struct Buffer {
    #[index]
    #[index_mut]
    #[deref]
    #[deref_mut]
    #[as_ref(forward)]
    #[as_mut(forward)]
    #[into_iterator(owned, ref, ref_mut)]
    inner: Grid<Cell>,
    pub arena: GraphemeArena,
}

impl Buffer {
    pub const EMPTY: Self = Self {
        inner: Grid::EMPTY,
        arena: GraphemeArena::EMPTY,
    };

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: Grid::new(width, height),
            arena: GraphemeArena::new(),
        }
    }

    pub fn with_arena(width: usize, height: usize, arena: GraphemeArena) -> Self {
        Self {
            inner: Grid::new(width, height),
            arena,
        }
    }

    /// Create a buffer from a slice of fixed elements.
    ///
    /// A convenience constructor mostly used for tests.
    pub(crate) fn from_chars(width: usize, height: usize, chars: &[(usize, usize, char, Style)]) -> Self {
        let mut buffer = Self::new(width, height);
        for &(row, col, ch, style) in chars {
            buffer[(row, col)] = Cell::from_char(ch, style);
        }
        buffer
    }


    pub fn clone_from_region(&mut self, bounds: Bounds) -> Self {
        Self {
            inner: self.inner.clone_from_region(bounds),
            arena: self.arena.clone(),
        }
    }

    /// Insert `n` lines at row `y`, shifting remaining lines down (ANSI IL).
    /// Operates on the full buffer width.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.insert_line_area(y, n, cell, self.bounds());
    }

    /// Delete `n` lines at row `y`, shifting remaining lines up (ANSI DL).
    /// Operates on the full buffer width.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.delete_line_area(y, n, cell, self.bounds());
    }

    /// Insert `n` cells at `(x, y)`, shifting cells right (ANSI ICH).
    /// Operates on the full buffer width.
    pub fn insert_cell(&mut self, x: usize, y: usize, n: usize, cell: Cell) {
        self.insert_cell_area(x, y, n, cell, self.bounds());
    }

    /// Delete `n` cells at `(x, y)`, shifting cells left (ANSI DCH).
    /// Operates on the full buffer width.
    pub fn delete_cell(&mut self, x: usize, y: usize, n: usize, cell: Cell) {
        self.delete_cell_area(x, y, n, cell, self.bounds());
    }

    /// Insert `n` lines at row `y` within specific bounds.
    /// Lines at `y` and below are shifted down; lines pushed beyond `bounds.max.row` are lost.
    /// New lines are filled with `cell`.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Bounds) {
        if n == 0 {
            return;
        }

        // Clip to buffer bounds and ensure y is within bounds
        let bounds = self.clip(bounds);
        let y = y.clamp(bounds.min.row, bounds.max.row);
        let n = n.min(bounds.max.row - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.row {
            return;
        }

        let row_stride = self.width;

        // Move lines down (backwards to prevent overwriting)
        // Source: [y, max-n) -> Dest: [y+n, max)
        for row in (y..(bounds.max.row - n)).rev() {
            let src_start = row * row_stride + bounds.min.col;
            let dst_start = (row + n) * row_stride + bounds.min.col;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Fill new lines with the provided cell
        for row in y..(y + n) {
            let start = row * row_stride + bounds.min.col;
            self[start..start + width].fill(cell);
        }
    }

    /// Delete `n` lines at row `y` within specific bounds.
    /// Lines below shift up; new blank lines appear at bottom of bounds.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Bounds) {
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
    pub fn insert_cell_area(&mut self, x: usize, y: usize, n: usize, cell: Cell, bounds: Bounds) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(bounds);

        // Validate y is within vertical bounds
        if y < bounds.min.row || y >= bounds.max.row {
            return;
        }

        let x = x.clamp(bounds.min.col, bounds.max.col);
        let n = n.min(bounds.max.col - x);

        if n == 0 {
            return;
        }

        let row_offset = y * self.width;

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
    pub fn delete_cell_area(&mut self, x: usize, y: usize, n: usize, cell: Cell, bounds: Bounds) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(bounds);

        if y < bounds.min.row || y >= bounds.max.row {
            return;
        }

        let x = x.clamp(bounds.min.col, bounds.max.col);
        let n = n.min(bounds.max.col - x);

        if n == 0 {
            return;
        }

        let fill_cell = cell;
        let row_offset = y * self.width;

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

    pub fn clear(&mut self) {
        // Release all extended graphemes.
        for cell in &mut self.inner {
            cell.release(&mut self.arena);
            cell.clear();
        }

        self.arena.clear();
    }


    pub fn to_string(&self) -> String {
        self.iter().map(|cell| cell.as_str(&self.arena)).collect()
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("inner", &self.inner.as_slice())
            .field("arena", &self.arena)
            .finish()
    }
}


impl Context for Buffer {
    fn min(&self) -> Position { Position::ZERO }
    fn max(&self) -> Position { Position::new(self.height(), self.width()) }

    fn x(&self) -> usize { 0 }
    fn y(&self) -> usize { 0 }

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
}
