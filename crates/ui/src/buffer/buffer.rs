use crate::{Graphemes, BufferDiff, BufferIndex, ByCells, ByRuns, Cell, Cells, TrackingBuffer, DiffStrategy};
use ansi::Style;
use core::slice::IterMut;
use std::borrow::{Borrow, BorrowMut};
use derive_more::{ Deref, DerefMut, IntoIterator};
use geometry::Resolve;
use geometry::{Bound, Intersect, Point, Rect};
use std::fmt::{Debug, Display, Formatter, Write};
use std::iter::StepBy;
use std::ops::{Range};
use std::slice::Iter;
use std::slice::SliceIndex;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Deref, DerefMut, IntoIterator, Clone, PartialEq)]
pub struct Buffer {
    #[deref(forward)]
    #[deref_mut]
    #[into_iterator(owned, ref, ref_mut)]
    inner: Vec<Cell>,
    width: usize,
    height: usize,
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
            width: width,
            height: height,
        }
    }

    /// Create a buffer from a slice of fixed elements.
    ///
    /// A convenience constructor mostly used for tests.
    pub fn from_cells(width: usize, height: usize, chars: &[(usize, usize, char, Style)]) -> Self {
        let mut buffer = Self::new(width, height);
        for &(row, col, ch, style) in chars {
            buffer[(row, col)] = Cell::new(ch).with_style(style);
        }
        buffer
    }

    /// Create a buffer from a slice of fixed elements.
    #[must_use]
    pub fn from_lines<'a>(lines: impl IntoIterator<Item = &'a str>, arena: &mut Graphemes) -> Self {
        let lines = lines.into_iter().collect::<Vec<_>>();
        let height = lines.len();
        let width = lines
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or_default();
        let mut buffer = Self::new(width, height);
        for (y, line) in lines.iter().enumerate() {
            buffer.set_line(Point { x: 0, y: y as u16 }, line, arena);
        }
        buffer
    }

    /// Creates a [`Buffer`] by calling `f` for each cell position.
    ///
    /// # Example
    /// ```ignore
    /// let buf = Buffer::from_fn(10, 5, |row, col| Cell::inline(char::from(b'A' + (row * 10 + col) as u8 % 26)));
    /// ```
    pub fn from_fn(width: usize, height: usize, f: impl Fn(usize, usize) -> Cell) -> Self {
        let mut inner = Vec::with_capacity(width * height);
        for row in 0..height {
            for col in 0..width {
                inner.push(f(row, col));
            }
        }
        Self {
            inner,
            width,
            height,
        }
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
        unsafe { index.get_unchecked(self) }
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
        unsafe { index.get_unchecked_mut(self) }
    }

    pub fn contains<I: BufferIndex>(&self, index: I) -> bool {
        index.get(self).is_some()
    }

    /// Print the given string until the end of the given index.
    pub fn set_string(
        &mut self,
        index: impl BufferIndex<Output = [Cell]>,
        string: impl AsRef<str>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
       self.set_string_impl(index, string, None, arena)
    }

    pub fn set_string_styled(
        &mut self,
        index: impl BufferIndex<Output = [Cell]>,
        string: impl AsRef<str>,
        style: Style,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        self.set_string_impl(index, string, Some(style), arena)
    }

    /// Write one measured grapheme and all of its continuation cells.
    ///
    /// Returns `None` when the grapheme is zero-width or its complete display
    /// width does not fit in the buffer row.
    pub fn set_grapheme_styled(
        &mut self,
        position: Point,
        grapheme: &str,
        width: usize,
        style: Style,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        if width == 0 || position.y as usize >= self.height {
            return None;
        }

        let start_x = position.x as usize;
        let end_x = start_x.checked_add(width)?;

        if end_x > self.width {
            return None;
        }

        let start = position.y as usize * self.width + start_x;
        Some(
            Cells::write_into(&mut self.inner[start..start + width], grapheme, width, Some(style), arena),
        )
    }

    /// Print the given string until the end of the given index.
    fn set_string_impl(
        &mut self,
        index: impl BufferIndex<Output = [Cell]>,
        string: impl AsRef<str>,
        style: Option<Style>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        let slice = self.get_mut(index)?;
        let mut remaining = slice.len();
        let mut i = 0;

        for grapheme in string.as_ref().graphemes(true) {
            if grapheme.contains(char::is_control) {
                continue;
            }

            let width = grapheme.width();
            remaining = remaining.checked_sub(width)?;

            i += Cells::write_into((&mut slice[i..]), grapheme, width, style, arena);
        }

        Some(i)
    }
    /// Print the given string until the end of the line.
    pub fn set_line(
        &mut self,
        start: Point,
        string: impl AsRef<str>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        self.set_string(
            start..Point {
                x: self.width as u16,
                y: start.y,
            },
            string,
            arena,
        )
    }

    pub fn index_of<T>(&self, value: T) -> usize
    where
        Self: Resolve<T, usize>,
    {
        self.resolve(value)
    }

    pub fn point_of<T>(&self, value: T) -> Point
    where
        Self: Resolve<T, Point>,
    {
        self.resolve(value)
    }

    /// Returns the slice index of the given buffer index.
    pub fn slice_index_of<I: BufferIndex>(&self, index: I) -> I::Index {
        index.into_slice_index(self)
    }
    /// Insert `n` lines at row `y`, shifting remaining lines down (ANSI IL).
    /// Operates on the full buffer width.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) -> Range<usize> {
        self.insert_line_area(y, n, cell, self.bounds())
    }

    /// Delete `n` lines at row `y`, shifting remaining lines up (ANSI DL).
    /// Operates on the full buffer width.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) -> Range<usize> {
        self.delete_line_area(y, n, cell, self.bounds())
    }

    /// Insert `n` cells at `(x, y)`, shifting cells right (ANSI ICH).
    /// Operates on the full buffer width.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) -> Range<usize> {
        self.insert_cell_area(row, col, n, cell, self.bounds())
    }

    /// Delete `n` cells at `(x, y)`, shifting cells left (ANSI DCH).
    /// Operates on the full buffer width.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) -> Range<usize> {
        self.delete_cell_area(row, col, n, cell, self.bounds())
    }

    /// Insert `n` lines at row `y` within specific bounds.
    /// Lines at `y` and below are shifted down; lines pushed beyond `bounds.max.y` are lost.
    /// New lines are filled with `cell`.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn insert_line_area(
        &mut self,
        y: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) -> Range<usize> {
        if n == 0 {
            return 0..0;
        }

        // Clip to buffer bounds and ensure y is within bounds
        let bounds = self.clip(&bounds);
        let min_x = bounds.min.x as usize;
        let min_y = bounds.min.y as usize;
        let max_y = bounds.max.y as usize;
        let y = y.clamp(min_y, max_y);
        let n = n.min(max_y - y);
        let width = bounds.width() as usize;

        if width == 0 || y >= max_y {
            return 0..0;
        }

        // If the bounds cover the entire width of the buffer, we can use a
        // single copy operation.
        if min_x == 0 && width == self.width {
            let stride = self.width;

            let src_start = y * stride;
            let src_end = (max_y - n) * stride;
            let dst_start = (y + n) * stride;

            self.inner.copy_within(src_start..src_end, dst_start);

            let fill_start = y * stride;
            let fill_end = (y + n) * stride;
            self.inner[fill_start..fill_end].fill(cell);

            return y..max_y;
        }

        // Move lines down (backwards to prevent overwriting)
        // Source: [y, max-n) -> Dest: [y+n, max)
        for row in (y..(max_y - n)).rev() {
            let src_start = row * self.width() + min_x;
            let dst_start = (row + n) * self.width() + min_x;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Fill new lines with the provided cell
        for row in y..(y + n) {
            let start = row * self.width() + min_x;
            self[start..start + width].fill(cell);
        }

        // Every row from the insertion point to the bottom of the region was
        // either shifted down or filled.
        y..max_y
    }

    /// Delete `n` lines at row `y` within specific bounds.
    /// Lines below shift up; new blank lines appear at bottom of bounds.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn delete_line_area(
        &mut self,
        y: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) -> Range<usize> {
        if n == 0 {
            return 0..0;
        }
        let bounds = self.clip(&bounds);
        let min_x = bounds.min.x as usize;
        let min_y = bounds.min.y as usize;
        let max_y = bounds.max.y as usize;
        let y = y.clamp(min_y, max_y);
        let n = n.min(max_y - y);
        let width = bounds.width() as usize;

        if width == 0 || y >= max_y {
            return 0..0;
        }

        if min_x == 0 && width == self.width {
            let stride = self.width;

            let src_start = (y + n) * stride;
            let src_end = max_y * stride;
            let dst_start = y * stride;

            self.inner.copy_within(src_start..src_end, dst_start);

            let clear_start = (max_y - n) * stride;
            let clear_end = max_y * stride;
            self.inner[clear_start..clear_end].fill(cell);

            return y..max_y;
        }

        let row_stride = self.width();

        // Move lines up
        // Source: [y+n, max) -> Dest: [y, max-n)
        for row in y..(max_y - n) {
            let src_start = (row + n) * row_stride + min_x;
            let dst_start = row * row_stride + min_x;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Clear bottom n lines
        for row in (max_y - n)..max_y {
            let start = row * row_stride + min_x;
            self[start..start + width].fill(cell);
        }

        // Every row from the deletion point to the bottom of the region was
        // either shifted up or cleared.
        y..max_y
    }

    /// Insert `n` cells at `(x, y)` within specific bounds (ANSI ICH).
    /// Cells shift right; cells pushed beyond right margin are lost.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn insert_cell_area(
        &mut self,
        row: usize,
        col: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) -> Range<usize> {
        if n == 0 {
            return 0..0;
        }

        let bounds = self.clip(&bounds);
        let min_x = bounds.min.x as usize;
        let max_x = bounds.max.x as usize;
        let min_y = bounds.min.y as usize;
        let max_y = bounds.max.y as usize;

        // Validate y is within vertical bounds
        if row < min_y || row >= max_y {
            return 0..0;
        }

        let x = col.clamp(min_x, max_x);
        let n = n.min(max_x - x);

        if n == 0 {
            return 0..0;
        }

        let row_offset = row * self.width();

        // Shift cells right: [x, max-n) -> [x+n, max)
        if x + n < max_x {
            let src_start = row_offset + x;
            let src_end = row_offset + max_x - n;
            let dst_start = row_offset + x + n;
            self.copy_within(src_start..src_end, dst_start);
        }

        // Fill insertion point
        let fill_start = row_offset + x;
        let fill_end = fill_start + n;
        self[fill_start..fill_end].fill(cell);

        row..row + 1
    }

    /// Delete `n` cells at `(x, y)` within specific bounds (ANSI DCH).
    /// Cells shift left; new blank cells appear at right margin.
    ///
    /// Returns the range of rows whose contents changed (empty if none did).
    pub fn delete_cell_area(
        &mut self,
        row: usize,
        col: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) -> Range<usize> {
        if n == 0 {
            return 0..0;
        }

        let bounds = self.clip(&bounds);
        let min_x = bounds.min.x as usize;
        let max_x = bounds.max.x as usize;
        let min_y = bounds.min.y as usize;
        let max_y = bounds.max.y as usize;

        if row < min_y || row >= max_y {
            return 0..0;
        }

        let x = col.clamp(min_x, max_x);
        let n = n.min(max_x - x);

        if n == 0 {
            return 0..0;
        }

        let fill_cell = cell;
        let row_offset = row * self.width();

        // Shift cells left: [x+n, max) -> [x, max-n)
        if x + n < max_x {
            let src_start = row_offset + x + n;
            let src_end = row_offset + max_x;
            let dst_start = row_offset + x;
            self.copy_within(src_start..src_end, dst_start);
        }

        // Clear rightmost cells
        let clear_start = row_offset + max_x - n;
        let clear_end = row_offset + max_x;
        self[clear_start..clear_end].fill(fill_cell);

        row..row + 1
    }

    // Row operations
    pub fn push_row<I>(&mut self, row: I)
    where
        I: IntoIterator<Item = Cell>,
        I::IntoIter: ExactSizeIterator,
    {
        let row = row.into_iter();
        let len = row.len();

        assert_ne!(len, 0);

        if self.height > 0 {
            assert_eq!(
                len, self.width,
                "pushed row does not match. Length must be {}, but was {}.",
                self.width, len
            );
        }

        self.inner.extend(row);
        self.height += 1;

        if self.width == 0 {
            self.width = len;
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
        if index > self.height {
            return;
        }

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
                    self.inner[dst + copy_w..dst + width].fill(Cell::EMPTY);
                }
                // Row 0: just fill the tail
                self.inner[copy_w..width].fill(Cell::EMPTY);
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
        self.inner.fill(Cell::EMPTY);
    }

    pub fn iter_col(&self, col: usize) -> StepBy<Iter<'_, Cell>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );

        self.inner[col..].iter().step_by(self.width)
    }

    pub fn iter_col_mut(&mut self, col: usize) -> StepBy<IterMut<'_, Cell>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );
        self.inner[col..].iter_mut().step_by(self.width)
    }

    pub fn iter_row(&self, row: usize) -> Iter<'_, Cell> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        self[row * self.width..row * self.width + self.width].iter()
    }

    pub fn iter_row_mut(&mut self, row: usize) -> IterMut<'_, Cell> {
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
        rect.steps().map(|point| &self[point])
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

    pub fn iter_rows(&self) -> impl Iterator<Item = Iter<'_, Cell>> {
        (0..self.height).map(move |row| self.iter_row(row))
    }

    pub fn iter_cols(&self) -> impl Iterator<Item = StepBy<Iter<'_, Cell>>> {
        (0..self.width).map(move |col| self.iter_col(col))
    }

    pub fn iter(&self) -> Iter<'_, Cell> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Cell> {
        self.inner.iter_mut()
    }

    pub fn to_string(&self, arena: &Graphemes) -> String {
        if self.is_empty() {
            return String::new();
        }

        let last_cell = self.iter().rposition(|c| !c.is_empty()).unwrap_or(0);

        if last_cell == 0 {
            return String::new();
        }

        let mut out = String::with_capacity(last_cell + (last_cell / 2));

        for y in 0..self.height {
            if y > 0 {
                out.push('\n');
            }

            let start = y * self.width;
            let end = start + self.width;

            for cell in &self.inner[start..end] {
                out.push_str(cell.as_str_or(arena, " "));
            }
        }

        out
    }

    /// Returns a [`BufferCells`] between the cells of `prev` and `next`.
    pub fn diff<'a, Strategy: DiffStrategy<'a>>(prev: &'a Strategy::Prev, next: &'a Strategy::Next) -> BufferDiff<'a, Strategy> {
        BufferDiff::new(prev, next)
    }

    /// Returns a [`BufferCells`] iterator.
    /// Yields a [`Changed`] for each cell that differs between `prev` and `next`.
    pub fn diff_cells<'a>(prev: &'a Buffer, next: &'a Buffer) -> BufferDiff<'a, ByCells> {
        Self::diff(prev, next)
    }

    /// Returns a [`BufferRuns`] iterator.
    /// Yields a [`Run`] for each run of changed cells on the same row.
    pub fn diff_runs<'a>(prev: &'a Buffer, next: &'a Buffer) -> BufferDiff<'a, ByRuns> {
        Self::diff(prev, next)
    }

    /// Create a [`TrackingBuffer`] from this buffer.
    ///
    /// All rows are marked.
    pub fn into_tracking(self) -> TrackingBuffer {
        TrackingBuffer::from_buffer_marked(self)
    }
}

