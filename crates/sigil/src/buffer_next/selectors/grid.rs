use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Grid<T> {
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

    // /// Returns a shared reference to the output at this location, if in
    // /// bounds.
    // pub fn get<I: Index>(&self, index: I) -> Option<&<I::Index as SliceIndex<[T]>>::Output>
    // {
    //     index.get(self)
    // }
    //
    // /// Returns a mutable reference to the output at this location, if in
    // /// bounds.
    // pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut <I::Index as SliceIndex<[T]>>::Output>
    // {
    //     index.index_of(self).get_mut(self.as_mut_slice())
    // }
    //
    // /// Returns a pointer to the output at this location, without
    // /// performing any bounds checking.
    // ///
    // /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    // /// is *[undefined behavior]* even if the resulting pointer is not used.
    // ///
    // /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    // pub unsafe fn get_unchecked<I: Index>(&self, index: I) -> *const <I::Index as SliceIndex<[T]>>::Output
    // {
    //     index.index_of(self).get_unchecked(self.as_slice())
    // }
    //
    // /// Returns a mutable pointer to the output at this location, without
    // /// performing any bounds checking.
    // ///
    // /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    // /// is *[undefined behavior]* even if the resulting pointer is not used.
    // ///
    // /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    // pub unsafe fn get_unchecked_mut<I: Index>(&mut self, index: I) -> *mut <I::Index as SliceIndex<[T]>>::Output
    // {
    //     index.get_unchecked_mut(self)
    // }
}

impl<T: Default + Clone> Grid<T> {
    pub fn default(width: usize, height: usize) -> Self {
        Self {
            inner: vec![T::default(); width * height],
            width,
            height,
        }
    }
}

impl<T: Clone> Grid<T> {
    pub fn new(width: usize, height: usize, value: T) -> Self {
        Self {
            inner: vec![value; width * height],
            width,
            height,
        }
    }
}
impl<T> const Deref for Grid<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> const DerefMut for Grid<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl<T> const AsRef<[T]> for Grid<T> {
    fn as_ref(&self) -> &[T] {
        self.inner.as_slice()
    }
}
impl<T> const AsMut<[T]> for Grid<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.inner.as_mut_slice()
    }
}