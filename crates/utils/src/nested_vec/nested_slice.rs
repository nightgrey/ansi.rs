use core::ops::Index;
use thiserror::Error;
use super::{NestedVec, NestedIter};

// An owned, growable container for groups of elements.
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

impl<'a, T>   NestedSlice<'a, T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            starts: &[],
            inner: &[],
        }
    }

    #[inline]
    pub const unsafe fn from_parts(starts: &'a [usize], inner: &'a [T]) -> Self {
        Self {
            starts,
            inner,
        }
    }
    #[inline]
    pub const fn from_nested_vec(params: &'a NestedVec<T>) -> Self {
        Self {
            starts: params.starts.as_slice(),
            inner: params.inner.as_slice(),
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
    pub fn len(&self) -> usize { self.starts.len().saturating_sub(1) }

    #[inline]
    pub fn is_empty(&self) -> bool { self.starts.len() == 0 }

    #[inline]
    pub fn iter(&self) -> NestedIter<T> {
        NestedIter::from_parts(self.starts, self.inner, 0, self.len())
    }

    #[inline]
    pub fn iter_flat(&self) -> std::slice::Iter<'a, T> {
        self.inner.iter()
    }

    #[inline]
    pub fn first(&self) -> Option<&[T]> { self.get(0) }

    #[inline]
    pub fn last(&self) -> Option<&[T]> { self.get(self.len().saturating_sub(1)) }

    #[inline]
    pub fn to_nested_vec<const N: usize, const M: usize>(&self) -> NestedVec<T, N, M> where T: Copy {
        NestedVec::from_nested_slice(self)
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<T> where T: Clone {
        self.inner.to_vec()
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
        self.get_unchecked(index)
    }
}

impl<'a, T> AsRef<[T]> for NestedSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

#[derive(Error, Debug)]
enum ParamsError {
    #[error("Params: Overflow")]
    Overflow,
}