impl Bound for Buffer {
    type Point = Point;

    fn min_x(&self) -> u16 {
        0
    }

    fn min_y(&self) -> u16 {
        0
    }

    fn max_x(&self) -> u16 {
        self.width as u16
    }

    fn max_y(&self) -> u16 {
        self.height as u16
    }

    fn min(&self) -> Self::Point {
        Point::ZERO
    }

    fn max(&self) -> Self::Point {
        Point::new(self.max_x(), self.max_y())
    }
}
const impl AsRef<[Cell]> for Buffer {
    fn as_ref(&self) -> &[Cell] {
        &self.inner
    }
}
const impl AsMut<[Cell]> for Buffer {
    fn as_mut(&mut self) -> &mut [Cell] {
        &mut self.inner
    }
}
const impl Borrow<[Cell]> for Buffer {
    fn borrow(&self) -> &[Cell] {
        self.as_ref()
    }
}
const impl BorrowMut<[Cell]> for Buffer {
    fn borrow_mut(&mut self) -> &mut [Cell] {
        self.as_mut()
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

impl Display for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.height == 0 || self.width == 0 {
            return Ok(())
        }

        for y in 0..self.height {
            if y > 0 {
                f.write_char('\n')?;
            }

            let start = y * self.width;
            let end = start + self.width;

            for cell in &self.inner[start..end] {
                write!(f, "{:?}", cell)?;
            }
        }
        Ok(())
    }
}
#[test]
fn qwe() {
    let buf = Buffer::new(10, 5);

}
