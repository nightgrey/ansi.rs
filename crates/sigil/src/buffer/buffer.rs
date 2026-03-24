use core::slice::IterMut;
use std::slice::Iter;
use std::fmt::Debug;
use std::iter::StepBy;
use std::{cmp};
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;
use derive_more::{AsMut, AsRef, Deref, DerefMut,  IntoIterator};
use ansi::{Style};
use geometry::{Area, Bounded, Intersect, Point, Position, PositionLike, Rect, Resolve, Row};
use crate::{Cell, GraphemeArena, IntoSliceIndex};

#[derive(Deref, DerefMut, AsRef, AsMut, IntoIterator, Clone)]
pub struct Buffer {
    #[deref]
    #[deref_mut]
    #[as_ref(forward)]
    #[as_mut(forward)]
    #[into_iterator(owned, ref, ref_mut)]
    pub data: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Buffer {
    pub const EMPTY: Self = Self {
        data: Vec::new(),
        width: 0,
        height: 0,
    };

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: Vec::with_capacity(width * height),
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
    pub fn get<I: IntoSliceIndex<Buffer, [Cell]>>(&self, index: I) -> Option<&<I::Index as SliceIndex<[Cell]>>::Output> {
        index.into_slice_index(self).get(self.as_ref())
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: IntoSliceIndex<Buffer, [Cell]>>(&mut self, index: I) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output> {
        index.into_slice_index(self).get_mut(self.as_mut())
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked<I: IntoSliceIndex<Buffer, [Cell]>>(&self, index: I) -> *const <I::Index as SliceIndex<[Cell]>>::Output {
        SliceIndex::get_unchecked(index.into_slice_index(self), self.as_ref())
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<I: IntoSliceIndex<Buffer, [Cell]>>(&mut self, index: I) -> *mut <I::Index as SliceIndex<[Cell]>>::Output {
        SliceIndex::get_unchecked_mut(index.into_slice_index(self), self.as_mut())
    }

    pub fn index_of<T: Resolve<usize, Area>>(&self, value: T) -> usize {
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
        self.data.extend(row);
        self.height += 1;
        if self.width == 0 {
            self.width = self.data.len();
        }
    }

    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        if self.height == 0 {
            return None;
        }
        let row = self.data.split_off(self.data.len() - self.width);
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
            .data
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
        self.data.splice(data_idx..data_idx, row);
        self.width = input_len;
        self.height += 1;
    }

    pub fn flip_rows(&mut self) {
        for row in 0..self.height / 2 {
            for col in 0..self.width {
                let cell1 = self.index_of((row, col));
                let cell2 = self.index_of((self.height - row - 1, col));
                self.data.swap(cell1, cell2);
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
        self.data.extend(col);
        for i in (1..self.height).rev() {
            let row_idx = i * self.width;
            self.data[row_idx..row_idx + self.width + i].rotate_right(i);
        }
        self.width += 1;
        if self.height == 0 {
            self.height = self.data.len();
        }
    }

    pub fn pop_col(&mut self) -> Option<Vec<Cell>> {
        if self.width == 0 {
            return None;
        }
        for i in 1..self.height {
            let row_idx = i * (self.width - 1);
            self.data[row_idx..row_idx + self.width + i - 1].rotate_left(i);
        }
        let col = self.data.split_off(self.data.len() - self.height);
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
            self.data.insert(data_idx, col_val);
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
                let end = cmp::min(row_idx + self.width + i, self.data.len());
                self.data[row_idx..end].rotate_left(i + 1);
            }
            self.data.split_off(self.data.len() - self.height)
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
            self.data[idx..idx + self.width].reverse();
        }
    }

    pub fn map_or<I, F>(&mut self, index: I, default: Cell, mut f: F)
    where
        I: IntoSliceIndex<Buffer, [Cell], Output = Cell>,
        F: FnMut(&mut Cell),
    {
        match self.get_mut(index.clone()) {
            Some(cell) => f(cell),
            None => self[index] = default,
        }
    }

    pub fn map<I, F>(&mut self, index: I, mut f: F)
    where
        I: IntoSliceIndex<Buffer, [Cell], Output = Cell>,
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
    /// Lines at `y` and below are shifted down; lines pushed beyond `bounds.max.row` are lost.
    /// New lines are filled with `cell`.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }

        // Clip to buffer bounds and ensure y is within bounds
        let bounds = self.clip(&Area::from(bounds));
        let y = y.clamp(bounds.min.row, bounds.max.row);
        let n = n.min(bounds.max.row - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.row {
            return;
        }


        // Move lines down (backwards to prevent overwriting)
        // Source: [y, max-n) -> Dest: [y+n, max)
        for row in (y..(bounds.max.row - n)).rev() {
            let src_start = row * self.width() + bounds.min.col;
            let dst_start = (row + n) * self.width() + bounds.min.col;
            self.copy_within(src_start..src_start + width, dst_start);
        }

        // Fill new lines with the provided cell
        for row in y..(y + n) {
            let start = row * self.width() + bounds.min.col;
            &mut self[start..start + width].fill(cell);
        }
    }

    /// Delete `n` lines at row `y` within specific bounds.
    /// Lines below shift up; new blank lines appear at bottom of bounds.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }
        let bounds = self.clip(&Area::from(bounds));
        let y = y.clamp(bounds.min.row, bounds.max.row);
        let n = n.min(bounds.max.row - y);
        let width = bounds.width();

        if width == 0 || y >= bounds.max.row {
            return;
        }

        let row_stride = self.width();

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
    pub fn insert_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(&Area::from(bounds));

        // Validate y is within vertical bounds
        if row < bounds.min.row || row >= bounds.max.row {
            return;
        }

        let x = col.clamp(bounds.min.col, bounds.max.col);
        let n = n.min(bounds.max.col - x);

        if n == 0 {
            return;
        }

        let row_offset = row * self.width();

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
    pub fn delete_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        if n == 0 {
            return;
        }

        let bounds = self.clip(&Area::from(bounds));

        if row < bounds.min.row || row >= bounds.max.row {
            return;
        }

        let x = col.clamp(bounds.min.col, bounds.max.col);
        let n = n.min(bounds.max.col - x);

        if n == 0 {
            return;
        }

        let fill_cell = cell;
        let row_offset = row * self.width();

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

    pub fn copy_from_area(&mut self, area: &Area) -> Self {
        let mut next = Self::from(self.clip(area));

        for position in area {
            next[(position.row - area.min.row, position.col - area.min.col)] = self[position];
        }

        next
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.resize_with(width, height, Cell::default());
    }

    pub fn resize_with(&mut self, width: usize, height: usize, value: Cell) {
        let (current_width, current_height) = (self.width(), self.height());
        if current_width == width && current_height == height {
            return;
        }


        if width != current_width {
            let copy_w = width.min(current_width);

            if width > current_width {
                // Growing: extend first, then shift rows back-to-front
                for y in (1..current_height).rev() {
                    let src = y * current_width;
                    let dst = y * width;
                    self.copy_within(src..src + copy_w, dst);
                    // Fill the new columns
                    &mut self[dst + copy_w..dst + width].fill(value);
                }
                // Row 0: just fill the tail
                &mut self[copy_w..width].fill(value);
            } else {
                // Shrinking: shift rows front-to-back, then truncate
                for y in 1..current_height {
                    let src = y * current_width;
                    let dst = y * width;
                    self.copy_within(src..src + copy_w, dst);
                }
                self.truncate(width * current_height);
            }
        }

        if height > current_height {
            self.data.resize(width * height, value);
        } else if height < current_height {
            self.truncate(width * height);
        }

        self.width = width;
        self.height = height;
    }

    pub fn resize_inner(&mut self, width: usize, height: usize) {
        self.data.reserve(width * height - self.len());
        self.data.fill(Cell::default());
    }


    pub fn iter_col(&self, col: usize) -> StepBy<Iter<Cell>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );

        self.data[col..].iter().step_by(self.width)
    }

    pub fn iter_col_mut(&mut self, col: usize) -> StepBy<IterMut<Cell>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );
        self.data[col..].iter_mut().step_by(self.width)
    }

    pub fn iter_row(&self, row: usize) -> StepBy<Iter<Cell>> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        self[Row(row)].iter().step_by(1)
    }

    pub fn iter_row_mut(&mut self, row: usize) -> StepBy<IterMut<Cell>> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );

        self[Row(row)].iter_mut().step_by(1)
    }

    pub fn indexed_iter(&self) -> impl Iterator<Item = ((usize, usize), &Cell)> {
        self.iter().enumerate().map(move |(idx, i)| {
            ((idx / self.width, idx % self.width), i)
        })
    }

    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = ((usize, usize), &mut Cell)> {
        let cols = self.width;

        self.iter_mut().enumerate().map(move |(idx, i)| {
            ((idx / cols, idx % cols), i)
        })
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = StepBy<Iter<Cell>>> {
        (0..self.height).map(move |row| self.iter_row(row))
    }

    pub fn iter_cols(&self) -> impl Iterator<Item = StepBy<Iter<Cell>>> {
        (0..self.width).map(move |col| self.iter_col(col))
    }

    pub fn iter(&self) -> Iter<Cell> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Cell> {
        self.iter_mut()
    }

    pub fn to_string(&self, arena: &GraphemeArena) -> String {
        self.iter_rows().map(|row| {
            row.map(|cell| cell.as_str(arena)).collect::<String>()
        }).intersperse(String::from("\n")).collect()
    }
}

