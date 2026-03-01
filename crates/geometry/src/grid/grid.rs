use std::ops;
use std::slice::{ChunksExact, SliceIndex};
use crate::{GridIndex};
use derive_more::{AsMut, AsRef, Deref, DerefMut, IntoIterator};
use crate::{Bounds, Position};

#[derive(Debug, Clone, Eq, PartialEq, Deref, DerefMut, IntoIterator, AsRef, AsMut)]
pub struct Grid<T> {
    #[deref]
    #[deref_mut]
    #[into_iterator(owned, ref, ref_mut)]
    #[as_ref(forward)]
    #[as_mut(forward)]
    inner: Vec<T>,
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

    /// Create a new grid with the given width and height filled with `T::default()`.
    pub fn new(width: usize, height: usize) -> Self where T: Default + Clone {
        Self {
            inner: vec![T::default(); width * height],
            width,
            height,
        }
    }

    /// Create a new grid with the given width and height filled with the given value.
    pub fn filled(width: usize, height: usize, value: T) -> Self where T: Clone {
        Self {
            inner: vec![value; width * height],
            width,
            height,
        }
    }

    /// Returns the bounds of this grid.
    pub fn bounds(&self) -> Bounds {
        Bounds::new(Position::ZERO, Position::new(self.height, self.width))
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    pub fn get<I: GridIndex<T>>(&self, index: I) -> Option<&<I::Index as SliceIndex<[T]>>::Output>
    {
        index.get(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: GridIndex<T>>(&mut self, index: I) -> Option<&mut <I::Index as SliceIndex<[T]>>::Output>
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
    pub unsafe fn get_unchecked<I: GridIndex<T>>(&self, index: I) -> *const <I::Index as SliceIndex<[T]>>::Output
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
    pub unsafe fn get_unchecked_mut<I: GridIndex<T>>(&mut self, index: I) -> *mut <I::Index as SliceIndex<[T]>>::Output
    {
        index.get_unchecked_mut(self)
    }

    pub fn fill_region(&mut self, bounds: Bounds, value: T) where T: Clone {
        for pos in &bounds {
            self[pos] = value.clone();
        }
    }
}

impl<T: Default> Grid<T> {

}

impl<T: Default + Clone> Grid<T> {
    pub fn clone_from_region(&mut self, bounds: Bounds) -> Self {
        let bounds = self.bounds().clip(bounds);
        let mut next = Self::new(bounds.width(), bounds.height());

        for position in &bounds {
            next[(position.row - bounds.min.row, position.col - bounds.min.col)] = self[position].clone();
        }

        next
    }

}
impl<T: Default + Copy> Grid<T> {
    /// Resize the buffer to the given width and height.
    pub fn resize(&mut self, width: usize, height: usize) {
        let (cur_w, cur_h) = (self.width, self.height);
        if cur_w == width && cur_h == height {
            return;
        }

        if width != cur_w {
            let copy_w = width.min(cur_w);

            if width > cur_w {
                // Growing: extend first, then shift rows back-to-front
                self.inner.resize(width * cur_h, T::default());
                for y in (1..cur_h).rev() {
                    let src = y * cur_w;
                    let dst = y * width;
                    self.inner.copy_within(src..src + copy_w, dst);
                    // Fill the new columns
                    &mut self.inner[dst + copy_w..dst + width].fill(T::default());
                }
                // Row 0: just fill the tail
                &mut self.inner[copy_w..width].fill(T::default());
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
            self.inner.resize(width * height, T::default());
        } else if height < cur_h {
            self.inner.truncate(width * height);
        }
        self.height = height;
    }
}

impl<T, I: GridIndex<T>>  ops::Index<I> for Grid<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}
impl<T, I: GridIndex<T>>  ops::IndexMut<I> for Grid<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(self)
    }
}
