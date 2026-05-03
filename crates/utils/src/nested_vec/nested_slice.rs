use crate::{Nested, NestedIter, NestedVec};
use core::ops::Index;
use smallvec::SmallVec;

/// A borrowed view into a nested collection.
///
/// [`NestedSlice`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedSlice<'a, T> {
    pub(super) starts: &'a [usize],
    pub(super) inner: &'a [T],
}

impl<'a, T> NestedSlice<'a, T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            starts: &[],
            inner: &[],
        }
    }

    #[inline]
    pub unsafe fn from_parts(values: &'a [T], starts: &'a [usize]) -> Self {
        debug_assert!(
            if starts.len() == 0 {
                values.len() == 0
            } else {
                starts.len() == values.len() + 1
            },
            "Invalid parts for nested slice. Starts: {:?}, Values: {:?}",
            starts,
            values.len()
        );
        Self {
            starts,
            inner: values,
        }
    }
}

impl<'a, T> Nested<T> for NestedSlice<'a, T> {
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
    fn iter_flat(&self) -> std::slice::Iter<'a, T> {
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
            inner: SmallVec::from(self.inner),
            starts: SmallVec::from(self.starts),
        }
    }
}

impl<'a, T> Default for NestedSlice<'a, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: &[],
            starts: &[],
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
        &self.inner
    }
}