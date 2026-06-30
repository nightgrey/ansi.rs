use std::ops;
use geometry::{Point, PointLike, Row};
use crate::{Buffer, BufferIndex, Cell};

/// A trait returning an iterator over cells.
///
/// Unlike [`BufferIndexExt`][crate::BufferIndexExt], this only requires
/// [`BufferIndex`], producing cell iterators directly from
/// [`BufferIndex::get`] / [`BufferIndex::get_mut`] without the
/// [`BufferIndexMany`][crate::BufferIndexMany] machinery. Out-of-bounds
/// indices yield an empty iterator.
pub trait BufferIndexIter: BufferIndex {
    fn iter(self, context: &Buffer) -> impl Iterator<Item = &Cell> + ExactSizeIterator ;
    fn iter_mut(self, context: &mut Buffer) -> impl Iterator<Item = &mut Cell> + ExactSizeIterator;
}

/// Single-cell indices (`Output = Cell`): the option's own iterator yields the
/// cell 0 or 1 times with an exact `size_hint` and no slice setup.
macro_rules! impl_single {
    ($($ty:ty),* $(,)?) => {$(
        impl BufferIndexIter for $ty {
            #[inline]
            fn iter(self, context: &Buffer) -> impl Iterator<Item = &Cell> + ExactSizeIterator {
                self.get(context).into_iter()
            }

            #[inline]
            fn iter_mut(self, context: &mut Buffer) -> impl Iterator<Item = &mut Cell> + ExactSizeIterator {
                self.get_mut(context).into_iter()
            }
        }
    )*};
}

/// Slice indices (`Output = [Cell]`): fall back to an empty slice when out of
/// bounds, then iterate.
macro_rules! impl_slice {
    ($($ty:ty),* $(,)?) => {$(
        impl BufferIndexIter for $ty {
            #[inline]
            fn iter(self, context: &Buffer) -> impl Iterator<Item = &Cell> + ExactSizeIterator {
                self.get(context).unwrap_or(&[]).iter()
            }

            #[inline]
            fn iter_mut(self, context: &mut Buffer) -> impl Iterator<Item = &mut Cell> + ExactSizeIterator {
                self.get_mut(context).unwrap_or(&mut []).iter_mut()
            }
        }
    )*};
}

/// Slice indices generic over the bound `T` (the range types).
macro_rules! impl_slice_generic {
    ($($ty:ty),* $(,)?) => {$(
        impl<T: BufferIndex<SliceIndex = usize>> BufferIndexIter for $ty {
            #[inline]
            fn iter(self, context: &Buffer) -> impl Iterator<Item = &Cell> + ExactSizeIterator {
                self.get(context).unwrap_or(&[]).iter()
            }

            #[inline]
            fn iter_mut(self, context: &mut Buffer) -> impl Iterator<Item = &mut Cell> + ExactSizeIterator {
                self.get_mut(context).unwrap_or(&mut []).iter_mut()
            }
        }
    )*};
}

impl_single!(usize, Point, PointLike, PointLike<usize>);
impl_slice!(Row, ops::RangeFull);
impl_slice_generic!(
    ops::Range<T>,
    ops::RangeInclusive<T>,
    ops::RangeTo<T>,
    ops::RangeToInclusive<T>,
    ops::RangeFrom<T>,
);

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer() -> Buffer {
        // 3x2 grid of 'a'..'f'.
        Buffer::from_fn(3, 2, |row, col| Cell::new(char::from(b'a' + (row * 3 + col) as u8)))
    }

    #[test]
    fn single_index_yields_one_cell() {
        let buf = buffer();
        assert_eq!(BufferIndexIter::iter(0usize, &buf).count(), 1);
        assert_eq!(BufferIndexIter::iter(Point::new(1, 0), &buf).count(), 1);
    }

    #[test]
    fn row_yields_full_width() {
        let buf = buffer();
        assert_eq!(BufferIndexIter::iter(Row(1), &buf).count(), 3);
    }

    #[test]
    fn range_yields_slice() {
        let buf = buffer();
        assert_eq!(BufferIndexIter::iter(0usize..3usize, &buf).count(), 3);
    }

    #[test]
    fn out_of_bounds_is_empty() {
        let buf = buffer();
        assert_eq!(BufferIndexIter::iter(999usize, &buf).count(), 0);
        assert_eq!(BufferIndexIter::iter(0usize..999usize, &buf).count(), 0);
    }

    #[test]
    fn iter_mut_can_write() {
        let mut buf = buffer();
        for cell in BufferIndexIter::iter_mut(Row(0), &mut buf) {
            *cell = Cell::new('z');
        }
        assert!(BufferIndexIter::iter(Row(0), &buf).all(|c| *c == Cell::new('z')));
    }
}
