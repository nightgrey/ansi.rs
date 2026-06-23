use crate::{Nested, NestedConstructor, NestedMut};
use core::ops::Index;
use smallvec::SmallVec;
use std::ops::IndexMut;

/// An owned, growable container for groups of elements.
///
/// [`NestedSoaVec`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedSoaVec<T, const N: usize, const M: usize = N> {
    groups: SmallVec<usize, N>,
    inner: SmallVec<T, N>,
}

impl<T, const N: usize, const M: usize> NestedSoaVec<T, N, M> {
    #[inline]
    /// Creates a `NestedSoaVec` with the specified capacity for total elements.
    /// The `starts` array will have `capacity + 1` entries (one more than elements).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SmallVec::with_capacity(capacity),
            groups: SmallVec::with_capacity(capacity + 1),
        }
    }

    pub fn get(&self, index: usize) -> Option<&[T]> {
        let offset = self.groups[..index].iter().copied().sum();
        let len = self.groups[index];
        Some(&self.inner[offset..offset + len])
    }
}

impl<T, const N: usize, const M: usize> NestedConstructor<T> for NestedSoaVec<T, N, M> {
    #[inline]
    fn new() -> Self {
        Self {
            inner: SmallVec::new(),
            groups: SmallVec::new(),
        }
    }
}

impl<T, const N: usize, const M: usize> Index<usize> for NestedSoaVec<T, N, M> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T, const N: usize, const M: usize> IndexMut<usize> for NestedSoaVec<T, N, M> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<T, const N: usize, const M: usize> AsRef<[T]> for NestedSoaVec<T, N, M> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

impl<T, const N: usize, const M: usize> Nested<T> for NestedSoaVec<T, N, M> {
    #[inline]
    fn first(&self) -> Option<&[T]> {
        self.get(0)
    }

    #[inline]
    fn last(&self) -> Option<&[T]> {
        self.get(self.groups.len().saturating_sub(1).saturating_sub(1))
    }

    #[inline]
    fn len(&self) -> usize {
        self.groups.len().saturating_sub(1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    fn values(&self) -> &[T] {
        &self.inner
    }

    fn starts(&self) -> &[usize] {
        &self.groups
    }
}
impl<T, const N: usize, const M: usize> NestedMut<T> for NestedSoaVec<T, N, M> {
    #[inline]
    fn push(&mut self, items: impl IntoIterator<Item = T>) {
        self.inner.extend(items);
        self.groups.push(self.inner.len());
    }

    #[inline]
    fn push_one(&mut self, val: T) {
        self.inner.push(val);
        self.groups.push(1);
    }

    fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        let group_len = match self.groups.last_mut() {
            Some(last_len) => last_len,
            None => {
                self.groups.push(0);
                &mut self.groups[0]
            }
        };
        self.inner.extend(iter);
        *group_len = self.inner.len() - *group_len;
    }
    fn extend_one(&mut self, item: T) {
        let group_len = match self.groups.last_mut() {
            Some(last_len) => last_len,
            None => {
                self.groups.push(0);
                &mut self.groups[0]
            }
        };

        self.inner.push(item);

        *group_len += 1;
    }

    fn values_mut(&mut self) -> &mut [T] {
        &mut self.inner
    }

    fn starts_mut(&mut self) -> &mut [usize] {
        &mut self.groups
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
        self.groups.clear();
    }
}

impl<T, Group: IntoIterator<Item = T>, const N: usize, const M: usize> FromIterator<Group>
    for NestedSoaVec<T, N, M>
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

impl<T, const N: usize, const M: usize> Default for NestedSoaVec<T, N, M> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
