use smallvec::SmallVec;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{Deref, Index, Sub};
use std::slice::Iter;
use std::usize;

/// An owned, growable nested structure that stores slices of `T` elements.
///
/// `NestedVec` owns its contents and can grow and shrink, like a `Vec`. For a
/// borrowed nested structure, see [`NestedSlice`].
///
/// `NestedVec` implements `Deref` to `NestedSlice`, so all methods available on
/// `NestedSlice` are available on `NestedVec`.
#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Eq)]
pub struct NestedVec<T, const N: usize = 8> {
    indices: SmallVec<usize, N>,
    values: SmallVec<T, N>,
}

impl<T, const N: usize> NestedVec<T, N> {
    pub const DEFAULT: Self = Self {
        indices: SmallVec::<usize, N>::from_buf([0]),
        values: SmallVec::<T, N>::new(),
    };

    pub fn with_capacity(capacity: usize) -> Self {
        NestedVec {
            indices: {
                let mut indices = SmallVec::with_capacity(capacity.max(1));

                indices.push(0);
                indices
            },
            values: SmallVec::with_capacity(capacity),
        }
    }

    /// Returns a shared reference to the output at this location, if in bounds.
    pub fn get(&self, index: usize) -> Option<&[T]> {
        if index >= self.len() {
            None
        } else {
            unsafe { Some(self.get_unchecked(index)) }
        }
    }

    pub fn get_unchecked(&self, index: usize) -> &[T] {
        unsafe {
            let start = *self.indices.get_unchecked(index);
            let end = *self.indices.get_unchecked(index + 1);

            self.values.get_unchecked(start..end)
        }
    }

    pub fn push(&mut self, group: impl IntoIterator<Item = T>) {
        self.values.extend(group);
    }

    pub fn push_one(&mut self, item: T) {
        self.values.push(item);
    }

    pub fn separate(&mut self) {
        if self.values.len() == self.indices.len() - 1 {
            return;
        }

        self.indices.push(self.values.len());
    }

    pub fn len(&self) -> usize {
        self.indices.len().saturating_sub(1)
    }

    pub fn len_values(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Returns true if self has a length of zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.indices.reserve(additional + 1);
        self.values.reserve(additional);
    }

    /// Returns an iterator over the groups.
    pub fn iter(&self) -> NestedIter<'_, T, N> {
        NestedIter::new(self)
    }

    /// Returns an iterator over all values.
    pub fn iter_values(&self) -> Iter<'_, T> {
        self.values.iter()
    }

    /// Truncates this `Nested`, removing all contents.
    ///
    /// While this means the `Nested` will have a length of zero, it does not touch its capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
        self.indices.clear();
        self.indices[0] = 0;
    }

    #[inline]
    pub fn as_slice(&self) -> NestedSlice<'_, T> {
        NestedSlice {
            indices: &self.indices,
            values: &self.values,
        }
    }

    pub fn as_indices(&self) -> &[usize] {
        self.indices.as_slice()
    }

    pub fn as_values(&self) -> &[T] {
        self.values.as_slice()
    }

    fn from_iter_nested<I: IntoIterator<Item = Sub>, Sub: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();

        let mut nested = match iter.size_hint() {
            (0, None) => NestedVec::default(),
            (_, Some(l)) | (l, None) => NestedVec::<T, N>::with_capacity((l * 2).max(N)),
        };

        iter.for_each(|sub| {
            nested.push(sub);
            dbg!(nested.indices.len());
            nested.separate();
        });

        nested
    }
}

impl<T, const N: usize> Default for NestedVec<T, N> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<T, const N: usize> Index<usize> for NestedVec<T, N> {
    type Output = [T];
    fn index(&self, index: usize) -> &Self::Output {
        self.get_unchecked(index)
    }
}

impl<T, const N: usize> Extend<T> for NestedVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.push(iter);
    }

    fn extend_one(&mut self, item: T) {
        self.push_one(item);
    }
}

#[derive(Debug, Clone)]
pub struct NestedIter<'a, T: 'a, const N: usize> {
    values: &'a [T],
    windows: std::slice::Windows<'a, usize>,
}

impl<'a, T, const N: usize> NestedIter<'a, T, N> {
    pub fn new(nested: &'a NestedVec<T, N>) -> Self {
        Self {
            windows: nested.indices.windows(2),
            values: nested.values.as_slice(),
        }
    }
}

impl<'a, T, const N: usize> Iterator for NestedIter<'a, T, N> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.windows.next().map(|w| &self.values[w[0]..w[1]])
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.windows.size_hint()
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for NestedIter<'a, T, N> {
    fn len(&self) -> usize {
        self.windows.len()
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for NestedIter<'a, T, N> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        self.windows.next_back().map(|w| &self.values[w[0]..w[1]])
    }
}

