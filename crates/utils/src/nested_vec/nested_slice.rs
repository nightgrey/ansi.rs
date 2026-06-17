use crate::Nested;
use core::ops::Index;

/// A borrowed view into a nested collection.
///
/// [`NestedSlice`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedSlice<'a, T> {
    pub(super) values: &'a [T],
    pub(super) starts: &'a [usize],
}

impl<'a, T> NestedSlice<'a, T> {
    #[inline]
    pub fn from_nested(nested: &'a impl Nested<T>) -> Self {
        unsafe { Self::from_parts(nested.values(), nested.starts()) }
    }

    #[inline]
    pub fn from_parts(values: &'a [T], starts: &'a [usize]) -> Self {
        debug_assert!(
            if starts.is_empty() {
                values.is_empty()
            } else {
                starts.last().copied() == Some(values.len())
            },
            "Invalid parts for nested slice. Starts: {:?}, Values: {:?}",
            starts,
            values.len()
        );
        Self { starts, values }
    }
}

impl<'a, T> Nested<T> for NestedSlice<'a, T> {
    #[inline]
    fn first(&self) -> Option<&[T]> {
        self.get(0)
    }

    #[inline]
    fn last(&self) -> Option<&[T]> {
        self.get(self.len().saturating_sub(1))
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
        self.values
    }

    fn starts(&self) -> &[usize] {
        self.starts
    }
}

impl<'a, T> Default for NestedSlice<'a, T> {
    #[inline]
    fn default() -> Self {
        Self {
            values: &[],
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
        self.values
    }
}

impl<'a, T, N: Nested<T>> From<&'a N> for NestedSlice<'a, T> {
    #[inline]
    fn from(nested: &'a N) -> Self {
        Self::from_nested(nested)
    }
}
