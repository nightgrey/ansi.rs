use std::ops;
use std::slice::SliceIndex;
use geometry::{Point, Position, Row};
use super::{Buffer, Cell};

pub const trait Index: Sized {
    type Output: ?Sized;
    type Index: [const] SliceIndex<[Cell], Output = Self::Output>;

    #[inline]
    fn index_of(self, of: &Buffer) -> Self::Index;

    #[inline]
    fn get(self, slice: &Buffer) -> Option<&Self::Output> {
        SliceIndex::get(self.index_of(slice), slice.as_slice())
    }

    #[inline]
    fn get_mut(self, slice: &mut Buffer) -> Option<&mut Self::Output> {
        SliceIndex::get_mut(self.index_of(slice), slice.as_mut_slice())
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const Buffer) -> *const Self::Output {
        SliceIndex::get_unchecked(self.index_of(&*slice), (&*slice).as_slice())
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut Buffer) -> *mut Self::Output {
        SliceIndex::get_unchecked_mut(self.index_of(&*slice), (&mut *slice).as_mut_slice())
    }

    #[track_caller]
    #[inline]
    fn index(self, slice: &Buffer) -> &Self::Output {
        SliceIndex::index(self.index_of(slice), slice.as_slice())
    }

    #[track_caller]
    #[inline]
    fn index_mut(self, slice: &mut Buffer) -> &mut Self::Output {
        SliceIndex::index_mut(self.index_of(slice), slice.as_mut_slice())
    }
}

impl const Index for usize {
    type Output = Cell;
    type Index = usize;

    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for ops::Range<usize> {
    type Output = [Cell];
    type Index = ops::Range<usize>;
    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for ops::RangeTo<usize> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;
    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for ops::RangeFrom<usize> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;
    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for ops::RangeInclusive<usize> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;
    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for ops::RangeToInclusive<usize> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;
    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for ops::RangeFull {
    type Output = [Cell];
    type Index = ops::RangeFull;
    fn index_of(self, _: &Buffer) -> Self::Index { self }
}

impl const Index for Position {
    type Output = Cell;
    type Index = usize;
    fn index_of(self, of: &Buffer) -> usize { self.row * of.width + self.col }
}

impl const Index for Point {
    type Output = Cell;
    type Index = usize;
    fn index_of(self, of: &Buffer) -> usize { self.y * of.width + self.x }
}

impl const Index for Row {
    type Output = [Cell];
    type Index = ops::Range<usize>;
    fn index_of(self, of: &Buffer) -> ops::Range<usize> {
        self.0 * of.width..(self.0 + 1) * of.width
    }
}

impl const Index for ops::Range<Row> {
    type Output = [Cell];
    type Index = ops::Range<usize>;
    fn index_of(self, of: &Buffer) -> ops::Range<usize> {
        *self.start * of.width..*self.end * of.width
    }
}

impl const Index for ops::RangeTo<Row> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;
    fn index_of(self, of: &Buffer) -> ops::RangeTo<usize> {
        ops::RangeTo { end: *self.end * of.width + of.width }
    }
}

impl const Index for ops::RangeFrom<Row> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;
    fn index_of(self, of: &Buffer) -> ops::RangeFrom<usize> {
        ops::RangeFrom { start: *self.start * of.width }
    }
}

impl const Index for ops::RangeInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;
    fn index_of(self, of: &Buffer) -> ops::RangeInclusive<usize> {
        ops::RangeInclusive::new(**self.start() * of.width, **self.end() * of.width + of.width)
    }
}

impl const Index for ops::RangeToInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;
    fn index_of(self, of: &Buffer) -> ops::RangeToInclusive<usize> {
        ops::RangeToInclusive { end: *self.end * of.width + of.width }
    }
}

impl<Idx: [const] Index> const ops::Index<Idx> for Buffer {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        index.index(self)
    }
}

impl<Idx: [const] Index> const ops::IndexMut<Idx> for Buffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        index.index_mut(self)
    }
}

pub trait IntoIndex<T>: Sized {
    fn into_index(self, of: &Buffer) -> T;
}

impl<I: Index> IntoIndex<I::Index> for I {
    fn into_index(self, of: &Buffer) -> I::Index { self.index_of(of) }
}