impl<T, const N: usize> FromIterator<T> for NestedVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut nested = match iter.size_hint() {
            (0, None) => NestedVec::default(),
            (_, Some(l)) | (l, None) => NestedVec::<T, N>::with_capacity(l),
        };

        nested.extend(iter);
        nested
    }
}

/// A borrowed nested structure - immutable view into nested data.
///
/// This is a lightweight wrapper around two slices that provides
/// convenient access to nested slice structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NestedSlice<'a, T> {
    indices: &'a [usize],
    values: &'a [T],
}

impl<'a, T> NestedSlice<'a, T> {
    /// Creates a new `NestedSlice` from indices and values.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if invariants are violated:
    /// - indices must not be empty
    /// - first index must be 0
    /// - last index must equal values.len()
    #[inline]
    pub const fn new(indices: &'a [usize], values: &'a [T]) -> Self {
        if indices[0] != 0 {
            panic!("first index must be 0");
        }
        NestedSlice { indices, values }
    }

    /// Returns the number of slices in this nested structure.
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len().saturating_sub(1)
    }

    /// Returns the total number of elements across all slices.
    #[inline]
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// Returns true if there are no slices.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the slice at the given index, if in bounds.
    pub fn get(&self, index: usize) -> Option<&'a [T]> {
        if index >= self.len() {
            None
        } else {
            // SAFETY: We've checked bounds above
            unsafe { Some(self.get_unchecked(index)) }
        }
    }

    /// Returns a reference to the slice at the given index without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &'a [T] {
        let start = *self.indices.get_unchecked(index);
        let end = *self.indices.get_unchecked(index + 1);
        self.values.get_unchecked(start..end)
    }

    /// Returns an iterator over the slices.
    pub fn iter(&self) -> NestedSliceIterator<'a, T> {
        NestedSliceIterator {
            values: self.values,
            windows: self.indices.windows(2),
        }
    }

    /// Returns the values slice.
    pub fn values(&self) -> &[T] {
        self.values
    }

    /// Returns the indices slice.
    pub fn indices(&self) -> &[usize] {
        self.indices
    }

    /// Returns the underlying values as a single slice.
    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        self.values
    }

    /// Returns the indices slice.
    #[inline]
    pub fn as_indices(&self) -> &[usize] {
        self.indices
    }

    pub fn as_values(&self) -> &[T] {
        self.values
    }
}

impl<'a, T: Clone> NestedSlice<'a, T> {
    pub fn to_vec<const N: usize>(&self) -> NestedVec<T, N> {
        NestedVec {
            indices: SmallVec::from_iter(self.indices.iter().copied()),
            values: SmallVec::from_iter(self.values.iter().cloned()),
        }
    }
}

impl<'a, T> Index<usize> for NestedSlice<'a, T> {
    type Output = [T];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<'a, T> AsRef<[T]> for NestedSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.values
    }
}

#[derive(Debug, Clone)]
pub struct NestedSliceIterator<'a, T> {
    values: &'a [T],
    windows: std::slice::Windows<'a, usize>,
}

impl<'a, T> Iterator for NestedSliceIterator<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.windows
            .next()
            .map(|window| &self.values[window[0]..window[1]])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.windows.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for NestedSliceIterator<'a, T> {
    fn len(&self) -> usize {
        self.windows.len()
    }
}

impl<'a, T> DoubleEndedIterator for NestedSliceIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.windows
            .next_back()
            .map(|window| &self.values[window[0]..window[1]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        let mut nested = NestedVec::<u8, 4>::default();
        nested.extend([1]);
        nested.extend([2, 3]);
        nested.extend([4]);

        assert_eq!(nested.len(), 3);
    }

    #[test]
    fn test_count() {
        let mut nested = NestedVec::<u8, 4>::default();
        nested.extend([1]);
        nested.extend([2, 3]);
        nested.extend([4]);

        assert_eq!(nested.len_values(), 4);
    }

    #[test]
    fn test_iter() {
        let mut nested = NestedVec::<u8, 4>::default();
        nested.extend([5]);
        nested.extend([6]);
        nested.extend([7]);
        nested.extend([8, 9, 10, 12]);

        let mut iter = nested.iter();

        assert_eq!(iter.next().unwrap(), &[5]);
        assert_eq!(iter.next().unwrap(), &[6]);
        assert_eq!(iter.next().unwrap(), &[7]);
        assert_eq!(iter.next().unwrap(), &[8, 9, 10, 12]);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn from_iter() {
        let from_iter_nested: NestedVec<i32> =
            NestedVec::from_iter_nested(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);

        assert_eq!(from_iter_nested.len(), 3);
        assert_eq!(from_iter_nested.len_values(), 9);

        let from_iter: NestedVec<i32> = NestedVec::from_iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);

        assert_eq!(from_iter.len(), 1);
        assert_eq!(from_iter.len_values(), 9);
    }
}
