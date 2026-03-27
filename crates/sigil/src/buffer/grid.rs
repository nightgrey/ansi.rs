#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
/*!
# Two Dimensional Grid
Continuous growable 2D data structure.
The purpose of this crate is to provide an universal data structure that is faster,
uses less memory, and is easier to use than a naive `Vec<Vec<T>>` solution.

This crate will always provide a 2D data structure. If you need three or more dimensions take a look at the
[ndarray](https://docs.rs/ndarray/0.13.0/ndarray/) library. The `grid` crate is a container for all kind of data.
If you need to perform matrix operations, you are better off with a linear algebra lib, such as
[cgmath](https://docs.rs/cgmath/0.17.0/cgmath/) or [nalgebra](https://docs.rs/nalgebra/0.21.0/nalgebra/).
No other dependencies except for the std lib are used.
Most of the functions `std::Vec<T>` offer are also implemented in `grid` and slightly modified for a 2D data object.

# Memory layout

Similar to *C-like* arrays, `grid` uses a flat 1D `Vec<T>` data structure to have a continuous
memory data layout. See also [this](https://stackoverflow.com/questions/17259877/1d-or-2d-array-whats-faster)
explanation of why you should probably use a one-dimensional array approach.

Note that this crate uses a [*row-major*](https://eli.thegreenplace.net/2015/memory-layout-of-multi-dimensional-arrays) memory layout by default.

If you need a specific memory layout, please seek the `*_with_order` constructors. You should also take note that some transformation methods
change the internal memory layout, like [`transpose`](Grid::transpose).

This choice is important, because operations on rows are faster with a row-major memory layout.
Likewise, operations on columns are faster with column-major memory layout.

# Examples
```
use grid::*;
let mut grid = grid![[1,2,3]
                     [4,5,6]];
assert_eq!(grid, Grid::from_vec(vec![1,2,3,4,5,6],3));
assert_eq!(grid.get(0, 2), Some(&3));
assert_eq!(grid[(1, 1)], 5);
assert_eq!(grid.dimensions(), (2, 3));
grid.push_row(vec![7,8,9]);
assert_eq!(grid, grid![[1,2,3][4,5,6][7,8,9]])
 ```
*/

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, vec, vec::Vec};
#[cfg(feature = "serde")]
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};

use core::cmp::Eq;
use core::fmt;
use core::hash;
use core::iter::StepBy;
use core::ops::Index;
use core::ops::IndexMut;
use core::slice::Iter;
use core::slice::IterMut;
use core::{cmp, convert::TryInto};
use derive_more::{Deref, DerefMut};
use geometry::{Point, PointLike, Rect, Resolve};
use std::ops;
use std::ops::RangeBounds;
use std::slice::SliceIndex;

/// Stores elements of a certain type in a 2D grid structure.
///
/// Uses a rust `Vec<T>` type to reference the grid data on the heap.
/// Also the internal memory layout as well as the number of
/// rows and columns are stored in the grid data structure.
///
/// The size limit of a grid is `rows * cols < usize`.
#[derive(Deref, DerefMut)]
pub struct Grid<T> {
    #[deref]
    #[deref_mut]
    data: Vec<T>,
    pub width: usize,
    pub height: usize,
    pub order: Order,
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Grid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::marker::PhantomData;
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Data,
            Cols,
            Order,
        }

        struct GridVisitor<T> {
            _p: PhantomData<T>,
        }

        impl<'de, T: Deserialize<'de>> Visitor<'de> for GridVisitor<T> {
            type Value = Grid<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Grid")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Grid<T>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let cols = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let data = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let order = seq.next_element()?.unwrap_or_default();
                Ok(Grid::from_vec_with_order(data, cols, order))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Grid<T>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut cols = None;
                let mut data = None;
                let mut order = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        Field::Cols => {
                            if cols.is_some() {
                                return Err(de::Error::duplicate_field("cols"));
                            }
                            cols = Some(map.next_value()?);
                        }
                        Field::Order => {
                            if order.is_some() {
                                return Err(de::Error::duplicate_field("order"));
                            }
                            order = Some(map.next_value()?);
                        }
                    }
                }
                let cols = cols.ok_or_else(|| de::Error::missing_field("cols"))?;
                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                let order = order.unwrap_or_default();
                Ok(Grid::from_vec_with_order(data, cols, order))
            }
        }

        const FIELDS: &[&str] = &["cols", "data", "order"];
        deserializer.deserialize_struct("Grid", FIELDS, GridVisitor { _p: PhantomData })
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for Grid<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Grid", 3)?;
        state.serialize_field("cols", &self.width)?;
        state.serialize_field("data", &self.data)?;
        state.serialize_field("order", &self.order)?;
        state.end()
    }
}

impl<T> Grid<T> {
    pub const EMPTY: Self = Self {
        data: Vec::new(),
        width: 0,
        height: 0,
        order: Order::default(),
    };
    #[must_use]
    #[inline]
    pub fn new(rows: usize, cols: usize) -> Self
    where
        T: Default,
    {
        Self::new_with_order(rows, cols, Order::default())
    }

    pub fn new_with_order(rows: usize, cols: usize, order: Order) -> Self
    where
        T: Default,
    {
        if rows == 0 || cols == 0 {
            return Self {
                data: Vec::new(),
                height: 0,
                width: 0,
                order,
            };
        }
        let mut data = Vec::new();
        data.resize_with(rows.checked_mul(cols).unwrap(), T::default);
        Self {
            data,
            width: cols,
            height: rows,
            order,
        }
    }

    #[inline]
    pub fn init(rows: usize, cols: usize, data: T) -> Self
    where
        T: Clone,
    {
        Self::init_with_order(rows, cols, Order::default(), data)
    }

    pub fn init_with_order(rows: usize, cols: usize, order: Order, data: T) -> Self
    where
        T: Clone,
    {
        if rows == 0 || cols == 0 {
            return Self {
                data: Vec::new(),
                height: 0,
                width: 0,
                order,
            };
        }
        Self {
            data: vec![data; rows.checked_mul(cols).unwrap()],
            width: cols,
            height: rows,
            order,
        }
    }

    #[must_use]
    pub fn with_capacity(rows: usize, cols: usize) -> Self {
        Self::with_capacity_and_order(rows, cols, Order::default())
    }

    #[must_use]
    pub fn with_capacity_and_order(rows: usize, cols: usize, order: Order) -> Self {
        Self {
            data: Vec::with_capacity(rows.checked_mul(cols).unwrap()),
            width: 0,
            height: 0,
            order,
        }
    }

    #[must_use]
    #[inline]
    pub fn from_vec(vec: Vec<T>, cols: usize) -> Self {
        Self::from_vec_with_order(vec, cols, Order::default())
    }

    #[must_use]
    pub fn from_vec_with_order(vec: Vec<T>, cols: usize, order: Order) -> Self {
        let rows = vec.len().checked_div(cols).unwrap_or(0);
        assert_eq!(
            rows * cols,
            vec.len(),
            "Vector length {:?} should be a multiple of cols = {:?}",
            vec.len(),
            cols
        );
        if rows == 0 || cols == 0 {
            Self {
                data: vec,
                height: 0,
                width: 0,
                order,
            }
        } else {
            Self {
                data: vec,
                height: rows,
                width: cols,
                order,
            }
        }
    }

    #[inline]
    #[must_use]
    const fn get_index(&self, row: usize, col: usize) -> usize {
        match self.order {
            Order::RowMajor => row * self.width + col,
            Order::ColumnMajor => col * self.height + row,
        }
    }

    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked(&self, row: impl Into<usize>, col: impl Into<usize>) -> &T {
        let index = self.get_index(row.into(), col.into());
        self.data.get_unchecked(index)
    }

    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked_mut(
        &mut self,
        row: impl Into<usize>,
        col: impl Into<usize>,
    ) -> &mut T {
        let index = self.get_index(row.into(), col.into());
        self.data.get_unchecked_mut(index)
    }

    #[must_use]
    pub fn get(&self, row: impl TryInto<usize>, col: impl TryInto<usize>) -> Option<&T> {
        let row_usize = row.try_into().ok()?;
        let col_usize = col.try_into().ok()?;
        if row_usize < self.height && col_usize < self.width {
            unsafe { Some(self.get_unchecked(row_usize, col_usize)) }
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_mut(
        &mut self,
        row: impl TryInto<usize>,
        col: impl TryInto<usize>,
    ) -> Option<&mut T> {
        let row_usize = row.try_into().ok()?;
        let col_usize = col.try_into().ok()?;
        if row_usize < self.height && col_usize < self.width {
            unsafe { Some(self.get_unchecked_mut(row_usize, col_usize)) }
        } else {
            None
        }
    }

    #[must_use]
    pub const fn dimensions(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    #[must_use]
    pub const fn rows(&self) -> usize {
        self.height
    }

    #[must_use]
    pub const fn cols(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn order(&self) -> Order {
        self.order
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.height = 0;
        self.width = 0;
        self.data.clear();
    }

    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.data.iter_mut()
    }

    pub fn iter_col(&self, col: usize) -> StepBy<Iter<T>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );
        match self.order {
            Order::RowMajor => self.data[col..].iter().step_by(self.width),
            Order::ColumnMajor => {
                let start = col * self.height;
                self.data[start..(start + self.height)].iter().step_by(1)
            }
        }
    }

    pub fn iter_col_mut(&mut self, col: usize) -> StepBy<IterMut<T>> {
        assert!(
            col < self.width,
            "out of bounds. Column must be less than {:?}, but is {:?}",
            self.width,
            col
        );
        match self.order {
            Order::RowMajor => self.data[col..].iter_mut().step_by(self.width),
            Order::ColumnMajor => {
                let start = col * self.height;
                self.data[start..(start + self.height)]
                    .iter_mut()
                    .step_by(1)
            }
        }
    }

    pub fn iter_row(&self, row: usize) -> StepBy<Iter<T>> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        match self.order {
            Order::RowMajor => {
                let start = row * self.width;
                self.data[start..(start + self.width)].iter().step_by(1)
            }
            Order::ColumnMajor => self.data[row..].iter().step_by(self.height),
        }
    }

    pub fn iter_row_mut(&mut self, row: usize) -> StepBy<IterMut<T>> {
        assert!(
            row < self.height,
            "out of bounds. Row must be less than {:?}, but is {:?}",
            self.height,
            row
        );
        match self.order {
            Order::RowMajor => {
                let start = row * self.width;
                self.data[start..(start + self.width)].iter_mut().step_by(1)
            }
            Order::ColumnMajor => self.data[row..].iter_mut().step_by(self.height),
        }
    }

    pub fn indexed_iter(&self) -> impl Iterator<Item = ((usize, usize), &T)> {
        self.data.iter().enumerate().map(move |(idx, i)| {
            let position = match self.order {
                Order::RowMajor => (idx / self.width, idx % self.width),
                Order::ColumnMajor => (idx % self.height, idx / self.height),
            };
            (position, i)
        })
    }

    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = ((usize, usize), &mut T)> {
        let order = self.order;
        let cols = self.width;
        let rows = self.height;

        self.data.iter_mut().enumerate().map(move |(idx, i)| {
            let position = match order {
                Order::RowMajor => (idx / cols, idx % cols),
                Order::ColumnMajor => (idx % rows, idx / rows),
            };
            (position, i)
        })
    }

