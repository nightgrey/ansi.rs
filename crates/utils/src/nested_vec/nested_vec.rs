use crate::{Nested, NestedConstructor, NestedFromIterator, NestedIndex, NestedIter, NestedMut, NestedSlice, TryNestedMut};
use core::ops::Index;
use std::ops::IndexMut;
use smallvec::SmallVec;

/// An owned, growable container for groups of elements.
///
/// [`NestedVec`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedVec<T, const N: usize, const M: usize = N> {
    pub starts: SmallVec<usize, N>,
    pub(super) inner: SmallVec<T, N>,
}

impl<T, const N: usize, const M: usize> NestedVec<T, N, M> {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SmallVec::with_capacity(capacity),
            starts: SmallVec::with_capacity(capacity + 1),
        }
    }
}

impl<T, const N: usize, const M: usize> NestedConstructor<T> for NestedVec<T, N, M> {
    #[inline]
    fn new() -> Self {
        Self {
            inner: SmallVec::new(),
            starts: SmallVec::new(),
        }
    }
}


impl<T, const N: usize, const M: usize> Index<usize> for NestedVec<T, N, M> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T, const N: usize, const M: usize> IndexMut<usize> for NestedVec<T, N, M> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<T, const N: usize, const M: usize> AsRef<[T]> for NestedVec<T, N, M> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

impl<T, const N: usize, const M: usize> Nested<T> for NestedVec<T, N, M> {
    #[inline]
    fn len(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.starts.len() == 0
    }

    #[inline]
    fn iter(&self) -> NestedIter<'_, T> {
        NestedIter::from_parts(
            &self.starts,
            &self.inner,
            0,
            self.starts.len().saturating_sub(1),
        )
    }

    #[inline]
    fn iter_flat(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }

    #[inline]
    fn first(&self) -> Option<&[T]> {
        self.get(0)
    }

    #[inline]
    fn last(&self) -> Option<&[T]> {
        self.get(self.starts.len().saturating_sub(1).saturating_sub(1))
    }

    #[inline]
    fn as_slice(&self) -> &[T] {
        &self.inner
    }

    fn as_slices(&self) -> (&[T], &[usize]) {
        (&self.inner, &self.starts)
    }

    fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    fn as_ptrs(&self) -> (*const T, *const usize) {
        (self.inner.as_ptr(), self.starts.as_ptr())
    }

    fn as_nested_slice(&self) -> NestedSlice<'_, T> {
        NestedSlice {
            starts: &self.starts,
            inner: &self.inner,
        }
    }

    #[inline]
    fn to_nested_vec<const N2: usize, const M2: usize>(&self) -> NestedVec<T, N2, M2>
    where
        T: Clone,
    {
        NestedVec {
            inner: SmallVec::from(self.inner.as_slice()),
            starts: SmallVec::from(self.starts.as_slice()),
        }
    }
}
impl<T, const N: usize, const M: usize> NestedMut<T> for NestedVec<T, N, M> {
    #[inline]
    fn push(&mut self, items: impl IntoIterator<Item = T>) {
        if self.starts.is_empty() {
            self.starts.push(0);
        }
        self.inner.extend(items);
        self.starts.push(self.inner.len());
    }

    #[inline]
    fn push_one(&mut self, val: T) {
        if self.starts.is_empty() {
            self.starts.push(0);
        }
        self.inner.push(val);
        self.starts.push(self.inner.len());
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
            &mut self.inner
    }

    fn as_mut_slices(&mut self) -> (&mut [T], &mut [usize]) {
        (&mut self.inner, &mut self.starts)
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    fn as_ptrs(&mut self) -> (*mut T, *mut usize) {
        (self.inner.as_mut_ptr(), self.starts.as_mut_ptr())
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear();
        self.starts.clear();
    }
}

impl<T, const N: usize, const M: usize, Group: IntoIterator<Item = T>> NestedFromIterator<T, Group>
    for NestedVec<T, N, M>
{
}
impl<T, const N: usize, const M: usize, Group: IntoIterator<Item = T>> FromIterator<Group>
    for NestedVec<T, N, M>
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = Group>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut nested = match iter.size_hint() {
            (0, None) => NestedVec::new(),
            (_, Some(l)) | (l, None) => NestedVec::<T, N, M>::with_capacity(l),
        };

        for group in iter {
            nested.push(group)
        }

        nested
    }
}

impl<T, const N: usize, const M: usize> Default for NestedVec<T, N, M> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize, const M: usize> Extend<T> for NestedVec<T, N, M> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        if self.starts.len() == 0 {
            self.starts.push(0);
        }

        self.inner.extend(iter);

        self.starts.push(self.inner.len());
    }
    fn extend_one(&mut self, item: T) {
        if self.starts.len() == 0 {
            self.starts.push(0);
        }

        self.inner.push(item);

        self.starts.push(self.inner.len());
    }
}
