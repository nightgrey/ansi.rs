use crate::nested_vec::error::NestedError;
use crate::{Nested, NestedConstructor, NestedMut, TryNestedMut};
use core::ops::Index;
use std::ops::IndexMut;

/// Stack-allocated nested parameter buffer optimized for ANSI escape sequences.
/// `N` = max total values across all groups
/// `M` = max number of groups
#[derive(Clone, Copy, Debug)]
pub struct NestedRaw<T, const N: usize = 8, const M: usize = 8> {
    inner: [T; N],
    starts: [usize; M],
    pub(super) inner_len: usize,
    pub(super) starts_len: usize,
}

impl<T: Default + Copy, const N: usize, const M: usize> NestedConstructor<T>
for NestedRaw<T, N, M>
{
    #[inline]
    fn new() -> Self {
        Self {
            inner: [Default::default(); N],
            starts: [0; M],
            inner_len: 0,
            starts_len: 0,
        }
    }
}

impl<T, const N: usize, const M: usize> Nested<T> for NestedRaw<T, N, M> {
    #[inline]
    fn first(&self) -> Option<&[T]> {
        self.get(0)
    }

    #[inline]
    fn last(&self) -> Option<&[T]> {
        self.get(self.starts_len.saturating_sub(1).saturating_sub(1))
    }

    #[inline]
    fn len(&self) -> usize {
        self.starts_len.saturating_sub(1)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.starts_len == 0
    }

    fn values(&self) -> &[T] {
        &self.inner[..self.inner_len]
    }

    fn starts(&self) -> &[usize] {
        &self.starts[..self.starts_len]
    }
}

impl<T, const N: usize, const M: usize> NestedMut<T> for NestedRaw<T, N, M> {
    fn push(&mut self, items: impl IntoIterator<Item = T>) {
        self.try_push(items).expect("could not push items");
    }

    fn push_one(&mut self, val: T) {
        self.try_push_one(val).expect("could not push value");
    }

    fn extend(&mut self, items: impl IntoIterator<Item = T>) {
        self.try_extend(items).expect("could not extend values");
    }

    fn extend_one(&mut self, value: T) {
        self.try_extend_one(value).expect("could not extend value");
    }

    fn values_mut(&mut self) -> &mut [T] {
        &mut self.inner[..self.inner_len]
    }

    fn starts_mut(&mut self) -> &mut [usize] {
        &mut self.starts[..self.starts_len]
    }

    #[inline]
    fn clear(&mut self) {
        self.inner_len = 0;
        self.starts_len = 0;
    }
}

impl<T, const N: usize, const M: usize> TryNestedMut<T> for NestedRaw<T, N, M> {
    #[inline]
    fn try_push(&mut self, items: impl IntoIterator<Item = T>) -> Result<(), NestedError> {
        if self.inner_len >= N || self.starts_len >= M {
            return Err(NestedError::Overflow);
        }

        if self.starts_len == 0 {
            self.starts[0] = 0;
            self.starts_len = 1;
        }

        for item in items {
            if self.inner_len >= N {
                self.starts[self.starts_len] = self.inner_len;
                return Err(NestedError::Overflow);
            }
            self.inner[self.inner_len] = item;
            self.inner_len += 1;
        }

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;
        Ok(())
    }

    #[inline]
    fn try_push_one(&mut self, val: T) -> Result<(), NestedError> {
        if self.starts_len >= M || self.inner_len >= N {
            return Err(NestedError::Overflow);
        }

        if self.starts_len == 0 {
            self.starts[0] = 0;
            self.starts_len = 1;
        }

        self.inner[self.inner_len] = val;
        self.inner_len += 1;

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;
        Ok(())
    }

    #[inline]
    fn try_extend<I: IntoIterator<Item = T>>(&mut self, items: I) -> Result<(), NestedError> {
        if self.starts_len >= M || self.inner_len >= N {
            return Err(NestedError::Overflow);
        }

        if self.starts_len == 0 {
            return self.try_push(items);
        }
        for item in items {
            if self.inner_len >= N {
                self.starts[self.starts_len - 1] = self.inner_len;
                return Err(NestedError::Overflow);
            }
            self.inner[self.inner_len] = item;
            self.inner_len += 1;
        }

        self.starts[self.starts_len - 1] = self.inner_len;
        Ok(())
    }

    #[inline]
    fn try_extend_one(&mut self, item: T) -> Result<(), NestedError> {
        if self.starts_len >= M || self.inner_len >= N {
            return Err(NestedError::Overflow);
        }

        if self.starts_len == 0 {
            return self.try_push_one(item);
        }

        self.inner[self.inner_len] = item;
        self.inner_len += 1;

        self.starts[self.starts_len - 1] = self.inner_len;
        Ok(())
    }
}

impl<T, const N: usize, const M: usize> Default for NestedRaw<T, N, M>
where
    T: Default + Copy,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: [Default::default(); N],
            starts: [0; M],
            inner_len: 0,
            starts_len: 0,
        }
    }
}

