use std::ops;
use std::slice::SliceIndex;
use geometry::{Point, Position, PositionLike, Row};
use crate::{Buffer, Cell};

pub trait IntoSliceIndex<Context, Slice: ?Sized = Context>: Clone {
    type Output: ?Sized;
    type Index: SliceIndex<Slice, Output = Self::Output>;

    fn into_slice_index(self, context: &Context) -> Self::Index;
}

impl IntoSliceIndex<Buffer, [Cell]> for Point {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, grid: &Buffer) -> usize {
        self.y * grid.width + self.x
    }

}

impl IntoSliceIndex<Buffer, [Cell]> for Position {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, grid: &Buffer) -> usize {
        self.row * grid.width + self.col
    }

}

impl IntoSliceIndex<Buffer, [Cell]> for Row {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> ops::Range<usize> {
        self.value() * area.width..(self.value()) * area.width
    }
}
impl IntoSliceIndex<Buffer, [Cell]> for ops::Range<Row> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> ops::Range<usize> {
        self.start.value() * area.width..self.end.value() * area.width
    }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeTo<Row> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> ops::RangeTo<usize> {
        ..self.end.value() * area.width
    }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeFrom<Row> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> ops::RangeFrom<usize> {
        self.start.value() * area.width..
    }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> ops::RangeInclusive<usize> {
        self.start().value() * area.width..=self.end().value() * area.width
    }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeToInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> ops::RangeToInclusive<usize> {
        ..=self.end.value() * area.width
    }
}


impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeFull {
    type Output = [Cell];
    type Index = ops::RangeFull;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> ops::RangeFull { .. }
}


// Convenience for `Index` and `Position`
impl IntoSliceIndex<Buffer, [Cell]> for usize {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> usize { self }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::Range<usize> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> Self::Index { self }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeTo<usize> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> Self::Index { self }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeFrom<usize> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> Self::Index { self }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeInclusive<usize> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> Self::Index { self }
}

impl IntoSliceIndex<Buffer, [Cell]> for ops::RangeToInclusive<usize> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> Self::Index { self }
}

impl IntoSliceIndex<Buffer, [Cell]> for PositionLike {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, area: &Buffer) -> usize {
        self.0 * area.width + self.1
    }
}

