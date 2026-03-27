use crate::{Cell, GraphemeArena, BufferIndex};
use ansi::Style;
use core::slice::IterMut;
use derive_more::{AsMut, AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};
use geometry::{Bounded, Intersect, Point, Rect, Resolve};
use std::cmp;
use std::fmt::Debug;
use std::iter::StepBy;
use std::ops::{Index, IndexMut};
use std::slice::Iter;
use std::slice::SliceIndex;

#[derive(Deref, DerefMut, AsRef, AsMut, IntoIterator, Clone)]
pub struct Buffer {
    #[deref]
    #[deref_mut]
    #[as_ref(forward)]
    #[as_mut(forward)]
    #[into_iterator(owned, ref, ref_mut)]
    inner: Vec<Cell>,
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
            buffer[(row, col)] = Cell::from_char(ch, style);
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
    pub fn get<I: BufferIndex<Buffer, [Cell]>>(
        &self,
        index: I,
    ) -> Option<&<I::Index as SliceIndex<[Cell]>>::Output> {
        index.index_of(self).get(self.as_ref())
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: BufferIndex<Buffer, [Cell]>>(
        &mut self,
        index: I,
    ) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output> {
        index.index_of(self).get_mut(self.as_mut())
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked<I: BufferIndex<Buffer, [Cell]>>(
        &self,
        index: I,
    ) -> *const <I::Index as SliceIndex<[Cell]>>::Output {
        SliceIndex::get_unchecked(index.index_of(self), self.as_ref())
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<I: BufferIndex<Buffer, [Cell]>>(
        &mut self,
        index: I,
    ) -> *mut <I::Index as SliceIndex<[Cell]>>::Output {
        SliceIndex::get_unchecked_mut(index.index_of(self), self.as_mut())
    }

    pub fn index_of<T: Resolve<usize, Rect>>(&self, value: T) -> usize {
        value.resolve(self.bounds().into())
    }

    // Row  operations
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

    pub fn flip_rows(&mut self) {
        for row in 0..self.height / 2 {
            for col in 0..self.width {
                let cell1 = self.index_of((row, col));
                let cell2 = self.index_of((self.height - row - 1, col));
                self.inner.swap(cell1, cell2);
            }
        }
    }

    // Column operations

    pub fn push_col(&mut self, col: impl IntoIterator<Item = Cell>) {
        let col = col.into_iter();
        let (input_len, _) = col.size_hint();
        assert_ne!(input_len, 0);
        assert!(
            !(self.width > 0 && input_len != self.height),
            "pushed column does not match. Length must be {:?}, but was {:?}.",
            self.height,
            input_len
        );
        self.inner.extend(col);
        for i in (1..self.height).rev() {
            let row_idx = i * self.width;
            self.inner[row_idx..row_idx + self.width + i].rotate_right(i);
        }
        self.width += 1;
        if self.height == 0 {
            self.height = self.inner.len();
        }
    }

    pub fn pop_col(&mut self) -> Option<Vec<Cell>> {
        if self.width == 0 {
            return None;
        }
        for i in 1..self.height {
            let row_idx = i * (self.width - 1);
            self.inner[row_idx..row_idx + self.width + i - 1].rotate_left(i);
        }
        let col = self.inner.split_off(self.inner.len() - self.height);
        self.width -= 1;
        if self.width == 0 {
            self.height = 0;
        }
        Some(col)
    }

    pub fn insert_col(&mut self, index: usize, col: impl IntoIterator<Item = Cell>) {
        let col = col.into_iter();
        let (input_len, _) = col.size_hint();
        assert!(
            !(self.height > 0 && input_len != self.height),
            "Inserted col must be of length {}, but was {}.",
            self.height,
            input_len
        );
        assert!(
            index <= self.width,
            "Out of range. Index was {}, but must be less or equal to {}.",
            index,
            self.width
        );
        for (row_iter, col_val) in col.enumerate() {
            let data_idx = row_iter * self.width + index + row_iter;
            self.inner.insert(data_idx, col_val);
        }
        self.height = input_len;
        self.width += 1;
    }

    pub fn remove_col(&mut self, col_index: usize) -> Option<Vec<Cell>> {
        if self.width == 0 || self.height == 0 || col_index >= self.width {
            return None;
        }
        let col = {
            for i in 0..self.height {
                let row_idx = col_index + i * (self.width - 1);
                let end = cmp::min(row_idx + self.width + i, self.inner.len());
                self.inner[row_idx..end].rotate_left(i + 1);
            }
            self.inner.split_off(self.inner.len() - self.height)
        };
        self.width -= 1;
        if self.width == 0 {
            self.height = 0;
        }
        Some(col)
    }

    pub fn flip_cols(&mut self) {
        for row in 0..self.height {
            let idx = row * self.width;
            self.inner[idx..idx + self.width].reverse();
        }
    }

    pub fn map_or<I, F>(&mut self, index: I, default: Cell, mut f: F)
    where
        I: BufferIndex<Buffer, [Cell], Output = Cell>,
        F: FnMut(&mut Cell),
    {
        match self.get_mut(index.clone()) {
            Some(cell) => f(cell),
            None => self[index] = default,
        }
    }

    pub fn map<I, F>(&mut self, index: I, mut f: F)
    where
        I: BufferIndex<Buffer, [Cell], Output = Cell>,
        F: FnMut(&mut Cell),
    {
        match self.get_mut(index) {
            Some(cell) => f(cell),
            None => {}
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

    pub fn copy_from_area(&mut self, area: &Rect) -> Self {
        let mut next = Self::from(self.clip(area));

        for position in area {
            next[(position.y - area.min.y, position.x - area.min.x)] = self[position];
        }

        next
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.resize_with(width, height, Cell::default());
    }


    pub fn resize_with(&mut self, width: usize, height: usize, value: Cell) {
        let (cur_w, cur_h) = (self.width, self.height);
        if cur_w == width && cur_h == height {
            return;
        }

        if width != cur_w {
            let copy_w = width.min(cur_w);

            if width > cur_w {
                // Growing: extend first, then shift rows back-to-front
                self.inner.resize(width * cur_h, value);
                for y in (1..cur_h).rev() {
                    let src = y * cur_w;
                    let dst = y * width;
                    self.inner.copy_within(src..src + copy_w, dst);
                    // Fill the new columns
                    &mut self.inner[dst + copy_w..dst + width].fill(value);
                }
                // Row 0: just fill the tail
                &mut self.inner[copy_w..width].fill(value);
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
            self.inner.resize(width * height, value);
        } else if height < cur_h {
            self.inner.truncate(width * height);
        }
        self.height = height;
    }

    pub fn resize_inner(&mut self, width: usize, height: usize) {
        self.inner.reserve(width * height - self.len());
        self.inner.fill(Cell::default());
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

    pub fn iter_row(&self, row: usize) -> StepBy<Iter<Cell>> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        self[row * self.width..row * self.width + self.width].iter().step_by(1)
    }

    pub fn iter_row_mut(&mut self, row: usize) -> StepBy<IterMut<Cell>> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        let width = self.width;

        self[row * width..row * width + width].iter_mut().step_by(1)
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

    pub fn iter_rows(&self) -> impl Iterator<Item = StepBy<Iter<Cell>>> {
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

    pub fn to_string(&self, arena: &GraphemeArena) -> String {
        self.iter_rows()
            .map(|row| row.map(|cell| cell.as_str(arena)).collect::<String>())
            .intersperse(String::from("\n"))
            .collect()
    }
}

impl From<Rect> for Buffer {
    fn from(value: Rect) -> Self {
        Self::new(value.width(), value.height())
    }
}


impl<I: BufferIndex<Buffer, [Cell]>> Index<I> for Buffer {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        index.index_of(self).index(self)
    }
}

impl<I: BufferIndex<Buffer, [Cell]>> IndexMut<I> for Buffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_of(self).index_mut(self)
    }
}

impl Bounded for Buffer {
    type Point = Point;
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

    fn min(&self) -> Self::Point {
        Point::ZERO
    }

    fn max(&self) -> Self::Point {
        Point::new(self.max_x(), self.max_y())
    }

    fn bounds(&self) -> Self::Bounds {
        Rect::new(self.min(), self.max())
    }
}

impl Intersect<Rect> for Buffer {
    type Output = Rect;

    fn intersect(&self, other: &Rect) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Rect::<Point>::ZERO;
        }

        let mut r = Rect::<Point>::ZERO;

        let x1 = 0.max(other.min.x);
        let y1 = 0.max(other.min.y);
        let x2 = self.width().min(other.max.x);
        let y2 = self.height().min(other.max.y);

        r.min.x = x1;
        r.min.y = y1;

        let mut w = x2 - x1;
        let mut h = y2 - y1;

        if w < 0 {
            w = 0;
        }

        if h < 0 {
            h = 0;
        }

        if w > usize::MAX {
            w = usize::MAX;
        }

        if h > usize::MAX {
            h = usize::MAX;
        }

        r.max.x = r.min.x + w;
        r.max.y = r.min.y + h;

        r
    }
}

impl Intersect<Buffer> for Rect {
    type Output = Rect;

    fn intersect(&self, other: &Buffer) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Rect::<Point>::ZERO;
        }

        let mut r = Rect::<Point>::ZERO;

        let x1 = self.min_x().max(other.min_x());
        let y1 = self.min_y().max(other.min_y());
        let x2 = self.max_x().min(other.max_x());
        let y2 = self.max_y().min(other.max_y());

        r.min.x = x1;
        r.min.y = y1;

        let mut w = x2 - x1;
        let mut h = y2 - y1;

        if w < 0 {
            w = 0;
        }

        if h < 0 {
            h = 0;
        }

        if w > usize::MAX {
            w = usize::MAX;
        }

        if h > usize::MAX {
            h = usize::MAX;
        }

        r.max.x = r.min.x + w;
        r.max.y = r.min.y + h;

        r
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

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
pub struct Buf<'a> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    inner: &'a mut Buffer,
    arena: &'a mut GraphemeArena,
}

impl<'a> Buf<'a> {
    pub fn new(buffer: &'a mut Buffer, arena: &'a mut GraphemeArena) -> Self {
        Self { inner: buffer, arena }
    }

    pub fn arena(&mut self) -> &mut GraphemeArena {
        self.arena
    }
}