    pub fn indexed_into_iter(self) -> impl Iterator<Item = ((usize, usize), T)> {
        let Self {
            data,
            width: cols,
            height: rows,
            order,
        } = self;
        data.into_iter().enumerate().map(move |(idx, i)| {
            let position = match order {
                Order::RowMajor => (idx / cols, idx % cols),
                Order::ColumnMajor => (idx % rows, idx / rows),
            };
            (position, i)
        })
    }

    pub fn push_row(&mut self, row: Vec<T>) {
        assert_ne!(row.len(), 0);
        assert!(
            !(self.height > 0 && row.len() != self.width),
            "pushed row does not match. Length must be {:?}, but was {:?}.",
            self.width,
            row.len()
        );
        self.data.extend(row);
        if self.order == Order::ColumnMajor {
            for i in (1..self.width).rev() {
                let col_idx = i * self.height;
                self.data[col_idx..col_idx + self.height + i].rotate_right(i);
            }
        }
        self.height += 1;
        if self.width == 0 {
            self.width = self.data.len();
        }
    }

    pub fn push_col(&mut self, col: Vec<T>) {
        assert_ne!(col.len(), 0);
        assert!(
            !(self.width > 0 && col.len() != self.height),
            "pushed column does not match. Length must be {:?}, but was {:?}.",
            self.height,
            col.len()
        );
        self.data.extend(col);
        if self.order == Order::RowMajor {
            for i in (1..self.height).rev() {
                let row_idx = i * self.width;
                self.data[row_idx..row_idx + self.width + i].rotate_right(i);
            }
        }
        self.width += 1;
        if self.height == 0 {
            self.height = self.data.len();
        }
    }

    pub fn pop_row(&mut self) -> Option<Vec<T>> {
        if self.height == 0 {
            return None;
        }
        if self.order == Order::ColumnMajor {
            for i in 1..self.width {
                let col_idx = i * (self.height - 1);
                self.data[col_idx..col_idx + self.height + i - 1].rotate_left(i);
            }
        }
        let row = self.data.split_off(self.data.len() - self.width);
        self.height -= 1;
        if self.height == 0 {
            self.width = 0;
        }
        Some(row)
    }

    pub fn remove_row(&mut self, row_index: usize) -> Option<Vec<T>> {
        if self.width == 0 || self.height == 0 || row_index >= self.height {
            return None;
        }
        let row = match self.order {
            Order::RowMajor => self
                .data
                .drain((row_index * self.width)..((row_index + 1) * self.width))
                .collect(),
            Order::ColumnMajor => {
                for i in 0..self.width {
                    let col_idx = row_index + i * (self.height - 1);
                    let end = cmp::min(col_idx + self.height + i, self.data.len());
                    self.data[col_idx..end].rotate_left(i + 1);
                }
                self.data.split_off(self.data.len() - self.width)
            }
        };
        self.height -= 1;
        if self.height == 0 {
            self.width = 0;
        }
        Some(row)
    }

    pub fn pop_col(&mut self) -> Option<Vec<T>> {
        if self.width == 0 {
            return None;
        }
        if self.order == Order::RowMajor {
            for i in 1..self.height {
                let row_idx = i * (self.width - 1);
                self.data[row_idx..row_idx + self.width + i - 1].rotate_left(i);
            }
        }
        let col = self.data.split_off(self.data.len() - self.height);
        self.width -= 1;
        if self.width == 0 {
            self.height = 0;
        }
        Some(col)
    }

    pub fn remove_col(&mut self, col_index: usize) -> Option<Vec<T>> {
        if self.width == 0 || self.height == 0 || col_index >= self.width {
            return None;
        }
        let col = match self.order {
            Order::RowMajor => {
                for i in 0..self.height {
                    let row_idx = col_index + i * (self.width - 1);
                    let end = cmp::min(row_idx + self.width + i, self.data.len());
                    self.data[row_idx..end].rotate_left(i + 1);
                }
                self.data.split_off(self.data.len() - self.height)
            }
            Order::ColumnMajor => self
                .data
                .drain((col_index * self.height)..((col_index + 1) * self.height))
                .collect(),
        };
        self.width -= 1;
        if self.width == 0 {
            self.height = 0;
        }
        Some(col)
    }

    pub fn insert_row(&mut self, index: usize, row: Vec<T>) {
        let input_len = row.len();
        assert!(
            !(self.width > 0 && input_len != self.width),
            "Inserted row must be of length {}, but was {}.",
            self.width,
            row.len()
        );
        assert!(
            index <= self.height,
            "Out of range. Index was {}, but must be less or equal to {}.",
            index,
            self.height
        );
        match self.order {
            Order::RowMajor => {
                let data_idx = index * input_len;
                self.data.splice(data_idx..data_idx, row);
            }
            Order::ColumnMajor => {
                for (col_iter, row_val) in row.into_iter().enumerate() {
                    let data_idx = col_iter * self.height + index + col_iter;
                    self.data.insert(data_idx, row_val);
                }
            }
        }
        self.width = input_len;
        self.height += 1;
    }

    pub fn insert_col(&mut self, index: usize, col: Vec<T>) {
        let input_len = col.len();
        assert!(
            !(self.height > 0 && input_len != self.height),
            "Inserted col must be of length {}, but was {}.",
            self.height,
            col.len()
        );
        assert!(
            index <= self.width,
            "Out of range. Index was {}, but must be less or equal to {}.",
            index,
            self.width
        );
        match self.order {
            Order::RowMajor => {
                for (row_iter, col_val) in col.into_iter().enumerate() {
                    let data_idx = row_iter * self.width + index + row_iter;
                    self.data.insert(data_idx, col_val);
                }
            }
            Order::ColumnMajor => {
                let data_idx = index * input_len;
                self.data.splice(data_idx..data_idx, col);
            }
        }
        self.height = input_len;
        self.width += 1;
    }

    #[must_use]
    pub const fn flatten(&self) -> &Vec<T> {
        &self.data
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<T> {
        self.data
    }

    pub fn transpose(&mut self) {
        self.order = self.order.counterpart();
        core::mem::swap(&mut self.height, &mut self.width);
    }

    pub fn flip_cols(&mut self) {
        match self.order {
            Order::RowMajor => {
                for row in 0..self.height {
                    let idx = row * self.width;
                    self.data[idx..idx + self.width].reverse();
                }
            }
            Order::ColumnMajor => {
                for col in 0..self.width / 2 {
                    for row in 0..self.height {
                        let cell1 = self.get_index(row, col);
                        let cell2 = self.get_index(row, self.width - col - 1);
                        self.data.swap(cell1, cell2);
                    }
                }
            }
        }
    }

    pub fn flip_rows(&mut self) {
        match self.order {
            Order::RowMajor => {
                for row in 0..self.height / 2 {
                    for col in 0..self.width {
                        let cell1 = self.get_index(row, col);
                        let cell2 = self.get_index(self.height - row - 1, col);
                        self.data.swap(cell1, cell2);
                    }
                }
            }
            Order::ColumnMajor => {
                for col in 0..self.width {
                    let idx = col * self.height;
                    self.data[idx..idx + self.height].reverse();
                }
            }
        }
    }

    pub fn rotate_left(&mut self) {
        self.transpose();
        self.flip_rows();
    }

    pub fn rotate_right(&mut self) {
        self.transpose();
        self.flip_cols();
    }

    pub fn rotate_half(&mut self) {
        self.data.reverse();
    }

    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }

    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> T,
    {
        self.data.fill_with(f);
    }

    pub fn map<U, F>(self, f: F) -> Grid<U>
    where
        F: FnMut(T) -> U,
    {
        Grid {
            data: self.data.into_iter().map(f).collect(),
            width: self.width,
            height: self.height,
            order: self.order,
        }
    }

    #[must_use]
    pub fn map_ref<U, F>(&self, f: F) -> Grid<U>
    where
        F: Fn(&T) -> U,
    {
        Grid {
            data: self.data.iter().map(f).collect(),
            width: self.width,
            height: self.height,
            order: self.order,
        }
    }

    #[must_use]
    pub const fn iter_rows(&self) -> GridRowIter<'_, T> {
        GridRowIter {
            grid: self,
            row_start_index: 0,
            row_end_index: self.height,
        }
    }

    #[must_use]
    pub const fn iter_cols(&self) -> GridColIter<'_, T> {
        GridColIter {
            grid: self,
            col_start_index: 0,
            col_end_index: self.width,
        }
    }

    pub fn swap(&mut self, (row_a, col_a): (usize, usize), (row_b, col_b): (usize, usize)) {
        assert!(
            !(row_a >= self.height || col_a >= self.width),
            "grid index out of bounds: ({row_a},{col_a}) out of ({},{})",
            self.height,
            self.width
        );
        assert!(
            !(row_b >= self.height || col_b >= self.width),
            "grid index out of bounds: ({row_b},{col_b}) out of ({},{})",
            self.height,
            self.width
        );

        let a_idx = self.get_index(row_a, col_a);
        let b_idx = self.get_index(row_b, col_b);

        self.data.swap(a_idx, b_idx);
    }

    pub fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dest: usize)
    where
        T: Copy,
    {
        self.data.copy_within(src, dest);
    }

    pub fn copy_from_slice(&mut self, other: &[T])
    where
        T: Copy,
    {
        self.data.copy_from_slice(other);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.data.resize(new_len, value);
    }
}

impl<T> AsRef<[T]> for Grid<T> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T> AsMut<[T]> for Grid<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}
impl<T: Default> Grid<T> {
    pub fn expand_rows(&mut self, rows: usize) {
        if rows > 0 && self.width > 0 {
            self.data
                .resize_with(self.data.len() + rows * self.width, T::default);

            if self.order == Order::ColumnMajor {
                for row_added in 0..rows {
                    for i in (1..self.width).rev() {
                        let total_rows = self.height + row_added;
                        let col_idx = i * total_rows;
                        self.data[col_idx..col_idx + total_rows + i].rotate_right(i);
                    }
                }
            }
            self.height += rows;
        }
    }

    pub fn expand_cols(&mut self, cols: usize) {
        if cols > 0 && self.height > 0 {
            self.data
                .resize_with(self.data.len() + cols * self.height, T::default);

            if self.order == Order::RowMajor {
                for col_added in 0..cols {
                    for i in (1..self.height).rev() {
                        let total_cols = self.width + col_added;
                        let row_idx = i * total_cols;
                        self.data[row_idx..row_idx + total_cols + i].rotate_right(i);
                    }
                }
            }
            self.width += cols;
        }
    }

    pub fn prepend_rows(&mut self, rows: usize) {
        if rows > 0 && self.width > 0 {
            self.data
                .resize_with(self.data.len() + rows * self.width, T::default);

            match self.order {
                Order::RowMajor => {
                    for i in (0..self.height).rev() {
                        let s = i * self.width;
                        let e = (i + rows + 1) * self.width;
                        self.data[s..e].rotate_left(self.width);
                    }
                }
                Order::ColumnMajor => {
                    for row_added in 0..rows {
                        for i in (0..self.width).rev() {
                            let total_rows = self.height + row_added;
                            let col_idx = i * total_rows;
                            self.data[col_idx..=col_idx + total_rows + i].rotate_right(i + 1);
                        }
                    }
                }
            }
            self.height += rows;
        }
    }

