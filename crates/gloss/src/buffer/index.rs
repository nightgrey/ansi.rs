use geometry::{Point, PointLike, Row};
use std::ops;
use std::ops::Index;
use std::slice::SliceIndex;
use crate::{Buffer, Cell};

pub trait BufferIndex<Context, Slice: ?Sized = Context>: Clone {
    type Output: ?Sized;
    type Index: SliceIndex<Slice, Output = Self::Output>;

    fn index_of(self, context: &Context) -> Self::Index;
}

impl BufferIndex<Buffer, [Cell]> for Point {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn index_of(self, grid: &Buffer) -> usize {
        self.y * grid.width + self.x
    }
}


impl BufferIndex<Buffer, [Cell]> for Row {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::Range<usize> {
        let start = self.value() * area.width;
        start..start + area.width
    }
}
impl BufferIndex<Buffer, [Cell]> for ops::Range<Row> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::Range<usize> {
        self.start.value() * area.width..self.end.value() * area.width + area.width
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeTo<Row> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeTo<usize> {
        ..self.end.value() * area.width + area.width

    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeFrom<Row> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeFrom<usize> {
        self.start.value() * area.width..

    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeInclusive<usize> {
        self.start().value() * area.width..=self.end().value() * area.width + area.width
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeToInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeToInclusive<usize> {
        ..=self.end.value() * area.width + area.width
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeFull {
    type Output = [Cell];
    type Index = ops::RangeFull;

    #[inline]
    fn index_of(self, _: &Buffer) -> ops::RangeFull {
        ..
    }
}

// Convenience for `Index` and `Position`
impl BufferIndex<Buffer, [Cell]> for usize {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn index_of(self, _: &Buffer) -> usize {
        self
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::Range<usize> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn index_of(self, _: &Buffer) -> Self::Index {
        self
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeTo<usize> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn index_of(self, _: &Buffer) -> Self::Index {
        self
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeFrom<usize> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn index_of(self, _: &Buffer) -> Self::Index {
        self
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeInclusive<usize> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn index_of(self, _: &Buffer) -> Self::Index {
        self
    }
}

impl BufferIndex<Buffer, [Cell]> for ops::RangeToInclusive<usize> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn index_of(self, _: &Buffer) -> Self::Index {
        self
    }
}
impl BufferIndex<Buffer, [Cell]> for PointLike {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn index_of(self, area: &Buffer) -> usize {
        self.1 * area.width + self.0
    }
}