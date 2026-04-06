use ansi::Style;
use core::slice::IterMut;
use derive_more::{AsMut, AsRef, Deref, DerefMut, IntoIterator};
use geometry::{Bounded, Intersect, Point, Position, Rect, Sides, Zero};
use std::fmt::Debug;
use std::iter::StepBy;
use std::ops::{Index, IndexMut};
use std::slice::Iter;
use std::slice::SliceIndex;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use geometry::Resolve;
use crate::{Cell, Arena, BufferIndex, Buf, BufMut};
#[derive(Deref, DerefMut, AsRef, AsMut, IntoIterator, Clone)]
pub struct Buffer {
    #[deref]
    #[deref_mut]
    #[as_ref(forward)]
    #[as_mut(forward)]
    #[into_iterator(owned, ref, ref_mut)]
    pub(crate) inner: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Buffer {
    pub const EMPTY: Self = Self {
        inner: Vec::new(),
        width: 0,
        height: 0,
    };

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![Cell::EMPTY; width * height],
            width,
            height,
        }
    }

    /// Create a buffer from a slice of fixed elements.
    ///
    /// A convenience constructor mostly used for tests.
    pub fn from_chars(width: usize, height: usize, chars: &[(usize, usize, char, Style)]) -> Self {
        let mut buffer = Self::new(width, height);
        for &(row, col, ch, style) in chars {
            buffer[(col, row)] = Cell::inline(ch, style);
        }
        buffer
    }



    /// Create a buffer from a slice of fixed elements.
    #[must_use]
    pub fn from_lines<'a>(lines: impl IntoIterator<Item = &'a str>, arena: &mut Arena) -> Self
    {
        let lines = lines.into_iter().collect::<Vec<_>>();
        let height = lines.len();
        let width = lines.iter().map(|line| line.width()).max().unwrap_or_default();
        let mut buffer = Self::new(width, height);
        for (y, line) in lines.iter().enumerate() {
            buffer.set_line((0, y), line, arena);
        }
        buffer
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    pub fn get<I: BufferIndex>(
        &self,
        index: I,
    ) -> Option<&<I::Index as SliceIndex<[Cell]>>::Output> {
        index.get(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: BufferIndex>(
        &mut self,
        index: I,
    ) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output> {
        index.get_mut(self)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked<I: BufferIndex>(
        &self,
        index: I,
    ) -> *const <I::Index as SliceIndex<[Cell]>>::Output {
        index.get_unchecked(self)
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<I: BufferIndex>(
        &mut self,
        index: I,
    ) -> *mut <I::Index as SliceIndex<[Cell]>>::Output {
        index.get_unchecked_mut(self)
    }

    /// Print the given string until the end of the given index.
    /// Skips zero-width graphemes and control characters.
    pub fn set_string(&mut self, index: impl BufferIndex<Output = [Cell]>, string: impl AsRef<str>, arena: &mut Arena) -> Option<usize> {
        let width = self.width;

        if let Some(slice) = self.get_mut(index) {
            let mut remaining_width = width.saturating_sub(slice.len());
            let mut i = 0;

            for (symbol, width) in UnicodeSegmentation::graphemes(string.as_ref(), true)
                .filter(|symbol| !symbol.contains(char::is_control))
                .map(|(symbol)| (symbol, symbol.width()))
                .filter(|(_symbol, width)| *width > 0)
                .map_while(|(symbol, width)| {
                    remaining_width = remaining_width.checked_sub(width)?;
                    Some((symbol, width))
                }) {
                slice[i].set_measured_str(symbol, width, arena);

                let next_symbol = i + width;
                i += 1;
                // Reset following cells if multi-width (they would be hidden by the grapheme),
                while i < next_symbol {
                    slice[i].clear();
                    i += 1;
                }
            }
            Some(i)
        }
        else {
            None
        }
    }

    /// Print the given string until the end of the line.
    /// Skips zero-width graphemes and control characters.
    pub fn set_line(&mut self, point: impl Into<Point>, string: impl AsRef<str>, arena: &mut Arena) -> Option<usize> {
        let point = point.into();
        self.set_string(point..Point { x: self.width, y: point.y }, string, arena)
    }

    pub fn index_of<T>(&self, value: T) -> usize where Self: Resolve<T, usize> {
        self.resolve(value)
    }

    pub fn resolve<T, V>(&self, value: V) -> T where Self: Resolve<V, T> {
        self.resolve(value)
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
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.insert_cell_area(row, col, n, cell, self.bounds());
    }

    /// Delete `n` cells at `(x, y)`, shifting cells left (ANSI DCH).
    /// Operates on the full buffer width.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.delete_cell_area(row, col, n, cell, self.bounds());
    }

    /// Insert `n` lines at row `y` within specific bounds.
    /// Lines at `y` and below are shifted down; lines pushed beyond `bounds.max.y` are lost.
    /// New lines are filled with `cell`.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }

        // Clip to buffer bounds and ensure y is within bounds
        let bounds = self.clip(&Rect::from(bounds));
        let y = y.clamp(bounds.min.y, bounds.max.y);
        let n = n.min(bounds.max.y - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.y {
            return;
        }

        // Move lines down (backwards to prevent overwriting)
        // Source: [y, max-n) -> Dest: [y+n, max)
        for row in (y..(bounds.max.y - n)).rev() {
            let src_start = row * self.width() + bounds.min.x;
            let dst_start = (row + n) * self.width() + bounds.min.x;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Fill new lines with the provided cell
        for row in y..(y + n) {
            let start = row * self.width() + bounds.min.x;
            &mut self[start..start + width].fill(cell);
        }
    }

    /// Delete `n` lines at row `y` within specific bounds.
    /// Lines below shift up; new blank lines appear at bottom of bounds.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }
        let bounds = self.clip(&Rect::from(bounds));
        let y = y.clamp(bounds.min.y, bounds.max.y);
        let n = n.min(bounds.max.y - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.y {
            return;
        }

        let row_stride = self.width();

        // Move lines up
        // Source: [y+n, max) -> Dest: [y, max-n)
        for row in y..(bounds.max.y - n) {
            let src_start = (row + n) * row_stride + bounds.min.x;
            let dst_start = row * row_stride + bounds.min.x;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Clear bottom n lines
        for row in (bounds.max.y - n)..bounds.max.y {
            let start = row * row_stride + bounds.min.x;
            self[start..start + width].fill(cell);
        }
    }

    /// Insert `n` cells at `(x, y)` within specific bounds (ANSI ICH).
    /// Cells shift right; cells pushed beyond right margin are lost.
    pub fn insert_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(&Rect::from(bounds));

        // Validate y is within vertical bounds
        if row < bounds.min.y || row >= bounds.max.y {
            return;
        }

        let x = col.clamp(bounds.min.x, bounds.max.x);
        let n = n.min(bounds.max.x - x);

        if n == 0 {
            return;
        }

        let row_offset = row * self.width();

        // Shift cells right: [x, max-n) -> [x+n, max)
        if x + n < bounds.max.x {
            let src_start = row_offset + x;
            let src_end = row_offset + bounds.max.x - n;
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
    pub fn delete_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(&Rect::from(bounds));

        if row < bounds.min.y || row >= bounds.max.y {
            return;
        }

        let x = col.clamp(bounds.min.x, bounds.max.x);
        let n = n.min(bounds.max.x - x);

        if n == 0 {
            return;
        }

        let fill_cell = cell;
        let row_offset = row * self.width();

        // Shift cells left: [x+n, max) -> [x, max-n)
        if x + n < bounds.max.x {
            let src_start = row_offset + x + n;
            let src_end = row_offset + bounds.max.x;
            let dst_start = row_offset + x;
            self.copy_within(src_start..src_end, dst_start);
        }

        // Clear rightmost cells
        let clear_start = row_offset + bounds.max.x - n;
        let clear_end = row_offset + bounds.max.x;
        self[clear_start..clear_end].fill(fill_cell);
    }

    // Row operations
    pub fn push_row(&mut self, row: impl IntoIterator<Item = Cell>) {
        let row = row.into_iter();
        let (input_len, _) = row.size_hint();
        assert_ne!(input_len, 0);
        assert!(
            !(self.height > 0 && input_len != self.width),
            "pushed row does not match. Length must be {:?}, but was {:?}.",
            self.width,
            input_len
        );
        self.inner.extend(row);
        self.height += 1;
        if self.width == 0 {
            self.width = self.inner.len();
        }
    }

    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        if self.height == 0 {
            return None;
        }
        let row = self.inner.split_off(self.inner.len() - self.width);
        self.height -= 1;
        if self.height == 0 {
            self.width = 0;
        }
        Some(row)
    }

    pub fn remove_row(&mut self, row_index: usize) -> Option<Vec<Cell>> {
        if self.width == 0 || self.height == 0 || row_index >= self.height {
            return None;
        }
        let row = self
            .inner
            .drain((row_index * self.width)..((row_index + 1) * self.width))
            .collect();
        self.height -= 1;
        if self.height == 0 {
            self.width = 0;
        }
        Some(row)
    }

    pub fn insert_row(&mut self, index: usize, row: impl IntoIterator<Item = Cell>) {
        let row = row.into_iter();
        let (input_len, _) = row.size_hint();
        assert!(
            !(self.width > 0 && input_len != self.width),
            "Inserted row must be of length {}, but was {}.",
            self.width,
            input_len
        );
        assert!(
            index <= self.height,
            "Out of range. Index was {}, but must be less or equal to {}.",
            index,
            self.height
        );
        let data_idx = index * input_len;
        self.inner.splice(data_idx..data_idx, row);
        self.width = input_len;
        self.height += 1;
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let (cur_w, cur_h) = (self.width, self.height);
        if cur_w == width && cur_h == height {
            return;
        }

        if width != cur_w {
            let copy_w = width.min(cur_w);

            if width > cur_w {
                // Growing: extend first, then shift rows back-to-front
                self.inner.resize(width * cur_h, Cell::EMPTY);
                for y in (1..cur_h).rev() {
                    let src = y * cur_w;
                    let dst = y * width;
                    self.inner.copy_within(src..src + copy_w, dst);
                    // Fill the new columns
                    &mut self.inner[dst + copy_w..dst + width].fill(Cell::EMPTY);
                }
                // Row 0: just fill the tail
                &mut self.inner[copy_w..width].fill(Cell::EMPTY);
            } else {
                // Shrinking: shift rows front-to-back, then truncate
                for y in 1..cur_h {
                    let src = y * cur_w;
                    let dst = y * width;
                    self.inner.copy_within(src..src + copy_w, dst);
                }
                self.inner.truncate(width * cur_h);
            }

            self.width = width;
        }

        if height > cur_h {
            self.inner.resize(width * height, Cell::EMPTY);
        } else if height < cur_h {
            self.inner.truncate(width * height);
        }
        self.height = height;
    }

    pub fn clear(&mut self) {
        self.inner.fill(Cell::default());
    }

    pub fn iter_col(&self, col: usize) -> StepBy<Iter<Cell>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );

        self.inner[col..].iter().step_by(self.width)
    }

    pub fn iter_col_mut(&mut self, col: usize) -> StepBy<IterMut<Cell>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );
        self.inner[col..].iter_mut().step_by(self.width)
    }

    pub fn iter_row(&self, row: usize) -> Iter<Cell> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        self[row * self.width..row * self.width + self.width].iter()
    }

    pub fn iter_row_mut(&mut self, row: usize) -> IterMut<Cell> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        let width = self.width;

        self[row * width..row * width + width].iter_mut()
    }

    pub fn iter_rect(&self, rect: &Rect) -> impl Iterator<Item = &Cell> {
       rect.iter().map(|point| &self[point])
    }

    pub fn indexed_iter(&self) -> impl Iterator<Item = ((usize, usize), &Cell)> {
        self.iter()
            .enumerate()
            .map(move |(idx, i)| ((idx / self.width, idx % self.width), i))
    }

    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = ((usize, usize), &mut Cell)> {
        let cols = self.width;

        self.iter_mut()
            .enumerate()
            .map(move |(idx, i)| ((idx / cols, idx % cols), i))
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = Iter<Cell>> {
        (0..self.height).map(move |row| self.iter_row(row))
    }

    pub fn iter_cols(&self) -> impl Iterator<Item = StepBy<Iter<Cell>>> {
        (0..self.width).map(move |col| self.iter_col(col))
    }

    pub fn iter(&self) -> Iter<Cell> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Cell> {
        self.iter_mut()
    }

    pub fn to_string(&self, arena: &Arena) -> String {
        self.iter_rows()
            .map(|row| row.map(|cell| cell.as_str(arena)).collect::<String>())
            .intersperse(String::from("\n"))
            .collect()
    }

    pub fn as_buf<'a>(&'a self, arena: &'a Arena) -> Buf<'a> {
        Buf::new(self, arena)
    }

    pub fn as_buf_mut<'a>(&'a mut self, arena: &'a mut Arena) -> BufMut<'a> {
        BufMut::new(self, arena)
    }
}