    pub fn prepend_cols(&mut self, cols: usize) {
        if cols > 0 && self.height > 0 {
            self.data
                .resize_with(self.data.len() + cols * self.height, T::default);

            match self.order {
                Order::RowMajor => {
                    for col_added in 0..cols {
                        for i in (0..self.height).rev() {
                            let total_cols = self.width + col_added;
                            let row_idx = i * total_cols;
                            self.data[row_idx..=row_idx + total_cols + i].rotate_right(i + 1);
                        }
                    }
                }
                Order::ColumnMajor => {
                    for i in (0..self.width).rev() {
                        let s = i * self.height;
                        let e = (i + cols + 1) * self.height;
                        self.data[s..e].rotate_left(self.height);
                    }
                }
            }
            self.width += cols;
        }
    }
}

impl<T> Default for Grid<T> {
    fn default() -> Self {
        Self {
            data: Vec::default(),
            width: 0,
            height: 0,
            order: Order::default(),
        }
    }
}

impl<T: Clone> Clone for Grid<T> {
    fn clone(&self) -> Self {
        Self {
            height: self.height,
            width: self.width,
            data: self.data.clone(),
            order: self.order,
        }
    }
}

impl<T: hash::Hash> hash::Hash for Grid<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.height.hash(state);
        self.width.hash(state);
        self.order.hash(state);
        self.data.hash(state);
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }
}

impl<T> Index<ops::Range<usize>> for Grid<T> {
    type Output = [T];
    #[inline]
    fn index(&self, range: ops::Range<usize>) -> &[T] {
        &self.data[range]
    }
}
impl<T> IndexMut<ops::Range<usize>> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, range: ops::Range<usize>) -> &mut [T] {
        &mut self.data[range]
    }
}
impl<T> Index<ops::RangeTo<usize>> for Grid<T> {
    type Output = [T];
    #[inline]
    fn index(&self, range: ops::RangeTo<usize>) -> &[T] {
        &self.data[range]
    }
}
impl<T> IndexMut<ops::RangeTo<usize>> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, range: ops::RangeTo<usize>) -> &mut [T] {
        &mut self.data[range]
    }
}
impl<T> Index<ops::RangeFrom<usize>> for Grid<T> {
    type Output = [T];
    #[inline]
    fn index(&self, range: ops::RangeFrom<usize>) -> &[T] {
        &self.data[range]
    }
}
impl<T> IndexMut<ops::RangeFrom<usize>> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, range: ops::RangeFrom<usize>) -> &mut [T] {
        &mut self.data[range]
    }
}
impl<T> Index<ops::RangeFull> for Grid<T> {
    type Output = [T];
    #[inline]
    fn index(&self, _: ops::RangeFull) -> &[T] {
        &self.data
    }
}
impl<T> IndexMut<ops::RangeFull> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, _: ops::RangeFull) -> &mut [T] {
        &mut self.data
    }
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, (row, col): (usize, usize)) -> &T {
        assert!(
            !(row >= self.height || col >= self.width),
            "grid index out of bounds: ({row},{col}) out of ({},{})",
            self.height,
            self.width
        );
        let index = self.get_index(row, col);
        &self.data[index]
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut T {
        assert!(
            !(row >= self.height || col >= self.width),
            "grid index out of bounds: ({row},{col}) out of ({},{})",
            self.height,
            self.width
        );
        let index = self.get_index(row, col);
        &mut self.data[index]
    }
}

impl<T> IntoIterator for Grid<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Grid<T> {
    type IntoIter = core::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type IntoIter = core::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for Grid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<T: PartialEq> PartialEq for Grid<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.height != other.height || self.width != other.width {
            return false;
        }
        if self.order == other.order {
            return self.data == other.data;
        }
        for (self_row, other_row) in core::iter::zip(self.iter_rows(), other.iter_rows()) {
            if self_row.ne(other_row) {
                return false;
            }
        }
        true
    }
}

impl<T: Eq> Eq for Grid<T> {}

impl<T> From<Vec<Vec<T>>> for Grid<T> {
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn from(vec: Vec<Vec<T>>) -> Self {
        let cols = vec.first().map_or(0, |row| row.len());
        Self::from_vec_with_order(vec.into_iter().flatten().collect(), cols, Order::default())
    }
}

impl<T: Clone> From<&Vec<Vec<T>>> for Grid<T> {
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn from(vec: &Vec<Vec<T>>) -> Self {
        let cols = vec.first().map_or(0, |row| row.len());
        Self::from_vec_with_order(
            vec.clone().into_iter().flatten().collect(),
            cols,
            Order::default(),
        )
    }
}

impl<T: Clone> From<&Vec<&Vec<T>>> for Grid<T> {
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn from(vec: &Vec<&Vec<T>>) -> Self {
        let cols = vec.first().map_or(0, |row| row.len());
        Self::from_vec_with_order(
            vec.clone()
                .into_iter()
                .flat_map(|inner| inner.clone())
                .collect(),
            cols,
            Order::default(),
        )
    }
}

impl<T> From<(Vec<T>, usize)> for Grid<T> {
    fn from(value: (Vec<T>, usize)) -> Self {
        Self::from_vec_with_order(value.0, value.1, Order::default())
    }
}

impl<T: Clone> From<(&Vec<T>, usize)> for Grid<T> {
    fn from(value: (&Vec<T>, usize)) -> Self {
        Self::from_vec_with_order(value.0.clone(), value.1, Order::default())
    }
}

impl<T: Clone> From<(&Vec<T>, &usize)> for Grid<T> {
    fn from(value: (&Vec<T>, &usize)) -> Self {
        Self::from_vec_with_order(value.0.clone(), *value.1, Order::default())
    }
}

#[derive(Clone)]
pub struct GridRowIter<'a, T> {
    grid: &'a Grid<T>,
    row_start_index: usize,
    row_end_index: usize,
}

#[derive(Clone)]
pub struct GridColIter<'a, T> {
    grid: &'a Grid<T>,
    col_start_index: usize,
    col_end_index: usize,
}

impl<'a, T> Iterator for GridRowIter<'a, T> {
    type Item = StepBy<Iter<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_start_index >= self.row_end_index {
            return None;
        }

        let row_iter = self.grid.iter_row(self.row_start_index);
        self.row_start_index += 1;
        Some(row_iter)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.row_end_index - self.row_start_index;
        (size, Some(size))
    }
}

impl<T> ExactSizeIterator for GridRowIter<'_, T> {}

impl<T> DoubleEndedIterator for GridRowIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.row_start_index >= self.row_end_index {
            return None;
        }

        let row_iter = self.grid.iter_row(self.row_end_index - 1);
        self.row_end_index -= 1;
        Some(row_iter)
    }
}

impl<'a, T> Iterator for GridColIter<'a, T> {
    type Item = StepBy<Iter<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col_start_index >= self.col_end_index {
            return None;
        }

        let col_iter = self.grid.iter_col(self.col_start_index);
        self.col_start_index += 1;
        Some(col_iter)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.col_end_index - self.col_start_index;
        (size, Some(size))
    }
}

impl<T> ExactSizeIterator for GridColIter<'_, T> {}

impl<T> DoubleEndedIterator for GridColIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.col_start_index >= self.col_end_index {
            return None;
        }

        let col_iter = self.grid.iter_col(self.col_end_index - 1);
        self.col_end_index -= 1;
        Some(col_iter)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

/// Init a grid with values.
///
/// Each array within `[]` represents a row starting from top to button.
///
/// # Examples
///
/// In this example a grid of numbers from 1 to 9 is created:
///
///
/// ```
/// use grid::grid;
/// let grid = grid![[1, 2, 3]
/// [4, 5, 6]
/// [7, 8, 9]];
/// assert_eq!(grid.dimensions(), (3, 3))
/// ```
///
/// Note that each row must be of the same length. The following example will not compile:
///
/// ``` ignore
/// use grid::grid;
/// let grid = grid![[1, 2, 3]
/// [4, 5] // This does not work!
/// [7, 8, 9]];
/// ```
#[macro_export]
macro_rules! grid {
    () => {
        $crate::Grid::from_vec(vec![], 0)
    };
    ( [$( $x:expr ),* ]) => { {
        let vec = vec![$($x),*];
        let len  = vec.len();
        $crate::Grid::from_vec(vec, len)
    } };
    ( [$( $x0:expr ),*] $([$( $x:expr ),*])* ) => {
        {
            let mut _assert_width0 = [(); $crate::count!($($x0)*)];
            let cols = $crate::count!($($x0)*);
            let rows = 1usize;

            $(
                let _assert_width = [(); $crate::count!($($x)*)];
                _assert_width0 = _assert_width;
                let rows = rows + 1usize;
            )*

            let mut vec = Vec::with_capacity(rows.checked_mul(cols).unwrap());

            $( vec.push($x0); )*
            $( $( vec.push($x); )* )*

            $crate::Grid::from_vec(vec, cols)
        }
    };
}

/// Init a column-major grid with values.
///
/// Each array within `[]` represents a row starting from top to button.
///
/// # Examples
///
/// In this example a grid of numbers from 1 to 9 is created:
///
/// ```
/// use grid::grid_cm;
/// let grid = grid_cm![[1, 2, 3]
/// [4, 5, 6]
/// [7, 8, 9]];
/// assert_eq!(grid.dimensions(), (3, 3));
/// assert_eq!(grid[(1, 1)], 5);
/// ```
///
/// Note that each row must be of the same length. The following example will not compile:
///
/// ``` ignore
/// use grid::grid_cm;
/// let grid = grid_cm![[1, 2, 3]
/// [4, 5] // This does not work!
/// [7, 8, 9]];
/// ```
#[macro_export]
macro_rules! grid_cm {
    () => {
        $crate::Grid::from_vec_with_order(vec![], 0, $crate::Order::ColumnMajor)
    };
    ( [$( $x:expr ),* ]) => { {
        let vec = vec![$($x),*];
        let len  = vec.len();
        $crate::Grid::from_vec_with_order(vec, len, $crate::Order::ColumnMajor)
    } };
    ( [$( $x0:expr ),*] $([$( $x:expr ),*])* ) => {
        {
            let mut _assert_width0 = [(); $crate::count!($($x0)*)];
            let cols = $crate::count!($($x0)*);
            let rows = 1usize;

            $(
                let _assert_width = [(); $crate::count!($($x)*)];
                _assert_width0 = _assert_width;
                let rows = rows + 1usize;
            )*

            let vec = Vec::with_capacity(rows.checked_mul(cols).unwrap());
            let mut grid = $crate::Grid::from_vec_with_order(vec, cols, $crate::Order::ColumnMajor);

            grid.push_row(vec![$($x0),*]);
            $( grid.push_row(vec![$($x),*]); )*

            grid
        }
    };
}

/// Define the internal memory layout of the grid.
#[derive_const(Clone, Default, PartialEq, Eq)]
#[derive(Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Order {
    #[default]
    RowMajor,

    ColumnMajor,
}

