use core::ops::Index;
use std::ops::IndexMut;
use smallvec::SmallVec;
use thiserror::Error;
use crate::{NestedIter, NestedSlice};

// An owned, growable container for groups of elements.
///
/// [`NestedVec`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedVec<T, const N: usize = 8, const M: usize = N> {
    pub(super) starts: SmallVec<usize, N>,
    pub(super) inner: SmallVec<T, N>,
}

impl<T, const N: usize, const M: usize> NestedVec<T, N, M> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: SmallVec::new(),
            starts: SmallVec::new(),
        }
    }
    
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SmallVec::with_capacity(capacity),
            starts: SmallVec::with_capacity(capacity + 1),
        }
    }
    
    #[inline]
    pub fn from_nested_slice(params: &NestedSlice<T>) -> Self where T: Copy {
        NestedVec {
            inner: SmallVec::from_slice_copy(params.inner),
            starts: SmallVec::from_slice_copy(params.starts),
        }
    }
    
    #[inline]
    pub fn from_iter_flat(iter: impl IntoIterator<Item = T>) -> Self {
        let inner = SmallVec::from_iter(iter.into_iter());
        NestedVec {
            starts: SmallVec::from_iter(0..=inner.len()),
            inner,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&[T]> {
        if index >= self.starts.len() { return None; }
        Some(self.get_unchecked(index))
    }

    #[inline]
    pub fn get_unchecked(&self, index: usize) -> &[T] {
        let start = self.starts[index];
        let end = self.starts[index + 1];
        &self.inner[start..end]
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut [T]> {
        if index >= self.starts.len() { return None; }
        Some(self.get_unchecked_mut(index))
    }

    #[inline]
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut [T] {
        let start = self.starts[index];
        let end = self.starts[index + 1];
        &mut self.inner[start..end]
    }

    /// Starts a **new group** and appends multiple values.
    #[inline]
    pub fn push(&mut self, items: impl IntoIterator<Item = T>) {
        self.starts.push(self.inner.len());

        self.inner.extend(items);

        self.starts.push(self.inner.len());
    }

    /// Appends multiple values to the last group.
    #[inline]
    pub fn extend(&mut self, items: impl IntoIterator<Item = T>) {
        if self.starts.len() == 0 {
            self.starts.push(0);
        }

        self.inner.extend(items);

        self.starts.push(self.inner.len());
    }

    /// Starts a **new group** and appends a single value to it.
    /// Extends the `starts` boundary array.
    #[inline]
    pub fn push_one(&mut self, value: T) {
        self.starts.push(self.inner.len());

        self.inner.push(value);

        self.starts.push(self.inner.len());
    }

    /// Appends a single value to the **current (last) group**.
    /// Does NOT create a new boundary.
    #[inline]
    pub fn extend_one(&mut self, value: T) {
        if self.starts.len() == 0 {
            self.starts.push(0);
        }

        self.inner.push(value);

        self.starts.push(self.inner.len());
    }

    #[inline]
    pub fn len(&self) -> usize { self.starts.len() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.starts.len() == 0 }

    #[inline]
    pub fn iter(&self) -> NestedIter<T> {
        NestedIter::new(&self.as_nested_slice())
    }

    #[inline]
    pub fn iter_flat(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }
    
    #[inline]
    pub fn first(&self) -> Option<&[T]> { self.get(0) }

    #[inline]
    pub fn last(&self) -> Option<&[T]> { self.get(self.len().saturating_sub(1)) }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.starts.clear();
    }

    #[inline]
    pub fn as_nested_slice(&self) -> NestedSlice<T> {
        NestedSlice {
            starts: &self.starts,
            inner: &self.inner,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }
}

impl<T, const N: usize, const M: usize> Default for NestedVec<T, N, M> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize, const M: usize> Index<usize> for NestedVec<T, N, M> {
    type Output = [T];
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get_unchecked(index)
    }
}

impl<T, const N: usize, const M: usize> IndexMut<usize> for NestedVec<T, N, M> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_unchecked_mut(index)
    }
}

impl<T, const N: usize, const M: usize> Extend<T> for NestedVec<T, N, M> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        Self::extend(self, iter);
    }
    fn extend_one(&mut self, item: T) {
        Self::extend_one(self, item);
    }
}

impl<T, const N: usize, const M: usize> AsRef<[T]> for NestedVec<T, N, M> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

// impl<'a, T, U: IntoIterator<Item = T> , const N: usize, const M: usize> FromIterator<&'a U> for NestedVec<T, N, M> {
//     fn from_iter<I: IntoIterator<Item = &'a U>>(iter: I) -> Self {
//         let iter = iter.into_iter();
//         let mut nested = match iter.size_hint() {
//             (0, None) => NestedVec::default(),
//             (_, Some(l)) | (l, None) => NestedVec::<T, N, M>::with_capacity(l),
//         };
//
//         for item in iter {
//             nested.push(item);
//         }
//
//         nested
//     }
// }
impl<'a, T, const U: usize, const N: usize, const M: usize> FromIterator<[T; U]> for NestedVec<T, N, M> {
    fn from_iter<I: IntoIterator<Item = [T; U]>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut nested = match iter.size_hint() {
            (0, None) => NestedVec::default(),
            (_, Some(l)) | (l, None) => NestedVec::<T, N, M>::with_capacity(l),
        };

        for group in iter {
            nested.push(group)
        }

        nested
    }
}
impl<'a, T: Clone, const N: usize, const M: usize> FromIterator<&'a [T]> for NestedVec<T, N, M> {
    fn from_iter<I: IntoIterator<Item = &'a [T]>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut nested = match iter.size_hint() {
            (0, None) => NestedVec::default(),
            (_, Some(l)) | (l, None) => NestedVec::<T, N, M>::with_capacity(l),
        };

        for group in iter {
            nested.push(group.iter().cloned())
        }
        nested
    }
}

#[derive(Error, Debug)]
enum ParamsError {
    #[error("Params: Overflow")]
    Overflow,
}
