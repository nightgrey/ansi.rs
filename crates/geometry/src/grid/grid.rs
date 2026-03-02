use std::ops;
use std::slice::{ChunksExact, SliceIndex};
use crate::{SpatialIndex};
use derive_more::{AsMut, AsRef, Deref, DerefMut, IntoIterator};
use crate::{Bounds, Position};

#[derive(Debug, Clone, Eq, PartialEq, Deref, DerefMut, IntoIterator, AsRef, AsMut)]
pub struct Grid<T> {
    #[deref]
    #[deref_mut]
    #[into_iterator(owned, ref, ref_mut)]
    #[as_ref(forward)]
    #[as_mut(forward)]
    pub inner: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl<T> Grid<T> {
    pub const EMPTY: Self = Self {
        inner: Vec::new(),
        width: 0,
        height: 0,
    };

    pub fn empty() -> Self {
        Self::EMPTY
    }

    pub fn rows(&self) -> ChunksExact<'_, T> {
        self.inner.chunks_exact(self.width)
    }

    /// Create a new, filled grid with the given width and height.
    pub fn new(width: usize, height: usize) -> Self where T: Default + Copy {
        Self {
            inner: vec![T::default(); width * height],
            width,
            height,
        }
    }

    /// Create a new, empty grid with a Vec with the given capacity.
    pub fn with_capacity(width: usize, height: usize) -> Self {
        Self {
            inner: Vec::with_capacity(width * height),
            width,
            height,
        }
    }

    /// Create a clipped version of `self`.
    ///
    /// Note: The new [`Grid`] does not share any content.
    pub fn clipped(&mut self, bounds: Bounds) -> Self {
        Self::from(self.clip(bounds))
    }

    pub fn min(&self) -> Position {
        Position::ZERO
    }

    pub fn max(&self) -> Position {
        Position::new(self.height, self.width)
    }

    /// Returns the bounds of this grid.
    pub fn bounds(&self) -> Bounds {
        Bounds::new(Position::ZERO, Position::new(self.height, self.width))
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    pub fn get<I: SpatialIndex<T>>(&self, index: I) -> Option<&<I::Index as SliceIndex<[T]>>::Output>
    {
        index.get(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: SpatialIndex<T>>(&mut self, index: I) -> Option<&mut <I::Index as SliceIndex<[T]>>::Output>
    {
        index.get_mut(self)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked<I: SpatialIndex<T>>(&self, index: I) -> *const <I::Index as SliceIndex<[T]>>::Output
    {
        index.get_unchecked(self)
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<I: SpatialIndex<T>>(&mut self, index: I) -> *mut <I::Index as SliceIndex<[T]>>::Output
    {
        index.get_unchecked_mut(self)
    }

    pub fn intersect(&self, other: Bounds) -> Bounds {
        let min_row = 0.max(other.min.row);
        let min_col = 0.max(other.min.col);
        let max_row = self.height.min(other.max.row);
        let max_col = self.width.min(other.max.col);

        // Clamp to empty if min overtakes max on either axis.
        let (max_row, max_col) = if min_row > max_row || min_col > max_col {
            (min_row, min_col)
        } else {
            (max_row, max_col)
        };

        Bounds { min: Position::new(min_row, min_col), max: Position::new(max_row, max_col) }
    }

    pub fn clip(&self, other: Bounds) -> Bounds {
        let min_row = other.min.row.max(0);
        let min_col = other.min.col.max(0);
        let max_row = other.max.row.min(self.height);
        let max_col = other.max.col.min(self.width);

        // Clamp to empty if min overtakes max on either axis.
        let (max_row, max_col) = if min_row > max_row || min_col > max_col {
            (min_row, min_col)
        } else {
            (max_row, max_col)
        };

        Bounds { min: Position::new(min_row, min_col), max: Position::new(max_row, max_col) }
    }

    pub fn fill_area(&mut self, bounds: Bounds, value: T) where T: Copy {
        for pos in &self.clip(bounds) {
            self[pos] = value;
        }
    }

    pub fn clear_and_resize(&mut self, width: usize, height: usize) where T: Default + Clone {
        self.width = width;
        self.height = height;
        self.inner.clear();
        self.inner.resize(width * height, T::default());
    }
}

impl<T: Clone> Grid<T> {
    pub fn clone_from_region(&mut self, bounds: Bounds) -> Self {
        let mut next = self.clipped(bounds);

        for position in &bounds {
            next[(position.row - bounds.min.row, position.col - bounds.min.col)] = self[position].clone();
        }

        next
    }
}
impl<T: Copy> Grid<T> {
    pub fn copy_from_region(&mut self, bounds: Bounds) -> Self {
        let mut next = self.clipped(bounds);

        for position in &bounds {
            next[(position.row - bounds.min.row, position.col - bounds.min.col)] = self[position];
        }

        next
    }

    pub fn resize_with(&mut self, width: usize, height: usize, value: T) where T: Clone {
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

    /// Resize the buffer to the given width and height.
    pub fn resize(&mut self, width: usize, height: usize) where T: Default {
        self.resize_with(width, height, T::default());
    }
}

impl<T, I: SpatialIndex<T>>  ops::Index<I> for Grid<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}
impl<T, I: SpatialIndex<T>>  ops::IndexMut<I> for Grid<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(self)
    }
}

impl<T> From<Bounds> for Grid<T> {
    fn from(value: Bounds) -> Self {
        Grid::with_capacity(value.width(), value.height())
    }
}