impl Order {
    const fn counterpart(self) -> Self {
        match self {
            Self::RowMajor => Self::ColumnMajor,
            Self::ColumnMajor => Self::RowMajor,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;

    fn test_grid<T>(grid: &Grid<T>, rows: usize, cols: usize, order: Order, data: &[T])
    where
        T: fmt::Debug + PartialEq,
    {
        assert_eq!(grid.height, rows, "number of rows is unexpected");
        assert_eq!(grid.width, cols, "number of cols is unexpected");
        assert_eq!(grid.order, order, "grid order is unexpected");
        assert_eq!(grid.data, data, "internal data is unexpected");
    }

    #[test]
    fn from_1d_vec() {
        let grid: Grid<u8> = Grid::from((vec![1, 2, 3], 1));
        test_grid(&grid, 3, 1, Order::RowMajor, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_1d_vec_panic() {
        let _: Grid<u8> = Grid::from((vec![1, 2, 3], 2));
    }

    #[test]
    fn from_1d_vec_reference() {
        let vec = vec![1, 2, 3];
        let grid: Grid<u8> = Grid::from((&vec, 1));
        test_grid(&grid, 3, 1, Order::RowMajor, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_1d_vec_reference_panic() {
        let vec = vec![1, 2, 3];
        let _: Grid<u8> = Grid::from((&vec, 2));
    }

    #[test]
    fn from_1d_vec_reference_and_reference() {
        let vec = vec![1, 2, 3];
        let cols = 1;
        let grid: Grid<u8> = Grid::from((&vec, &cols));
        test_grid(&grid, 3, 1, Order::RowMajor, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_1d_vec_reference_and_reference_panic() {
        let vec = vec![1, 2, 3];
        let cols = 2;
        let _: Grid<u8> = Grid::from((&vec, &cols));
    }

    #[test]
    fn from_2d_vec() {
        let grid: Grid<u8> = Grid::from(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);
        test_grid(&grid, 3, 3, Order::RowMajor, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_2d_vec_panic() {
        let _: Grid<u8> = Grid::from(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8]]);
    }

    #[test]
    fn from_2d_vec_reference() {
        let vec = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let grid: Grid<u8> = Grid::from(&vec);
        test_grid(&grid, 3, 3, Order::RowMajor, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_2d_vec_reference_panic() {
        let vec = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8]];
        let _: Grid<u8> = Grid::from(&vec);
    }

    #[test]
    fn from_2d_vec_reference_of_references() {
        let inner_vec1 = vec![1, 2, 3];
        let inner_vec2 = vec![4, 5, 6];
        let inner_vec3 = vec![7, 8, 9];
        let vec = vec![&inner_vec1, &inner_vec2, &inner_vec3];
        let grid: Grid<u8> = Grid::from(&vec);
        test_grid(&grid, 3, 3, Order::RowMajor, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_2d_vec_reference_of_references_panic() {
        let inner_vec1 = vec![1, 2, 3];
        let inner_vec2 = vec![4, 5, 6];
        let inner_vec3 = vec![7, 8];
        let vec = vec![&inner_vec1, &inner_vec2, &inner_vec3];
        let _: Grid<u8> = Grid::from(&vec);
    }

    #[test]
    fn from_vec_zero_with_cols() {
        let grid: Grid<u8> = Grid::from_vec(vec![], 1);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn from_vec_zero() {
        let grid: Grid<u8> = Grid::from_vec(vec![], 0);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_vec_panics_1() {
        let _: Grid<u8> = Grid::from_vec(vec![1, 2, 3], 0);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_vec_panics_2() {
        let _: Grid<u8> = Grid::from_vec(vec![1, 2, 3], 2);
    }

    #[test]
    fn from_vec_uses_original_vec() {
        let capacity = 10_000_000;
        let vec = Vec::with_capacity(capacity);
        let grid: Grid<u8> = Grid::from_vec(vec, 0);
        assert!(grid.into_inner().capacity() >= capacity);
    }

    #[test]
    fn from_vec_with_order_zero_with_cols() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![], 1, Order::ColumnMajor);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn from_vec_with_order_zero() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_vec_with_order_panics_1() {
        let _: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3], 0, Order::ColumnMajor);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn from_vec_with_order_panics_2() {
        let _: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3], 2, Order::ColumnMajor);
    }

    #[test]
    fn from_vec_with_order_uses_original_vec() {
        let capacity = 10_000_000;
        let vec = Vec::with_capacity(capacity);
        let grid: Grid<u8> = Grid::from_vec_with_order(vec, 0, Order::ColumnMajor);
        assert!(grid.into_inner().capacity() >= capacity);
    }

    #[test]
    fn insert_col_at_end() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.insert_col(2, vec![5, 6]);
        test_grid(&grid, 2, 3, Order::RowMajor, &[1, 2, 5, 3, 4, 6]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn insert_col_out_of_idx() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.insert_col(3, vec![4, 5]);
    }

    #[test]
    fn insert_col_empty() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        grid.insert_col(0, vec![1, 2, 3]);
        test_grid(&grid, 3, 1, Order::RowMajor, &[1, 2, 3]);
    }

    #[test]
    fn insert_col_at_end_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.insert_col(2, vec![5, 6]);
        test_grid(&grid, 2, 3, Order::ColumnMajor, &[1, 3, 2, 4, 5, 6]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn insert_col_out_of_idx_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.insert_col(3, vec![4, 5]);
    }

    #[test]
    fn insert_col_empty_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        grid.insert_col(0, vec![1, 2, 3]);
        test_grid(&grid, 3, 1, Order::ColumnMajor, &[1, 2, 3]);
    }

    #[test]
    fn insert_row_at_end() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.insert_row(2, vec![5, 6]);
        test_grid(&grid, 3, 2, Order::RowMajor, &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn insert_row_empty() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        grid.insert_row(0, vec![1, 2, 3]);
        test_grid(&grid, 1, 3, Order::RowMajor, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn insert_row_out_of_idx() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.insert_row(3, vec![4, 5]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn insert_row_wrong_size_of_idx() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.insert_row(1, vec![4, 5, 4]);
    }

    #[test]
    fn insert_row_start() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.insert_row(1, vec![5, 6]);
        test_grid(&grid, 3, 2, Order::RowMajor, &[1, 2, 5, 6, 3, 4]);
    }

    #[test]
    fn insert_row_at_end_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.insert_row(2, vec![5, 6]);
        test_grid(&grid, 3, 2, Order::ColumnMajor, &[1, 3, 5, 2, 4, 6]);
    }

    #[test]
    fn insert_row_empty_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        grid.insert_row(0, vec![1, 2, 3]);
        test_grid(&grid, 1, 3, Order::ColumnMajor, &[1, 2, 3]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn insert_row_out_of_idx_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::ColumnMajor);
        grid.insert_row(3, vec![4, 5]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn insert_row_wrong_size_of_idx_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::ColumnMajor);
        grid.insert_row(1, vec![4, 5, 4]);
    }

    #[test]
    fn insert_row_start_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.insert_row(1, vec![5, 6]);
        test_grid(&grid, 3, 2, Order::ColumnMajor, &[1, 5, 3, 2, 6, 4]);
    }

    #[test]
    fn pop_col_1x3() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3], 3, Order::RowMajor);
        assert_eq!(grid.pop_col(), Some(vec![3]));
        test_grid(&grid, 1, 2, Order::RowMajor, &[1, 2]);
        assert_eq!(grid.pop_col(), Some(vec![2]));
        test_grid(&grid, 1, 1, Order::RowMajor, &[1]);
        assert_eq!(grid.pop_col(), Some(vec![1]));
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_col_3x1() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3], 1, Order::RowMajor);
        assert_eq!(grid.pop_col(), Some(vec![1, 2, 3]));
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_col_2x2() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        assert_eq!(grid.pop_col(), Some(vec![2, 4]));
        assert_eq!(grid.dimensions(), (2, 1));
        test_grid(&grid, 2, 1, Order::RowMajor, &[1, 3]);
        assert_eq!(grid.pop_col(), Some(vec![1, 3]));
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_col_3x4() {
        let internal = vec![1, 2, 3, 4, 11, 22, 33, 44, 111, 222, 333, 444];
        let mut grid: Grid<u16> = Grid::from_vec_with_order(internal, 4, Order::RowMajor);
        assert_eq!(grid.pop_col(), Some(vec![4, 44, 444]));
        let expected = [1, 2, 3, 11, 22, 33, 111, 222, 333];
        test_grid(&grid, 3, 3, Order::RowMajor, &expected);
        assert_eq!(grid.pop_col(), Some(vec![3, 33, 333]));
        test_grid(&grid, 3, 2, Order::RowMajor, &[1, 2, 11, 22, 111, 222]);
        assert_eq!(grid.pop_col(), Some(vec![2, 22, 222]));
        test_grid(&grid, 3, 1, Order::RowMajor, &[1, 11, 111]);
        assert_eq!(grid.pop_col(), Some(vec![1, 11, 111]));
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_col_empty() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_col_1x3_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3], 3, Order::ColumnMajor);
        assert_eq!(grid.pop_col(), Some(vec![3]));
        test_grid(&grid, 1, 2, Order::ColumnMajor, &[1, 2]);
        assert_eq!(grid.pop_col(), Some(vec![2]));
        test_grid(&grid, 1, 1, Order::ColumnMajor, &[1]);
        assert_eq!(grid.pop_col(), Some(vec![1]));
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn pop_col_3x1_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 2, 3], 1, Order::ColumnMajor);
        assert_eq!(grid.pop_col(), Some(vec![1, 2, 3]));
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn pop_col_2x2_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        assert_eq!(grid.pop_col(), Some(vec![2, 4]));
        assert_eq!(grid.dimensions(), (2, 1));
        test_grid(&grid, 2, 1, Order::ColumnMajor, &[1, 3]);
        assert_eq!(grid.pop_col(), Some(vec![1, 3]));
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn pop_col_3x4_column_major() {
        let internal = vec![1, 11, 111, 2, 22, 222, 3, 33, 333, 4, 44, 444];
        let mut grid: Grid<u16> = Grid::from_vec_with_order(internal, 4, Order::ColumnMajor);
        assert_eq!(grid.pop_col(), Some(vec![4, 44, 444]));
        let expected = [1, 11, 111, 2, 22, 222, 3, 33, 333];
        test_grid(&grid, 3, 3, Order::ColumnMajor, &expected);
        assert_eq!(grid.pop_col(), Some(vec![3, 33, 333]));
        test_grid(&grid, 3, 2, Order::ColumnMajor, &[1, 11, 111, 2, 22, 222]);
        assert_eq!(grid.pop_col(), Some(vec![2, 22, 222]));
        test_grid(&grid, 3, 1, Order::ColumnMajor, &[1, 11, 111]);
        assert_eq!(grid.pop_col(), Some(vec![1, 11, 111]));
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn pop_col_empty_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        assert_eq!(grid.pop_col(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn pop_row_2x2() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        assert_eq!(grid.pop_row(), Some(vec![3, 4]));
        test_grid(&grid, 1, 2, Order::RowMajor, &[1, 2]);
        assert_eq!(grid.pop_row(), Some(vec![1, 2]));
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq!(grid.pop_row(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_row_empty() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![], 0);
        assert_eq!(grid.pop_row(), None);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn pop_row_2x2_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        assert_eq!(grid.pop_row(), Some(vec![3, 4]));
        test_grid(&grid, 1, 2, Order::ColumnMajor, &[1, 2]);
        assert_eq!(grid.pop_row(), Some(vec![1, 2]));
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq!(grid.pop_row(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn pop_row_empty_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        assert_eq!(grid.pop_row(), None);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn ne_full_empty() {
        let g1 = Grid::from_vec(vec![1, 2, 3, 4], 2);
        let g2: Grid<u8> = grid![];
        assert_ne!(g1, g2);
    }

    #[test]
    fn ne() {
        let g1 = Grid::from_vec(vec![1, 2, 3, 5], 2);
        let g2 = Grid::from_vec(vec![1, 2, 3, 4], 2);
        assert_ne!(g1, g2);
    }

    #[test]
    fn ne_dif_rows() {
        let g1 = Grid::from_vec(vec![1, 2, 3, 4], 2);
        let g2 = Grid::from_vec(vec![1, 2, 3, 4], 1);
        assert_ne!(g1, g2);
    }

    #[test]
    fn equal_empty() {
        let grid: Grid<char> = grid![];
        let grid2: Grid<char> = grid![];
        assert_eq!(grid, grid2);
    }

    #[test]
    fn equal() {
        let grid: Grid<char> = grid![['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']];
        let grid2: Grid<char> = grid![['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']];
        assert_eq!(grid, grid2);
    }

    #[test]
    fn equal_different_order() {
        let grid = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        let grid2 = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        assert_eq!(grid, grid2);
    }

    #[test]
    fn equal_partial_eq() {
        let grid = grid![[1.0]];
        let grid2 = Grid::from_vec(vec![1.0], 1);
        assert_eq!(grid, grid2);
    }

    #[test]
    fn ne_partial_eq() {
        let grid = grid![[f64::NAN]];
        assert_ne!(grid, grid);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn idx_tup_out_of_col_bounds() {
        let grid: Grid<char> = grid![['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']];
        let _ = grid[(0, 5)];
    }

    #[test]
    fn push_col_2x3() {
        let mut grid: Grid<u8> = grid![
                    [0, 1, 2]
                    [10, 11, 12]];
        grid.push_col(vec![3, 13]);
        test_grid(&grid, 2, 4, Order::RowMajor, &[0, 1, 2, 3, 10, 11, 12, 13]);
    }

    #[test]
    fn push_col_3x4() {
        let mut grid: Grid<char> = grid![
                    ['a', 'b', 'c', 'd']
                    ['a', 'b', 'c', 'd']
                    ['a', 'b', 'c', 'd']];
        grid.push_col(vec!['x', 'y', 'z']);
        let expected = [
            'a', 'b', 'c', 'd', 'x', 'a', 'b', 'c', 'd', 'y', 'a', 'b', 'c', 'd', 'z',
        ];
        test_grid(&grid, 3, 5, Order::RowMajor, &expected);
    }

    #[test]
    fn push_col_1x3() {
        let mut grid: Grid<char> = grid![['a', 'b', 'c']];
        grid.push_col(vec!['d']);
        test_grid(&grid, 1, 4, Order::RowMajor, &['a', 'b', 'c', 'd']);
    }

    #[test]
    fn push_col_empty() {
        let mut grid: Grid<char> = grid![];
        grid.push_col(vec!['b', 'b', 'b', 'b']);
        test_grid(&grid, 4, 1, Order::RowMajor, &['b', 'b', 'b', 'b']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_col_wrong_size() {
        let mut grid: Grid<char> = grid![['a','a','a']['a','a','a']];
        grid.push_col(vec!['b']);
        grid.push_col(vec!['b', 'b']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_col_zero_len() {
        let mut grid: Grid<char> = grid![];
        grid.push_col(vec![]);
    }

    #[test]
    fn push_col_2x3_column_major() {
        let internal = vec![0, 10, 1, 11, 2, 12];
        let mut grid: Grid<u8> = Grid::from_vec_with_order(internal, 3, Order::ColumnMajor);
        grid.push_col(vec![3, 13]);
        let expected = [0, 10, 1, 11, 2, 12, 3, 13];
        test_grid(&grid, 2, 4, Order::ColumnMajor, &expected);
    }

    #[test]
    fn push_col_3x4_column_major() {
        let internal = vec!['a', 'a', 'a', 'b', 'b', 'b', 'c', 'c', 'c', 'd', 'd', 'd'];
        let mut grid: Grid<char> = Grid::from_vec_with_order(internal, 4, Order::ColumnMajor);
        grid.push_col(vec!['x', 'y', 'z']);
        let expected = [
            'a', 'a', 'a', 'b', 'b', 'b', 'c', 'c', 'c', 'd', 'd', 'd', 'x', 'y', 'z',
        ];
        test_grid(&grid, 3, 5, Order::ColumnMajor, &expected);
    }

    #[test]
    fn push_col_1x3_column_major() {
        let mut grid: Grid<char> =
            Grid::from_vec_with_order(vec!['a', 'b', 'c'], 3, Order::ColumnMajor);
        grid.push_col(vec!['d']);
        test_grid(&grid, 1, 4, Order::ColumnMajor, &['a', 'b', 'c', 'd']);
    }

    #[test]
    fn push_col_empty_column_major() {
        let mut grid: Grid<char> = Grid::new_with_order(0, 0, Order::ColumnMajor);
        grid.push_col(vec!['b', 'b', 'b', 'b']);
        test_grid(&grid, 4, 1, Order::ColumnMajor, &['b', 'b', 'b', 'b']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_col_wrong_size_column_major() {
        let mut grid: Grid<char> = Grid::init_with_order(2, 3, Order::ColumnMajor, 'a');
        grid.push_col(vec!['b']);
        grid.push_col(vec!['b', 'b']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_col_zero_len_column_major() {
        let mut grid: Grid<char> = Grid::new_with_order(0, 0, Order::ColumnMajor);
        grid.push_col(vec![]);
    }

    #[test]
    fn push_row() {
        let mut grid: Grid<u8> = grid![[1, 2][3, 4]];
        grid.push_row(vec![5, 6]);
        test_grid(&grid, 3, 2, Order::RowMajor, &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn push_row_empty() {
        let mut grid: Grid<char> = grid![];
        grid.push_row(vec!['b', 'b', 'b', 'b']);
        test_grid(&grid, 1, 4, Order::RowMajor, &['b', 'b', 'b', 'b']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_empty_row() {
        let mut grid = Grid::init(0, 1, 0);
        grid.push_row(vec![]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_row_wrong_size() {
        let mut grid: Grid<char> = grid![['a','a','a']['a','a','a']];
        grid.push_row(vec!['b']);
        grid.push_row(vec!['b', 'b', 'b', 'b']);
    }

    #[test]
    fn push_row_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.push_row(vec![5, 6]);
        test_grid(&grid, 3, 2, Order::ColumnMajor, &[1, 3, 5, 2, 4, 6]);
    }

    #[test]
    fn push_row_empty_column_major() {
        let mut grid: Grid<char> = Grid::new_with_order(0, 0, Order::ColumnMajor);
        grid.push_row(vec!['b', 'b', 'b', 'b']);
        test_grid(&grid, 1, 4, Order::ColumnMajor, &['b', 'b', 'b', 'b']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_empty_row_column_major() {
        let mut grid = Grid::init_with_order(0, 1, Order::ColumnMajor, 0);
        grid.push_row(vec![]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn push_row_wrong_size_column_major() {
        let mut grid: Grid<char> =
            Grid::from_vec_with_order(vec!['a', 'a', 'a', 'a', 'a', 'a'], 3, Order::ColumnMajor);
        grid.push_row(vec!['b']);
        grid.push_row(vec!['b', 'b', 'b', 'b']);
    }

    #[test]
    fn expand_rows() {
        let mut grid = Grid::from_vec_with_order(vec![1, 1, 1, 2, 2, 2], 3, Order::RowMajor);
        grid.expand_rows(2);

        assert_eq!(grid.dimensions(), (4, 3));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 4);
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.into_inner(), vec![1, 1, 1, 2, 2, 2, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn expand_rows_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 1, 2, 1, 2], 3, Order::ColumnMajor);
        grid.expand_rows(2);

        assert_eq!(grid.dimensions(), (4, 3));
        assert_eq!(grid.order, Order::ColumnMajor);
        assert_eq!(grid.rows(), 4);
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.into_inner(), vec![1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0, 0]);
    }

    #[test]
    fn expand_rows_zero() {
        let mut grid = Grid::from_vec_with_order(vec![1, 1, 1], 3, Order::RowMajor);
        grid.expand_rows(0);

        assert_eq!(grid.dimensions(), (1, 3));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 1);
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.into_inner(), vec![1, 1, 1]);
    }

    #[test]
    fn expand_rows_empty_grid() {
        let mut grid: Grid<u8> = Grid::init(0, 0, 0);
        grid.expand_rows(2);

        assert_eq!(grid.dimensions(), (0, 0));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 0);
        assert_eq!(grid.cols(), 0);
        assert_eq!(grid.into_inner(), Vec::<u8>::new());
    }

    #[test]
    fn expand_cols() {
        let mut grid = Grid::from_vec_with_order(vec![1, 1, 1, 2, 2, 2], 3, Order::RowMajor);
        grid.expand_cols(2);

        assert_eq!(grid.dimensions(), (2, 5));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 5);
        assert_eq!(grid.into_inner(), vec![1, 1, 1, 0, 0, 2, 2, 2, 0, 0]);
    }

    #[test]
    fn expand_cols_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 1, 2, 1, 2], 3, Order::ColumnMajor);
        grid.expand_cols(2);

        assert_eq!(grid.dimensions(), (2, 5));
        assert_eq!(grid.order, Order::ColumnMajor);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 5);
        assert_eq!(grid.into_inner(), vec![1, 2, 1, 2, 1, 2, 0, 0, 0, 0]);
    }

    #[test]
    fn expand_cols_zero() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 1, 2], 2, Order::RowMajor);
        grid.expand_cols(0);

        assert_eq!(grid.dimensions(), (2, 2));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 2);
        assert_eq!(grid.into_inner(), vec![1, 2, 1, 2]);
    }

    #[test]
    fn expand_cols_empty_grid() {
        let mut grid: Grid<u8> = Grid::init(0, 0, 0);
        grid.expand_cols(2);

        assert_eq!(grid.dimensions(), (0, 0));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 0);
        assert_eq!(grid.cols(), 0);
        assert_eq!(grid.into_inner(), Vec::<u8>::new());
    }

    #[test]
    fn prepend_rows() {
        let mut grid = Grid::from_vec_with_order(vec![1, 1, 1, 2, 2, 2], 3, Order::RowMajor);
        grid.prepend_rows(2);

        assert_eq!(grid.dimensions(), (4, 3));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 4);
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.into_inner(), vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2]);
    }

    #[test]
    fn prepend_rows_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 1, 2, 1, 2], 3, Order::ColumnMajor);
        grid.prepend_rows(2);

        assert_eq!(grid.dimensions(), (4, 3));
        assert_eq!(grid.order, Order::ColumnMajor);
        assert_eq!(grid.rows(), 4);
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.into_inner(), vec![0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2]);
    }

    #[test]
    fn prepend_rows_zero() {
        let mut grid = Grid::from_vec_with_order(vec![1, 1, 1], 3, Order::RowMajor);
        grid.prepend_rows(0);

        assert_eq!(grid.dimensions(), (1, 3));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 1);
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.into_inner(), vec![1, 1, 1]);
    }

    #[test]
    fn prepend_rows_empty_grid() {
        let mut grid: Grid<u8> = Grid::init(0, 0, 0);
        grid.prepend_rows(2);

        assert_eq!(grid.dimensions(), (0, 0));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 0);
        assert_eq!(grid.cols(), 0);
        assert_eq!(grid.into_inner(), Vec::<u8>::new());
    }

    #[test]
    fn prepend_cols() {
        let mut grid = Grid::from_vec_with_order(vec![1, 1, 1, 2, 2, 2], 3, Order::RowMajor);
        grid.prepend_cols(2);

        assert_eq!(grid.dimensions(), (2, 5));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 5);
        assert_eq!(grid.into_inner(), vec![0, 0, 1, 1, 1, 0, 0, 2, 2, 2]);
    }

    #[test]
    fn prepend_cols_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 1, 2, 1, 2], 3, Order::ColumnMajor);
        grid.prepend_cols(2);

        assert_eq!(grid.dimensions(), (2, 5));
        assert_eq!(grid.order, Order::ColumnMajor);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 5);
        assert_eq!(grid.into_inner(), vec![0, 0, 0, 0, 1, 2, 1, 2, 1, 2]);
    }

    #[test]
    fn prepend_cols_zero() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 1, 2], 2, Order::RowMajor);
        grid.prepend_cols(0);

        assert_eq!(grid.dimensions(), (2, 2));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.cols(), 2);
        assert_eq!(grid.into_inner(), vec![1, 2, 1, 2]);
    }

    #[test]
    fn prepend_cols_empty_grid() {
        let mut grid: Grid<u8> = Grid::init(0, 0, 0);
        grid.prepend_cols(2);

        assert_eq!(grid.dimensions(), (0, 0));
        assert_eq!(grid.order, Order::RowMajor);
        assert_eq!(grid.rows(), 0);
        assert_eq!(grid.cols(), 0);
        assert_eq!(grid.into_inner(), Vec::<u8>::new());
    }

    #[test]
    fn iter_row() {
        let grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let row: Vec<_> = grid.iter_row(1).collect();
        assert_eq!(row, [&4, &5, &6]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_out_of_bound() {
        let grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let _ = grid.iter_row(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_zero() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        let _ = grid.iter_row(0);
    }

    #[test]
    fn iter_row_rowumn_major() {
        let grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let row: Vec<_> = grid.iter_row(1).collect();
        assert_eq!(row, [&4, &5, &6]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_rowumn_major_out_of_bound() {
        let grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let _ = grid.iter_row(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_rowumn_major_zero() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        let _ = grid.iter_row(0);
    }

    #[test]
    fn iter_row_mut() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let row: Vec<_> = grid.iter_row_mut(1).collect();
        assert_eq!(row, [&mut 4, &mut 5, &mut 6]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_mut_out_of_bound() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let _ = grid.iter_row_mut(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_mut_zero() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        let _ = grid.iter_row_mut(0);
    }

    #[test]
    fn iter_row_mut_rowumn_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let row: Vec<_> = grid.iter_row_mut(1).collect();
        assert_eq!(row, [&mut 4, &mut 5, &mut 6]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_mut_rowumn_major_out_of_bound() {
        let mut grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let _ = grid.iter_row_mut(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_row_mut_rowumn_major_zero() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        let _ = grid.iter_row_mut(0);
    }

    #[test]
    fn iter_col() {
        let grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let col: Vec<_> = grid.iter_col(1).collect();
        assert_eq!(col, [&2, &5]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_out_of_bound() {
        let grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let _ = grid.iter_col(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_zero() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        let _ = grid.iter_col(0);
    }

    #[test]
    fn iter_col_column_major() {
        let grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let col: Vec<_> = grid.iter_col(1).collect();
        assert_eq!(col, [&2, &5]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_column_major_out_of_bound() {
        let grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let _ = grid.iter_col(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_column_major_zero() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        let _ = grid.iter_col(0);
    }

    #[test]
    fn iter_col_mut() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let col: Vec<_> = grid.iter_col_mut(1).collect();
        assert_eq!(col, [&mut 2, &mut 5]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_mut_out_of_bound() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        let _ = grid.iter_col_mut(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_mut_zero() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::RowMajor);
        let _ = grid.iter_col_mut(0);
    }

    #[test]
    fn iter_col_mut_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let col: Vec<_> = grid.iter_col_mut(1).collect();
        assert_eq!(col, [&mut 2, &mut 5]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_mut_column_major_out_of_bound() {
        let mut grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let _ = grid.iter_col_mut(3);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn iter_col_mut_column_major_zero() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![], 0, Order::ColumnMajor);
        let _ = grid.iter_col_mut(0);
    }

    #[test]
    fn iter() {
        let grid: Grid<u8> = grid![[1,2][3,4]];
        let mut iter = grid.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter() {
        let grid: Grid<u8> = grid![[1,1][1,1]];
        for val in grid {
            assert_eq!(val, 1);
        }
    }

    #[test]
    fn into_iter_ref() {
        let grid: Grid<u8> = grid![[1,1][1,1]];
        for val in &grid {
            assert_eq!(val, &1);
        }
    }

    #[test]
    fn into_iter_mut() {
        let mut grid: Grid<u8> = grid![[1,1][1,1]];
        grid.fill(2);
        assert_eq!(grid, grid![[2, 2][2, 2]]);
    }

    #[test]
    fn indexed_iter() {
        let grid: Grid<u8> = grid![[1,2][3,4]];
        let mut iter = grid.indexed_iter();
        assert_eq!(iter.next(), Some(((0, 0), &1)));
        assert_eq!(iter.next(), Some(((0, 1), &2)));
        assert_eq!(iter.next(), Some(((1, 0), &3)));
        assert_eq!(iter.next(), Some(((1, 1), &4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_empty() {
        let grid: Grid<u8> = Grid::new(0, 0);
        let mut iter = grid.indexed_iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_into_iter() {
        let grid: Grid<u8> = grid![[1,2][3,4]];
        let mut iter = grid.indexed_into_iter();
        assert_eq!(iter.next(), Some(((0, 0), 1)));
        assert_eq!(iter.next(), Some(((0, 1), 2)));
        assert_eq!(iter.next(), Some(((1, 0), 3)));
        assert_eq!(iter.next(), Some(((1, 1), 4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_column_major() {
        let grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        let mut iter = grid.indexed_iter();
        assert_eq!(iter.next(), Some(((0, 0), &1)));
        assert_eq!(iter.next(), Some(((1, 0), &3)));
        assert_eq!(iter.next(), Some(((0, 1), &2)));
        assert_eq!(iter.next(), Some(((1, 1), &4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_empty_column_major() {
        let grid: Grid<u8> = Grid::new_with_order(0, 0, Order::ColumnMajor);
        let mut iter = grid.indexed_iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_mut() {
        let mut grid: Grid<u8> = grid![[1,2][3,4]];
        let mut iter = grid.indexed_iter_mut();
        assert_eq!(iter.next(), Some(((0, 0), &mut 1)));
        assert_eq!(iter.next(), Some(((0, 1), &mut 2)));
        assert_eq!(iter.next(), Some(((1, 0), &mut 3)));
        assert_eq!(iter.next(), Some(((1, 1), &mut 4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_mut_empty() {
        let mut grid: Grid<u8> = Grid::new(0, 0);
        let mut iter = grid.indexed_iter_mut();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_mut_column_major() {
        let mut grid: Grid<u8> = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        let mut iter = grid.indexed_iter_mut();
        assert_eq!(iter.next(), Some(((0, 0), &mut 1)));
        assert_eq!(iter.next(), Some(((1, 0), &mut 3)));
        assert_eq!(iter.next(), Some(((0, 1), &mut 2)));
        assert_eq!(iter.next(), Some(((1, 1), &mut 4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn indexed_iter_mut_empty_column_major() {
        let mut grid: Grid<u8> = Grid::new_with_order(0, 0, Order::ColumnMajor);
        let mut iter = grid.indexed_iter_mut();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn clear() {
        let mut grid: Grid<u8> = grid![[1, 2, 3]];
        assert!(!grid.is_empty());
        grid.clear();
        assert!(grid.is_empty());
    }

    #[test]
    fn is_empty_false() {
        let grid: Grid<u8> = grid![[1, 2, 3]];
        assert!(!grid.is_empty());
    }

    #[test]
    fn is_empty() {
        let mut g: Grid<u8> = grid![[]];
        assert!(g.is_empty());
        g = grid![];
        assert!(g.is_empty());
        g = Grid::from_vec(vec![], 0);
        assert!(g.is_empty());
        g = Grid::new(0, 0);
        assert!(g.is_empty());
        g = Grid::new(0, 1);
        assert!(g.is_empty());
        g = Grid::new(1, 0);
        assert!(g.is_empty());
        g = Grid::init(0, 0, 10);
        assert!(g.is_empty());
    }

    #[test]
    fn fmt_empty() {
        let grid: Grid<u8> = grid![];
        assert_eq!(format!("{grid:?}"), "[]");
    }

    #[test]
    fn fmt_row() {
        let grid: Grid<u8> = grid![[1, 2, 3]];
        assert_eq!(format!("{grid:?}"), "[[1, 2, 3]]");
    }

    #[test]
    fn fmt_grid() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6][7,8,9]];
        assert_eq!(format!("{grid:?}"), "[[1, 2, 3][4, 5, 6][7, 8, 9]]");
    }

    #[test]
    fn fmt_column_major() {
        let grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        assert_eq!(format!("{grid:?}"), "[[1, 2, 3][4, 5, 6]]");
    }

    #[test]
    fn fmt_pretty_empty() {
        let grid: Grid<f32> = grid![];
        assert_eq!(format!("{grid:#?}"), "[]");
    }

    #[test]
    fn fmt_pretty_int() {
        let grid: Grid<u8> = grid![
            [1,2,3]
            [4,5,6]
            [7,8,95]
        ];

        let expected_output = r"[
    [  1,  2,  3]
    [  4,  5,  6]
    [  7,  8, 95]
]";

        assert_eq!(format!("{grid:#?}"), expected_output);

        let expected_output = r"[
    [   1,   2,   3]
    [   4,   5,   6]
    [   7,   8,  95]
]";

        assert_eq!(format!("{grid:#3?}"), expected_output);
    }

    #[test]
    fn fmt_pretty_float() {
        let grid: Grid<f32> = grid![
            [1.5,2.6,3.44]
            [4.775,5.,6.]
            [7.1,8.23444,95.55]
        ];

        let expected_output = r"[
    [   1.5,   2.6,   3.4]
    [   4.8,   5.0,   6.0]
    [   7.1,   8.2,  95.6]
]";

        assert_eq!(format!("{grid:#5.1?}"), expected_output);

        let expected_output = r"[
    [  1.50000,  2.60000,  3.44000]
    [  4.77500,  5.00000,  6.00000]
    [  7.10000,  8.23444, 95.55000]
]";

        assert_eq!(format!("{grid:#8.5?}"), expected_output);
    }

    #[test]
    fn fmt_pretty_tuple() {
        let grid: Grid<(i32, i32)> = grid![
            [(5,66), (432, 55)]
            [(80, 90), (5, 6)]
        ];

        let expected_output = r"[
    [ (        5,        66), (      432,        55)]
    [ (       80,        90), (        5,         6)]
]";

        assert_eq!(format!("{grid:#?}"), expected_output);

        let expected_output = r"[
    [ (  5,  66), (432,  55)]
    [ ( 80,  90), (  5,   6)]
]";

        assert_eq!(format!("{grid:#3?}"), expected_output);
    }

    #[test]
    fn fmt_pretty_struct_derived() {
        #[derive(Debug)]
        struct Person {
            _name: String,
            _precise_age: f32,
        }

        impl Person {
            fn new(name: &str, precise_age: f32) -> Self {
                Self {
                    _name: name.into(),
                    _precise_age: precise_age,
                }
            }
        }

        let grid: Grid<Person> = grid![
            [Person::new("Vic", 24.5), Person::new("Mr. Very Long Name", 1955.)]
            [Person::new("Sam", 8.9995), Person::new("John Doe", 40.14)]
        ];

        let expected_output = r#"[
    [ Person { _name: "Vic", _precise_age: 24.50000 }, Person { _name: "Mr. Very Long Name", _precise_age: 1955.00000 }]
    [ Person { _name: "Sam", _precise_age: 8.99950 }, Person { _name: "John Doe", _precise_age: 40.14000 }]
]"#;

        assert_eq!(format!("{grid:#5.5?}"), expected_output);
    }

    #[test]
    fn fmt_pretty_column_major() {
        let grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        let expected_output = r"[
    [ 1, 2, 3]
    [ 4, 5, 6]
]";
        assert_eq!(format!("{grid:#?}"), expected_output);
    }

    #[test]
    fn clone() {
        let grid = grid![[1, 2, 3][4, 5, 6]];
        let mut clone = grid.clone();
        clone[(0, 2)] = 10;
        test_grid(&grid, 2, 3, Order::RowMajor, &[1, 2, 3, 4, 5, 6]);
        test_grid(&clone, 2, 3, Order::RowMajor, &[1, 2, 10, 4, 5, 6]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_std() {
        let mut set = std::collections::HashSet::new();
        set.insert(grid![[1,2,3][4,5,6]]);
        set.insert(grid![[1,3,3][4,5,6]]);
        set.insert(grid![[1,2,3][4,5,6]]);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn macro_init() {
        let grid = grid![[1, 2, 3][4, 5, 6]];
        test_grid(&grid, 2, 3, Order::RowMajor, &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn macro_init_2() {
        let grid = grid![[1, 2, 3]
                         [4, 5, 6]
                         [7, 8, 9]];
        test_grid(&grid, 3, 3, Order::RowMajor, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn macro_init_char() {
        let grid = grid![['a', 'b', 'c']
                         ['a', 'b', 'c']
                         ['a', 'b', 'c']];
        let expected = ['a', 'b', 'c', 'a', 'b', 'c', 'a', 'b', 'c'];
        test_grid(&grid, 3, 3, Order::RowMajor, &expected);
    }

    #[test]
    fn macro_one_row() {
        let grid: Grid<usize> = grid![[1, 2, 3, 4]];
        test_grid(&grid, 1, 4, Order::RowMajor, &[1, 2, 3, 4]);
    }

    #[test]
    fn macro2_empty() {
        let grid: Grid<u8> = grid_cm![];
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn macro2_init() {
        let grid = grid_cm![[1, 2, 3]
                          [4, 5, 6]
                          [7, 8, 9]];
        let expected = [1, 4, 7, 2, 5, 8, 3, 6, 9];
        test_grid(&grid, 3, 3, Order::ColumnMajor, &expected);
    }

    #[test]
    fn macro2_init_char() {
        let grid = grid_cm![['a', 'b']['c', 'd']];
        test_grid(&grid, 2, 2, Order::ColumnMajor, &['a', 'c', 'b', 'd']);
    }

    #[test]
    fn macro2_one_row() {
        let grid = grid_cm![[1, 2, 3, 4]];
        test_grid(&grid, 1, 4, Order::ColumnMajor, &[1, 2, 3, 4]);
    }

    #[test]
    fn init() {
        let grid = Grid::init(1, 2, 3);
        test_grid(&grid, 1, 2, Order::RowMajor, &[3, 3]);

        let grid = Grid::init(1, 2, 1.2);
        test_grid(&grid, 1, 2, Order::RowMajor, &[1.2, 1.2]);

        let grid = Grid::init(1, 2, 'a');
        test_grid(&grid, 1, 2, Order::RowMajor, &['a', 'a']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn init_panics() {
        Grid::init(usize::MAX, 2, 3);
    }

    #[test]
    fn init_empty() {
        let grid = Grid::init(0, 1, 0);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);

        let grid = Grid::init(1, 0, -1);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn init_with_order() {
        let grid = Grid::init_with_order(1, 2, Order::RowMajor, 3);
        test_grid(&grid, 1, 2, Order::RowMajor, &[3, 3]);

        let grid = Grid::init_with_order(1, 2, Order::ColumnMajor, 1.2);
        test_grid(&grid, 1, 2, Order::ColumnMajor, &[1.2, 1.2]);

        let grid = Grid::init_with_order(1, 2, Order::ColumnMajor, 'a');
        test_grid(&grid, 1, 2, Order::ColumnMajor, &['a', 'a']);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn init_with_order_panics() {
        Grid::init_with_order(usize::MAX, 2, Order::ColumnMajor, 3);
    }

    #[test]
    fn init_with_order_empty() {
        let grid = Grid::init_with_order(0, 1, Order::ColumnMajor, 0);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);

        let grid = Grid::init_with_order(1, 0, Order::RowMajor, -1);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn new() {
        let grid: Grid<u8> = Grid::new(1, 2);
        test_grid(&grid, 1, 2, Order::RowMajor, &[0, 0]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn new_panics() {
        let _: Grid<u8> = Grid::new(usize::MAX, 2);
    }

    #[test]
    fn new_empty() {
        let grid: Grid<u8> = Grid::new(0, 1);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);

        let grid: Grid<u8> = Grid::new(1, 0);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn new_with_order() {
        let grid: Grid<u8> = Grid::new_with_order(2, 2, Order::ColumnMajor);
        test_grid(&grid, 2, 2, Order::ColumnMajor, &[0, 0, 0, 0]);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn new_with_order_panics() {
        let _: Grid<u8> = Grid::new_with_order(usize::MAX, 2, Order::ColumnMajor);
    }

    #[test]
    fn new_with_order_empty() {
        let grid: Grid<u8> = Grid::new_with_order(0, 3, Order::RowMajor);
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);

        let grid: Grid<u8> = Grid::new_with_order(3, 0, Order::ColumnMajor);
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn with_capacity() {
        // doesn't impl Default
        struct Foo();

        let grid: Grid<Foo> = Grid::with_capacity(20, 20);
        assert!(grid.is_empty());
        assert_eq!(grid.order(), Order::default());
    }

    #[test]
    fn with_capacity_and_order() {
        // doesn't impl Default
        struct Foo();

        let grid: Grid<Foo> = Grid::with_capacity_and_order(20, 20, Order::ColumnMajor);
        assert!(grid.is_empty());
        assert_eq!(grid.order(), Order::ColumnMajor);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn with_capacity_panics_internal() {
        // doesn't impl Default
        struct Foo();

        let _grid: Grid<Foo> = Grid::with_capacity_and_order(usize::MAX, 2, Order::RowMajor);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn with_capacity_panics_vec() {
        let rows: usize = isize::MAX.try_into().expect("isize::MAX is positive");
        assert!(
            core::mem::size_of::<u8>() * rows < usize::MAX,
            "shows that panic is from Vec::with_capacity, not internal check"
        );

        let _grid: Grid<u8> = Grid::with_capacity_and_order(rows, 2, Order::RowMajor);
    }

    #[test]
    fn get() {
        let grid = Grid::from_vec_with_order(vec![1, 2], 2, Order::RowMajor);
        assert_eq!(grid.get(0_i64, 1_i32), Some(&2));
    }

    #[test]
    fn get_column_major() {
        let grid = Grid::from_vec_with_order(vec![1, 2], 1, Order::ColumnMajor);
        assert_eq!(grid.get(1, 0), Some(&2));
    }

    #[test]
    fn get_none() {
        let grid = Grid::from_vec_with_order(vec![1, 2], 2, Order::RowMajor);
        assert_eq!(grid.get(1, 0), None);
    }

    #[test]
    fn get_none_column_major() {
        let grid = Grid::from_vec_with_order(vec![1, 2], 1, Order::ColumnMajor);
        assert_eq!(grid.get(0, 1), None);
    }

    #[test]
    fn get_mut() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2], 2, Order::RowMajor);
        assert_eq!(grid.get_mut(0_i64, 1_i32), Some(&mut 2));
    }

    #[test]
    fn get_mut_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2], 1, Order::ColumnMajor);
        assert_eq!(grid.get_mut(1, 0), Some(&mut 2));
    }

    #[test]
    fn get_mut_none() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2], 2, Order::RowMajor);
        assert_eq!(grid.get_mut(1, 0), None);
    }

    #[test]
    fn get_mut_none_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2], 1, Order::ColumnMajor);
        assert_eq!(grid.get_mut(0, 1), None);
    }

    #[test]
    fn idx_tup() {
        let grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        assert_eq!(grid[(0, 0)], 1);
        assert_eq!(grid[(0, 1)], 2);
        assert_eq!(grid[(1, 0)], 3);
        assert_eq!(grid[(1, 1)], 4);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn idx_tup_panic_1() {
        let grid = Grid::init(1, 2, 3);
        let _ = grid[(20, 0)];
    }

    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn idx_tup_panic_2() {
        let grid = Grid::init(1, 2, 3);
        let _ = grid[(0, 20)];
    }

    #[test]
    fn idx_tup_set() {
        let mut grid = Grid::init(1, 2, 3);
        grid[(0, 0)] = 4;
        assert_eq!(grid[(0, 0)], 4);
    }

    #[test]
    fn size() {
        let grid = Grid::init(1, 2, 3);
        assert_eq!(grid.dimensions(), (1, 2));
    }

    #[test]
    fn transpose() {
        let mut grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        grid.transpose();
        assert_eq!(grid, grid![[1,4][2,5][3,6]]);
    }

    #[test]
    fn fill() {
        let mut grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        grid.fill(7);
        test_grid(&grid, 2, 3, Order::RowMajor, &[7, 7, 7, 7, 7, 7]);
    }

    #[test]
    fn fill_with() {
        let mut grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        grid.fill_with(Default::default);
        test_grid(&grid, 2, 3, Order::RowMajor, &[0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn map() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let mapped = grid.map(|x| x * 2);
        test_grid(&mapped, 2, 3, Order::RowMajor, &[2, 4, 6, 8, 10, 12]);
    }

    #[test]
    fn map_ref() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let mapped = grid.map_ref(|x| *x * 2);
        test_grid(&mapped, 2, 3, Order::RowMajor, &[2, 4, 6, 8, 10, 12]);
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn iter_rows() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let max_by_row: Vec<u8> = grid
            .iter_rows()
            .map(|row| row.max().unwrap())
            .copied()
            .collect();
        assert_eq!(max_by_row, vec![3, 6]);

        let sum_by_row: Vec<u8> = grid.iter_rows().map(|row| row.sum()).collect();
        assert_eq!(sum_by_row, vec![1 + 2 + 3, 4 + 5 + 6]);
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn iter_rows_rev() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let max_by_row: Vec<u8> = grid
            .iter_rows()
            .rev()
            .map(|row| row.max().unwrap())
            .copied()
            .collect();
        assert_eq!(max_by_row, vec![6, 3]);

        let sum_by_row: Vec<u8> = grid.iter_rows().rev().map(|row| row.sum()).collect();
        assert_eq!(sum_by_row, vec![4 + 5 + 6, 1 + 2 + 3]);
    }

    #[test]
    fn iter_rows_exact_size() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let mut row_iter = grid.iter_rows();
        assert_eq!(row_iter.len(), 2);
        assert!(row_iter.next().is_some());
        assert_eq!(row_iter.len(), 1);
        assert!(row_iter.next().is_some());
        assert_eq!(row_iter.len(), 0);
        assert!(row_iter.next().is_none());
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn iter_cols() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let max_by_col: Vec<u8> = grid
            .iter_cols()
            .map(|col| col.max().unwrap())
            .copied()
            .collect();

        assert_eq!(max_by_col, vec![4, 5, 6]);

        let sum_by_col: Vec<u8> = grid.iter_cols().map(|row| row.sum()).collect();
        assert_eq!(sum_by_col, vec![1 + 4, 2 + 5, 3 + 6]);
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn iter_cols_rev() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let max_by_col: Vec<u8> = grid
            .iter_cols()
            .rev()
            .map(|col| col.max().unwrap())
            .copied()
            .collect();

        assert_eq!(max_by_col, vec![6, 5, 4]);

        let sum_by_col: Vec<u8> = grid.iter_cols().rev().map(|row| row.sum()).collect();
        assert_eq!(sum_by_col, vec![3 + 6, 2 + 5, 1 + 4]);
    }

    #[test]
    fn iter_cols_exact_size() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6]];
        let mut col_iter = grid.iter_cols();
        assert_eq!(col_iter.len(), 3);
        assert!(col_iter.next().is_some());
        assert_eq!(col_iter.len(), 2);
        assert!(col_iter.next().is_some());
        assert_eq!(col_iter.len(), 1);
        assert!(col_iter.next().is_some());
        assert_eq!(col_iter.len(), 0);
        assert!(col_iter.next().is_none());
    }

    #[test]
    fn remove_row() {
        let mut grid = grid![[1,2][3,4][5,6]];
        assert_eq![grid.remove_row(1), Some(vec![3, 4])];
        test_grid(&grid, 2, 2, Order::RowMajor, &[1, 2, 5, 6]);
        assert_eq![grid.remove_row(0), Some(vec![1, 2])];
        test_grid(&grid, 1, 2, Order::RowMajor, &[5, 6]);
        assert_eq![grid.remove_row(0), Some(vec![5, 6])];
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq![grid.remove_row(0), None];
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn remove_row_out_of_bound() {
        let mut grid = grid![[1, 2][3, 4]];
        assert_eq![grid.remove_row(5), None];
        test_grid(&grid, 2, 2, Order::RowMajor, &[1, 2, 3, 4]);
        assert_eq![grid.remove_row(1), Some(vec![3, 4])];
        test_grid(&grid, 1, 2, Order::RowMajor, &[1, 2]);
    }

    #[test]
    fn remove_row_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 3, 5, 2, 4, 6], 2, Order::ColumnMajor);
        assert_eq![grid.remove_row(1), Some(vec![3, 4])];
        test_grid(&grid, 2, 2, Order::ColumnMajor, &[1, 5, 2, 6]);
        assert_eq![grid.remove_row(0), Some(vec![1, 2])];
        test_grid(&grid, 1, 2, Order::ColumnMajor, &[5, 6]);
        assert_eq![grid.remove_row(0), Some(vec![5, 6])];
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq![grid.remove_row(0), None];
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn remove_row_out_of_bound_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        assert_eq![grid.remove_row(5), None];
        test_grid(&grid, 2, 2, Order::ColumnMajor, &[1, 3, 2, 4]);
        assert_eq![grid.remove_row(1), Some(vec![3, 4])];
        test_grid(&grid, 1, 2, Order::ColumnMajor, &[1, 2]);
    }

    #[test]
    fn remove_col() {
        let mut grid = grid![[1,2,3,4][5,6,7,8][9,10,11,12][13,14,15,16]];
        assert_eq![grid.remove_col(3), Some(vec![4, 8, 12, 16])];
        let expected = [1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15];
        test_grid(&grid, 4, 3, Order::RowMajor, &expected);
        assert_eq![grid.remove_col(0), Some(vec![1, 5, 9, 13])];
        test_grid(&grid, 4, 2, Order::RowMajor, &[2, 3, 6, 7, 10, 11, 14, 15]);
        assert_eq![grid.remove_col(1), Some(vec![3, 7, 11, 15])];
        test_grid(&grid, 4, 1, Order::RowMajor, &[2, 6, 10, 14]);
        assert_eq![grid.remove_col(0), Some(vec![2, 6, 10, 14])];
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
        assert_eq![grid.remove_col(0), None];
        test_grid(&grid, 0, 0, Order::RowMajor, &[]);
    }

    #[test]
    fn remove_col_out_of_bound() {
        let mut grid = grid![[1, 2][3, 4]];
        assert_eq!(grid.remove_col(5), None);
        test_grid(&grid, 2, 2, Order::RowMajor, &[1, 2, 3, 4]);
        assert_eq!(grid.remove_col(1), Some(vec![2, 4]));
        test_grid(&grid, 2, 1, Order::RowMajor, &[1, 3]);
    }

    #[test]
    fn remove_col_column_major() {
        let internal = vec![1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15, 4, 8, 12, 16];
        let mut grid = Grid::from_vec_with_order(internal, 4, Order::ColumnMajor);
        assert_eq![grid.remove_col(3), Some(vec![4, 8, 12, 16])];
        let expected = [1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15];
        test_grid(&grid, 4, 3, Order::ColumnMajor, &expected);
        assert_eq![grid.remove_col(0), Some(vec![1, 5, 9, 13])];
        let expected = [2, 6, 10, 14, 3, 7, 11, 15];
        test_grid(&grid, 4, 2, Order::ColumnMajor, &expected);
        assert_eq![grid.remove_col(1), Some(vec![3, 7, 11, 15])];
        test_grid(&grid, 4, 1, Order::ColumnMajor, &[2, 6, 10, 14]);
        assert_eq![grid.remove_col(0), Some(vec![2, 6, 10, 14])];
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
        assert_eq![grid.remove_col(0), None];
        test_grid(&grid, 0, 0, Order::ColumnMajor, &[]);
    }

    #[test]
    fn remove_col_out_of_bound_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        assert_eq!(grid.remove_col(5), None);
        test_grid(&grid, 2, 2, Order::ColumnMajor, &[1, 3, 2, 4]);
        assert_eq!(grid.remove_col(1), Some(vec![2, 4]));
        test_grid(&grid, 2, 1, Order::ColumnMajor, &[1, 3]);
    }

    #[test]
    fn flip_cols() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.flip_cols();
        test_grid(&grid, 2, 2, Order::RowMajor, &[2, 1, 4, 3]);
    }

    #[test]
    fn flip_cols_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.flip_cols();
        test_grid(&grid, 2, 2, Order::ColumnMajor, &[2, 4, 1, 3]);
    }

    #[test]
    fn flip_rows() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4], 2, Order::RowMajor);
        grid.flip_rows();
        test_grid(&grid, 2, 2, Order::RowMajor, &[3, 4, 1, 2]);
    }

    #[test]
    fn flip_rows_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 3, 2, 4], 2, Order::ColumnMajor);
        grid.flip_rows();
        test_grid(&grid, 2, 2, Order::ColumnMajor, &[3, 1, 4, 2]);
    }

    #[test]
    fn rotate_left() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        grid.rotate_left();
        test_grid(&grid, 3, 2, Order::ColumnMajor, &[3, 2, 1, 6, 5, 4]);
        assert_eq!(grid, grid![[3,6][2,5][1,4]]);
    }

    #[test]
    fn rotate_left_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        grid.rotate_left();
        test_grid(&grid, 3, 2, Order::RowMajor, &[3, 6, 2, 5, 1, 4]);
        assert_eq!(grid, grid![[3,6][2,5][1,4]]);
    }

    #[test]
    fn rotate_right() {
        let mut grid = Grid::from_vec_with_order(vec![1, 2, 3, 4, 5, 6], 3, Order::RowMajor);
        grid.rotate_right();
        test_grid(&grid, 3, 2, Order::ColumnMajor, &[4, 5, 6, 1, 2, 3]);
        assert_eq!(grid, grid![[4,1][5,2][6,3]]);
    }

    #[test]
    fn rotate_right_column_major() {
        let mut grid = Grid::from_vec_with_order(vec![1, 4, 2, 5, 3, 6], 3, Order::ColumnMajor);
        grid.rotate_right();
        test_grid(&grid, 3, 2, Order::RowMajor, &[4, 1, 5, 2, 6, 3]);
        assert_eq!(grid, grid![[4,1][5,2][6,3]]);
    }

    #[test]
    fn iter_cols_clone() {
        let grid = grid![[1,2,3][4,5,6]];
        let mut cols = grid.iter_cols().skip(1);
        let c3: u8 = cols.clone().nth(1).unwrap().sum();
        let c2: u8 = cols.next().unwrap().sum();
        assert_eq!(c2, 2 + 5);
        assert_eq!(c3, 3 + 6);
    }

    #[test]
    fn iter_rows_clone() {
        let grid = grid![[1,2,3][4,5,6][7,8,9]];
        let mut rows = grid.iter_rows().skip(1);
        let r3: u8 = rows.clone().nth(1).unwrap().sum();
        let r2: u8 = rows.next().unwrap().sum();
        assert_eq!(r2, 4 + 5 + 6);
        assert_eq!(r3, 7 + 8 + 9);
    }

    #[test]
    fn swap() {
        let mut grid = grid![[1,2][4,5]];
        grid.swap((0, 0), (1, 0));
        let end_grid = grid![[4,2][1,5]];
        assert_eq!(grid, end_grid);
    }

    #[test]
    #[should_panic(expected = "grid index out of bounds: (2,0) out of (2,2)")]
    fn swap_out_of_bounds() {
        let mut grid = grid![[1,2][4,5]];
        grid.swap((0, 0), (2, 0));
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn serialize() {
            let grid: Grid<u8> = grid![[1, 2][3, 4]];
            let s = serde_json::to_string(&grid).unwrap();
            assert_eq!(s, r#"{"cols":2,"data":[1,2,3,4],"order":"RowMajor"}"#);
        }

        #[test]
        fn deserialize() {
            let s = "{ \"cols\": 2, \"data\": [1, 2, 3, 4] }";
            let grid: Grid<u8> = serde_json::from_str(s).unwrap();
            assert_eq!(grid, grid![[1, 2][3, 4]]);
        }

        #[test]
        fn deserialize_with_order() {
            let s = "{ \"cols\": 2, \"data\": [1, 3, 2, 4], \"order\": \"ColumnMajor\" }";
            let grid: Grid<u8> = serde_json::from_str(s).unwrap();
            test_grid(&grid, 2, 2, Order::ColumnMajor, &[1, 3, 2, 4]);
        }
    }
}
