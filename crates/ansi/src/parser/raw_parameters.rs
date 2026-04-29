use core::ops::Index;
use std::iter::FusedIterator;
use std::ops::IndexMut;
use thiserror::Error;
use utils::NestedSlice;
use crate::parser::Params;

/// Stack-allocated nested parameter buffer optimized for ANSI escape sequences.
/// `N` = max total values across all groups
/// `M` = max number of groups
#[derive(Clone, Copy, Debug)]
pub struct RawParameters<const N: usize = 64, const M: usize = 16> {
    inner: [u16; N],
    starts: [usize; M],
    inner_len: usize,
    starts_len: usize,
}

impl<const N: usize, const M: usize> RawParameters<N, M> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&[u16]> {
        if index >= self.starts_len { return None; }
        Some(self.get_unchecked(index))
    }

    #[inline]
    pub fn get_unchecked(&self, index: usize) -> &[u16] {
        let start = self.starts[index];
        let end = self.starts[index + 1];
        &self.inner[start..end]
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut [u16]> {
        if index >= self.starts_len { return None; }
        Some(self.get_unchecked_mut(index))
    }

    #[inline]
    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut [u16] {
        let start = self.starts[index];
        let end = self.starts[index + 1];
        &mut self.inner[start..end]
    }

    #[inline]
    pub fn try_push(&mut self, items: impl IntoIterator<Item = u16>) -> Result<(), ParamsError> {
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
        Ok(())
    }

    #[inline]
    pub fn try_push_one(&mut self, val: u16) -> Result<(), ParamsError> {
        if self.starts_len >= M || self.inner_len >= N { return Err(ParamsError::Overflow) }

        self.starts[self.starts_len] = self.inner_len;
        self.starts_len += 1;

        self.inner[self.inner_len] = val;
        self.inner_len += 1;

        self.starts[self.starts_len] = self.inner_len;
        Ok(())
    }

    #[inline]
    pub fn try_extend(&mut self, items: impl IntoIterator<Item = u16>) -> Result<(), ParamsError> {
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

            self.starts[self.starts_len] = self.inner_len;

        Ok(())
    }

    #[inline]
    pub fn try_extend_one(&mut self, val: u16) -> Result<(), ParamsError> {
        if self.starts_len >= M || self.inner_len >= N { return Err(ParamsError::Overflow) }

        if self.starts_len == 0 {
            return self.try_push_one(val);
        }

        self.inner[self.inner_len] = val;
        self.inner_len += 1;

        self.starts[self.starts_len] = self.inner_len;
        Ok(())
    }

    /// Starts a **new group** and appends multiple values.
    #[inline]
    pub fn push(&mut self, items: impl IntoIterator<Item = u16>) {
        self.try_push(items).expect("could not push values");
    }

    /// Appends multiple values to the last group.
    #[inline]
    pub fn extend(&mut self, items: impl IntoIterator<Item = u16>) {
     self.try_extend(items).expect("could not extend values");
    }

    /// Starts a **new group** and appends a single value to it.
    /// Extends the `starts` boundary array.
    #[inline]
    pub fn push_one(&mut self, val: u16) {
        self.try_push_one(val).expect("Capacity exceeded");
    }

    /// Appends a single value to the **current (last) group**.
    /// Does NOT create a new boundary.
    #[inline]
    pub fn extend_one(&mut self, val: u16) {
        self.try_extend_one(val).expect("could not extend value");
    }
    #[inline]
    pub fn len(&self) -> usize { self.starts_len }

    #[inline]
    pub fn is_empty(&self) -> bool { self.starts_len == 0 }

    #[inline]
    pub fn iter(&self) -> utils::NestedIter<'_, u16> {
        utils::NestedIter::from_parts(&self.starts[..=self.starts_len], &self.inner[..self.inner_len])
    }

    #[inline]
    pub fn values(&self) -> std::slice::Iter<'_, u16> {
        self.inner[..self.inner_len].iter()
    }

    #[inline]
    pub fn first(&self) -> Option<&[u16]> { self.get(0) }

    #[inline]
    pub fn last(&self) -> Option<&[u16]> { self.get(self.len().saturating_sub(1)) }

    #[inline]
    pub fn clear(&mut self) {
        self.inner_len = 0;
        self.starts_len = 0;
    }

    #[inline]
    pub fn as_params(&self) -> Params<'_> {
        unsafe { Params::from_parts(&self.starts[..=self.starts_len], &self.inner[..self.inner_len]) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u16] {
        &self.inner[..self.inner_len]
    }
}

impl<const N: usize, const M: usize> Default for RawParameters<N, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: [0; N],
            starts: [0; M],
            inner_len: 0,
            starts_len: 0,
        }
    }
}

impl<const N: usize, const M: usize> Index<usize> for RawParameters<N, M> {
    type Output = [u16];
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get_unchecked(index)
    }
}

impl<const N: usize, const M: usize> IndexMut<usize> for RawParameters<N, M> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_unchecked_mut(index)
    }
}

impl<const N: usize, const M: usize> Extend<u16> for RawParameters<N, M> {
    fn extend<T: IntoIterator<Item = u16>>(&mut self, iter: T) {
        Self::extend(self, iter);
    }
    fn extend_one(&mut self, item: u16) {
        Self::extend_one(self, item);
    }
}

impl<const N: usize, const M: usize> AsRef<[u16]> for RawParameters<N, M> {
    fn as_ref(&self) -> &[u16] {
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
    fn t() {
        let mut p = RawParameters::<16, 8>::new();
        p.extend_one(1);
        p.extend_one(2);

        p.extend_one(3);

        p.extend_one(4);

        p.extend_one(5);

        dbg!(p.get(0));
        dbg!(p.iter().collect::<Vec<_>>());
    }
}