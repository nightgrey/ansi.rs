use crate::{Nested, NestedConstructor, NestedMut};
use core::ops::Index;
use smallvec::SmallVec;
use std::fmt::Debug;
use std::ops::IndexMut;

/// An owned, growable container for groups of elements.
///
/// [`NestedVec`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Clone, PartialEq, Eq)]
pub struct NestedVec<T, const N: usize, const M: usize = N> {
    pub(crate) starts: SmallVec<usize, N>,
    pub(crate) inner: SmallVec<T, N>,
}

impl<T, const N: usize, const M: usize> NestedVec<T, N, M> {
    #[inline]
    /// Creates a `NestedVec` with the specified capacity for total elements.
    /// The `starts` array will have `capacity + 1` entries (one more than elements).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SmallVec::with_capacity(capacity),
            starts: SmallVec::with_capacity(capacity + 1),
        }
    }

    #[inline]
    pub fn from_values(values: impl IntoIterator<Item = T>) -> Self {
        let inner = SmallVec::from_iter(values);
        Self {
            starts: SmallVec::from_iter(0..=inner.len()),
            inner,
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
    fn first(&self) -> Option<&[T]> {
        self.get(0)
    }

    #[inline]
    fn last(&self) -> Option<&[T]> {
        self.get(self.starts.len().saturating_sub(1).saturating_sub(1))
    }

    #[inline]
    fn len(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.starts.is_empty()
    }

    fn values(&self) -> &[T] {
        &self.inner
    }

    fn starts(&self) -> &[usize] {
        &self.starts
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

    fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        if self.starts.is_empty() {
            self.starts.push(0);
        }

        self.inner.extend(iter);

        if self.starts.len() == 1 {
            self.starts.push(self.inner.len());
        } else {
            *self.starts.last_mut().unwrap() = self.inner.len();
        }
    }
    fn extend_one(&mut self, item: T) {
        if self.starts.is_empty() {
            self.starts.push(0);
        }

        self.inner.push(item);

        if self.starts.len() == 1 {
            self.starts.push(self.inner.len());
        } else {
            *self.starts.last_mut().unwrap() = self.inner.len();
        }
    }

    fn values_mut(&mut self) -> &mut [T] {
        &mut self.inner
    }

    fn starts_mut(&mut self) -> &mut [usize] {
        &mut self.starts
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear();
        self.starts.clear();
    }
}

impl<T, Group: IntoIterator<Item = T>, const N: usize, const M: usize> FromIterator<Group>
    for NestedVec<T, N, M>
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = Group>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut nested = match iter.size_hint() {
            (0, None) => Self::new(),
            (_, Some(l)) | (l, None) => Self::with_capacity(l),
        };

        for group in iter {
            nested.push(group);
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

impl<T: Debug, const N: usize, const M: usize> Debug for NestedVec<T, N, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_nested_slice().fmt(f)
    }
}