impl From<Rect> for Buffer {
    fn from(value: Rect) -> Self {
        Self::new(value.width(), value.height())
    }
}

impl<I: BufferIndex> Index<I> for Buffer {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        index.index_of(self).index(self)
    }
}

impl<I: BufferIndex> IndexMut<I> for Buffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_of(self).index_mut(self)
    }
}

impl Bounded for Buffer {
    type Coordinate = Point;
    type Bounds = Rect;

    fn min_x(&self) -> usize {
        0
    }

    fn min_y(&self) -> usize {
        0
    }

    fn max_x(&self) -> usize {
        self.width
    }

    fn max_y(&self) -> usize {
        self.height
    }

    fn min(&self) -> Self::Coordinate {
        Point::ZERO
    }

    fn max(&self) -> Self::Coordinate {
        Point::new(self.max_x(), self.max_y())
    }

    fn bounds(&self) -> Self::Bounds {
        Rect::bounds(self.min(), self.max())
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if self.width > 0 {
            if f.alternate() {
                writeln!(f)?;
                /*
                    WARNING

                    Compound types becoming enormous as the entire `fmt::Debug` width is applied to each item individually.
                    For tuples and structs define padding and precision arguments manually to improve readability.
                */
                let width = f.width().unwrap_or_else(|| {
                    // Conditionally calculate the longest item by default.
                    self.inner
                        .iter()
                        .map(|i| format!("{i:?}").len())
                        .max()
                        .unwrap_or(0)
                });
                let precision = f.precision().unwrap_or(2);
                for mut row in self.iter_rows().map(Iterator::peekable) {
                    write!(f, "    [")?;
                    while let Some(item) = row.next() {
                        write!(f, " {item:width$.precision$?}")?;
                        if row.peek().is_some() {
                            write!(f, ",")?;
                        }
                    }
                    writeln!(f, "]")?;
                }
            } else {
                for row in self.iter_rows() {
                    f.debug_list().entries(row).finish()?;
                }
            }
        }
        write!(f, "]")
    }
}
impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        if self.height != other.height || self.width != other.width {
            return false;
        }
        for (self_row, other_row) in core::iter::zip(self.iter_rows(), other.iter_rows()) {
            if self_row.ne(other_row) {
                return false;
            }
        }
        true
    }
}
