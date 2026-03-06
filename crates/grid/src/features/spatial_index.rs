use std::ops;
use std::slice::SliceIndex;
use crate::{Grid, Position, PositionLike, Row};
use geometry::{Point};

pub trait SpatialIndex<T>: Sized {
    type Output: ?Sized;
    type Index: SliceIndex<[T], Output = Self::Output>;

    #[inline]
    fn index_of(self, of: &Grid<T>) -> Self::Index;

    #[inline]
    fn get(self, slice: &Grid<T>) -> Option<&Self::Output> {
        SliceIndex::get(self.index_of(slice), slice.as_slice())
    }

    #[inline]
    fn get_mut(self, slice: &mut Grid<T>) -> Option<&mut Self::Output> {
        SliceIndex::get_mut(self.index_of(slice), slice.as_mut_slice())
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const Grid<T>) -> *const Self::Output {
        SliceIndex::get_unchecked(self.index_of(&*slice), (&*slice).as_slice())
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut Grid<T>) -> *mut Self::Output {
        SliceIndex::get_unchecked_mut(self.index_of(&*slice), (&mut *slice).as_mut_slice())
    }

    #[track_caller]
    #[inline]
    fn index(self, slice: &Grid<T>) -> &Self::Output {
        SliceIndex::index(self.index_of(slice), slice.as_slice())
    }

    #[track_caller]
    #[inline]
    fn index_mut(self, slice: &mut Grid<T>) -> &mut Self::Output {
        SliceIndex::index_mut(self.index_of(slice), slice.as_mut_slice())
    }
}

impl<T> SpatialIndex<T> for usize {
    type Output = T;
    type Index = usize;

    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for ops::Range<usize> {
    type Output = [T];
    type Index = ops::Range<usize>;
    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for ops::RangeTo<usize> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;
    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for ops::RangeFrom<usize> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;
    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for ops::RangeInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;
    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for ops::RangeToInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;
    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for ops::RangeFull {
    type Output = [T];
    type Index = ops::RangeFull;
    fn index_of(self, _: &Grid<T>) -> Self::Index { self }
}

impl<T> SpatialIndex<T> for Position {
    type Output = T;
    type Index = usize;
    fn index_of(self, of: &Grid<T>) -> usize { self.row * of.width + self.col }
}

impl<T> SpatialIndex<T> for PositionLike {
    type Output = T;
    type Index = usize;
    fn index_of(self, of: &Grid<T>) -> usize { self.0 * of.width + self.1 }
}

impl<T> SpatialIndex<T> for Point {
    type Output = T;
    type Index = usize;
    fn index_of(self, of: &Grid<T>) -> usize { self.y * of.width + self.x }
}

impl<T> SpatialIndex<T> for Row {
    type Output = [T];
    type Index = ops::Range<usize>;
    fn index_of(self, of: &Grid<T>) -> ops::Range<usize> {
        self.0 * of.width..(self.0 + 1) * of.width
    }
}

impl<T> SpatialIndex<T> for ops::Range<Row> {
    type Output = [T];
    type Index = ops::Range<usize>;
    fn index_of(self, of: &Grid<T>) -> ops::Range<usize> {
        self.start.value() * of.width..self.end.value() * of.width
    }
}

impl<T> SpatialIndex<T> for ops::RangeTo<Row> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;
    fn index_of(self, of: &Grid<T>) -> ops::RangeTo<usize> {
        ops::RangeTo { end: self.end.value() * of.width + of.width }
    }
}

impl<T> SpatialIndex<T> for ops::RangeFrom<Row> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;
    fn index_of(self, of: &Grid<T>) -> ops::RangeFrom<usize> {
        ops::RangeFrom { start: self.start.value() * of.width }
    }
}

impl<T> SpatialIndex<T> for ops::RangeInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;
    fn index_of(self, of: &Grid<T>) -> ops::RangeInclusive<usize> {
        ops::RangeInclusive::new(self.start().value() * of.width, self.end().value() * of.width + of.width)
    }
}

impl<T> SpatialIndex<T> for ops::RangeToInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;
    fn index_of(self, of: &Grid<T>) -> ops::RangeToInclusive<usize> {
        ops::RangeToInclusive { end: self.end.value() * of.width + of.width }
    }
}
