use std::ops::{Index, IndexMut};
use smallvec::SmallVec;
use crate::{Nested, NestedError, NestedIter, NestedMut, NestedSlice, NestedVec};

/// A borrowed view into a nested collection.
///
/// [`NestedSliceMut`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Debug, PartialEq, Eq)]
pub struct NestedSliceMut<'a, T> {
    pub(super) starts: &'a mut [usize],
    pub(super) inner: &'a mut [T],
}

impl<'a, T> NestedSliceMut<'a, T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            starts: &mut [],
            inner: &mut [],
        }
    }

    #[inline]
    pub const unsafe fn from_parts(values: &'a mut [T], starts: &'a mut [usize]) -> Self {
        debug_assert!(
            if values.len() == 0 {
                starts.len() == 0
            } else {
                starts.len() == values.len() + 1
            },
            "Invalid parts for nested slice"
        );
        Self {
            starts,
            inner: values,
        }
    }
}

impl<'a, T> Nested<T> for NestedSliceMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.starts.len() == 0
    }

    #[inline]
    fn iter(&self) -> NestedIter<T> {
        NestedIter::from_parts(self.starts, self.inner, 0, self.len())
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
        self.get(self.len().saturating_sub(1))
    }

    #[inline]
    fn as_slice(&self) -> &[T] {
        self.inner
    }

    fn as_slices(&self) -> (&[T], &[usize]) {
        (self.inner, self.starts)
    }

    fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    fn as_ptrs(&self) -> (*const T, *const usize) {
        (self.inner.as_ptr(), self.starts.as_ptr())
    }

    #[inline]
    fn as_nested_slice(&self) -> NestedSlice<'_, T> {
        NestedSlice {
            starts: self.starts,
            inner: self.inner,
        }
    }

    #[inline]
    fn to_nested_vec<const N: usize, const M: usize>(&self) -> NestedVec<T, N, M>
    where
        T: Clone,
    {
        NestedVec {
            inner: SmallVec::from_iter(self.inner.iter().cloned()),
            starts: SmallVec::from_iter(self.starts.iter().cloned()),
        }
    }
}

impl<'a, T> NestedMut<T> for NestedSliceMut<'a, T> {
    #[inline]
    fn push(&mut self, items: impl IntoIterator<Item = T>) {
        if self.starts.len() == 0 {
            return self.push(items);
        }
        for item in items {
            self.inner[self.inner.len()] = item;
        }

        self.starts[self.starts.len() - 1] = self.inner.len();
    }

    #[inline]
    fn push_one(&mut self, val: T) {
        if self.starts.len() == 0 {
            self.starts[0] = 0;
        }

        self.inner[self.inner.len()] = val;
        // self.inner.len += 1;

        self.starts[self.starts.len()] = self.inner.len();
        // self.starts.len += 1;
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

    fn as_mut_nested_slice(&mut self) -> NestedSliceMut<'_, T> {
        NestedSliceMut {
            starts: self.starts,
            inner: self.inner,
        }
    }


    #[inline]
    fn clear(&mut self) where T: Clone {
        self.inner.clear();
        self.starts.clear();
    }
}

impl<'a, T> Default for NestedSliceMut<'a, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: &[],
            starts: &[],
        }
    }
}

impl<'a, T> Index<usize> for NestedSliceMut<'a, T> {
    type Output = [T];
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<'a, T> IndexMut<usize> for NestedSliceMut<'a, T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<'a, T> AsRef<[T]> for NestedSliceMut<'a, T> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}
impl<'a, T> Extend<T> for NestedSliceMut<'a, T> {
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