impl From<Rect> for Buffer {
    fn from(value: Rect) -> Self {
        Self::new(value.width(), value.height())
    }
}

impl From<Area> for Buffer {
    fn from(value: Area) -> Self {
        Self::new(value.width(), value.height())
    }
}

impl<I: IntoSliceIndex<Buffer, [Cell]>> Index<I> for Buffer {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        index.into_slice_index(self).index(self.as_ref())
    }
}

impl<I: IntoSliceIndex<Buffer, [Cell]>> IndexMut<I> for Buffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.into_slice_index(self).index_mut(self.as_mut())
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
impl Intersect<Area> for Buffer {
    type Output = Area;

    fn intersect(&self, other: &Area) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Area::ZERO;
        }

        let mut r = Area::ZERO;

        let x1 = 0.max(other.min.col);
        let y1 = 0.max(other.min.row);
        let x2 = self.width().min(other.max.col);
        let y2 = self.height().min(other.max.row);

        r.min.col = x1;
        r.min.row = y1;

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

        r.max.col = r.min.col + w;
        r.max.row = r.min.row + h;

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
impl Intersect<Buffer> for Area {
    type Output = Area;

    fn intersect(&self, other: &Buffer) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Area::ZERO;
        }

        let mut r = Area::ZERO;

        let x1 = self.min_x().max(other.min_x());
        let y1 = self.min_y().max(other.min_y());
        let x2 = self.width().min(other.max_x());
        let y2 = self.height().min(other.max_y());

        r.min.col = x1;
        r.min.row = y1;

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

        r.max.col = r.min.col + w;
        r.max.row = r.min.row + h;

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
                    self.data
                        .iter()
                        .map(|i| format!("{i:?}").len())
                        .max()
                        .unwrap()
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