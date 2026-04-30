use core::ops::Index;
use std::iter::FusedIterator;
use std::ops::IndexMut;
use thiserror::Error;
use crate::NestedSlice;

/// Stack-allocated nested parameter buffer optimized for ANSI escape sequences.
/// `N` = max total values across all groups
/// `M` = max number of groups
#[derive(Clone, Copy, Debug)]
pub struct NestedRaw<T, const N: usize = 8, const M: usize = 8> {
    inner: [T; N],
    starts: [usize; M],
    inner_len: usize,
    starts_len: usize,
}

impl<T, const N: usize, const M: usize> NestedRaw<T, N, M> {
    #[inline]
    pub fn new() -> Self where T: Default + Copy {
        Self {
            inner: [Default::default(); N],
            starts: [0; M],
            inner_len: 0,
            starts_len: 0,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&[T]> {
        if index >= self.starts_len { return None; }
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
        if index >= self.starts_len { return None; }
        Some(self.get_unchecked_mut(index))
    }

    #[inline]
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut [T] {
        let start = self.starts[index];
        let end = self.starts[index + 1];
        &mut self.inner[start..end]
    }

    #[inline]
    pub fn try_push(&mut self, items: impl IntoIterator<Item = T>) -> Result<(), ParamsError> {
        if self.inner_len >= N || self.starts_len >= M { return Err(ParamsError::Overflow) }

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;

        for item in items {
            if self.inner_len >= N {
                self.starts[self.starts_len] = self.inner_len;
                return Err(ParamsError::Overflow);
            }
            self.inner[self.inner_len] = item;
            self.inner_len += 1;
        }

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;
        Ok(())
    }

    #[inline]
    pub fn try_extend(&mut self, items: impl IntoIterator<Item = T>) -> Result<(), ParamsError> {
        if self.starts_len >= M || self.inner_len >= N { return Err(ParamsError::Overflow) }

        if self.starts_len == 0 {
            return self.try_push(items);
        }
        for item in items {
            if self.inner_len >= N {
                self.starts[self.starts_len] = self.inner_len;
                return Err(ParamsError::Overflow);
            }
            self.inner[self.inner_len] = item;
            self.inner_len += 1;
        }

        self.starts[self.starts_len.saturating_sub(1)] = self.inner_len;
        Ok(())
    }

    #[inline]
    pub fn try_push_one(&mut self, val: T) -> Result<(), ParamsError> {
        if self.starts_len >= M || self.inner_len >= N { return Err(ParamsError::Overflow) }

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;

        self.inner[self.inner_len] = val;
        self.inner_len += 1;

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;
        Ok(())
    }

    #[inline]
    pub fn try_extend_one(&mut self, val: T) -> Result<(), ParamsError> {
        if self.starts_len >= M || self.inner_len >= N { return Err(ParamsError::Overflow) }

        if self.starts_len == 0 {
            return self.try_push_one(val);
        }

        self.inner[self.inner_len] = val;
        self.inner_len += 1;

        self.starts[self.starts_len.saturating_sub(1)] = self.inner_len;
        Ok(())
    }

    /// Starts a **new group** and appends multiple values.
    #[inline]
    pub fn push(&mut self, items: impl IntoIterator<Item = T>) {
        self.try_push(items).expect("could not push values");
    }

    /// Appends multiple values to the last group.
    #[inline]
    pub fn extend(&mut self, items: impl IntoIterator<Item = T>) {
        self.try_extend(items).expect("could not extend values");
    }

    /// Starts a **new group** and appends a single value to it.
    /// Extends the `starts` boundary array.
    #[inline]
    pub fn push_one(&mut self, val: T) {
        self.try_push_one(val).expect("Capacity exceeded");
    }

    /// Appends a single value to the **current (last) group**.
    /// Does NOT create a new boundary.
    #[inline]
    pub fn extend_one(&mut self, val: T) {
        self.try_extend_one(val).expect("could not extend value");
    }

    #[inline]
    pub fn len(&self) -> usize { self.starts_len.saturating_sub(1) }

    #[inline]
    pub fn is_empty(&self) -> bool { self.starts_len == 0 }

    #[inline]
    pub fn iter(&self) -> super::NestedIter<'_, T> {
        super::NestedIter::from_parts(&self.starts[..self.starts_len], &self.inner[..self.inner_len], 0, self.starts_len)
    }

    #[inline]
    pub fn iter_flat(&self) -> std::slice::Iter<'_, T> {
        self.inner[..self.inner_len].iter()
    }

    #[inline]
    pub fn first(&self) -> Option<&[T]> { self.get(0) }

    #[inline]
    pub fn last(&self) -> Option<&[T]> { self.get(self.len().saturating_sub(1)) }

    #[inline]
    pub fn clear(&mut self) {
        self.inner_len = 0;
        self.starts_len = 0;
    }

    #[inline]
    pub fn as_params(&self) -> NestedSlice<'_, T> {
        unsafe { NestedSlice::from_parts(&self.starts[..self.starts_len], &self.inner[..self.inner_len]) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.inner[..self.inner_len]
    }


}

impl<T, const N: usize, const M: usize> Default for NestedRaw<T, N, M>  where T: Default + Copy  {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize, const M: usize> Index<usize> for NestedRaw<T, N, M> {
    type Output = [T];
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get_unchecked(index)
    }
}

impl<T, const N: usize, const M: usize> IndexMut<usize> for NestedRaw<T, N, M> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_unchecked_mut(index)
    }
}

impl<T, const N: usize, const M: usize> Extend<T> for NestedRaw<T, N, M> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        Self::extend(self, iter);
    }
    fn extend_one(&mut self, item: T) {
        Self::extend_one(self, item);
    }
}

impl<T, const N: usize, const M: usize> AsRef<[T]> for NestedRaw<T, N, M> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

#[derive(Error, Debug)]
enum ParamsError {
    #[error("Params: Overflow")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extend_first() {
        let mut p = NestedRaw::<u8, 16, 8>::new();
        p.push_one(2);
        p.extend_one(1);

        assert_eq!(p.len(), 2);
        assert_eq!(&p[0], &[1]);
        assert_eq!(&p[1], &[2]);
        assert_eq!(p.as_slice(), &[1, 2]);
        assert_eq!(p.as_params().iter().collect::<Vec<_>>(), vec![&[1], &[2]]);
        assert_eq!(p.iter().collect::<Vec<_>>(), vec![&[1], &[2]]);
    }

    #[test]
    fn test_push_first() {
        let mut p = NestedRaw::<u8, 16, 8>::new();
        p.push_one(1);
        p.extend_one(2);

        assert_eq!(p.len(), 1);
        assert_eq!(&p[0], &[1, 2]);
        assert_eq!(p.as_slice(), &[1, 2]);
        assert_eq!(p.as_params().iter().collect::<Vec<_>>(), vec![&[1, 2]]);
        assert_eq!(p.iter().collect::<Vec<_>>(), vec![&[1, 2]]);
    }

}