impl<T, const N: usize, const M: usize> Index<usize> for NestedRaw<T, N, M> {
    type Output = [T];
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T, const N: usize, const M: usize> IndexMut<usize> for NestedRaw<T, N, M> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<T, const N: usize, const M: usize> AsRef<[T]> for NestedRaw<T, N, M> {
    fn as_ref(&self) -> &[T] {
        &self.inner[..self.inner_len]
    }
}

impl<T, const N: usize, const M: usize> AsMut<[T]> for NestedRaw<T, N, M> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.inner[..self.inner_len]
    }
}

#[cfg(test)]
mod tests {
    use super::super::{NestedIter, NestedSlice};
    use super::*;

    const N: usize = 8;
    const M: usize = 8;

    #[test]
    fn test_push_one_creates_single_value_group() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        p.push_one(42);
        assert_eq!(p.starts_len.saturating_sub(1), 1);
        dbg!(&p.get(0));
        assert_eq!(&p[0], &[42]);
        assert_eq!(&p.inner[..p.inner_len], &[42]);
    }

    #[test]
    fn test_extend_one_appends_to_last_group() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        p.push_one(1);
        p.extend_one(2);
        assert_eq!(p.starts_len.saturating_sub(1), 1);
        assert_eq!(&p[0], &[1, 2]);
        assert_eq!(&p.inner[..p.inner_len], &[1, 2]);
        assert_eq!(
            NestedIter::from_parts(
                &p.starts[..p.starts_len],
                &p.inner[..p.inner_len],
                0,
                p.starts_len.saturating_sub(1)
            )
            .collect::<Vec<_>>(),
            vec![&[1u8, 2] as &[u8]]
        );
    }

    #[test]
    fn test_extend_one_on_empty_starts_new_group() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        p.extend_one(99);
        assert_eq!(p.starts_len.saturating_sub(1), 1);
        assert_eq!(&p[0], &[99]);
        assert_eq!(&p.inner[..p.inner_len], &[99]);
    }

    #[test]
    fn test_push_adds_new_group() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        let items = [1, 2];
        p.push(items);
        let items = [3, 4, 5];
        p.push(items);
        assert_eq!(p.starts_len.saturating_sub(1), 2);
        assert_eq!(&p[0], &[1, 2]);
        assert_eq!(&p[1], &[3, 4, 5]);
        assert_eq!(&p.inner[..p.inner_len], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_extend_appends_to_last_group() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        let items = [1, 2];
        p.push(items);
        let items = [3, 4];
        p.try_extend(items).expect("could not extend values");
        assert_eq!(p.starts_len.saturating_sub(1), 1);
        assert_eq!(&p[0], &[1, 2, 3, 4]);
        assert_eq!(&p.inner[..p.inner_len], &[1, 2, 3, 4]);
    }

    #[test]
    fn test_extend_on_empty_starts_new_group() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        let items = [5, 6];
        p.try_extend(items).expect("could not extend values");
        assert_eq!(p.starts_len.saturating_sub(1), 1);
        assert_eq!(&p[0], &[5, 6]);
        assert_eq!(&p.inner[..p.inner_len], &[5, 6]);
    }

    #[test]
    fn test_multiple_groups_iter() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        let items = [1];
        p.push(items);
        let items = [2, 3];
        p.push(items);
        let items = [4];
        p.push(items);
        assert_eq!(p.starts_len.saturating_sub(1), 3);
        assert_eq!(
            NestedIter::from_parts(
                &p.starts[..p.starts_len],
                &p.inner[..p.inner_len],
                0,
                p.starts_len.saturating_sub(1)
            )
            .collect::<Vec<_>>(),
            vec![&[1u8] as &[u8], &[2u8, 3], &[4u8]]
        );
    }

    #[test]
    fn test_get_out_of_bounds() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        p.push_one(1);
        assert!(p.get(0).is_some());
        assert!(p.get(1).is_none());
    }

    #[test]
    fn test_clear() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        let items = [1, 2, 3];
        p.push(items);
        p.inner_len = 0;
        p.starts_len = 0;
        assert!(p.starts_len == 0);
        assert_eq!(p.starts_len.saturating_sub(1), 0);
        assert_eq!(&p.inner[..p.inner_len], &[]);
    }

    #[test]
    fn test_first_last() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        assert!(p.get(0).is_none());
        assert!(
            p.get(p.starts_len.saturating_sub(1).saturating_sub(1))
                .is_none()
        );
        let items = [1];
        p.push(items);
        let items = [2];
        p.push(items);
        assert_eq!(p.get(0), Some(&[1u8] as &[u8]));
        assert_eq!(
            p.get(p.starts_len.saturating_sub(1).saturating_sub(1)),
            Some(&[2u8] as &[u8])
        );
    }

    #[test]
    fn test_as_params_roundtrip() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        p.push([10, 20]);
        p.push([30]);
        let ps = NestedSlice::from(&p);
        dbg!(&ps);
        assert_eq!(ps.len(), 2);
        assert_eq!(&ps[0], &[10, 20]);
        assert_eq!(&ps[1], &[30]);
    }

    #[test]
    fn test_index_mut() {
        let mut p: NestedRaw<u8, N, M> = NestedRaw::new();
        let items = [1, 2, 3];
        p.push(items);
        p[0][1] = 99;
        assert_eq!(&p[0], &[1, 99, 3]);
    }
}
