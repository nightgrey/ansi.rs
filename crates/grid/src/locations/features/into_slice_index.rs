use std::ops;
use std::slice::{SliceIndex};
use crate::{Area, Position, Spatial, IntoLocation, Row, Range, PositionLike, Index, Column};

/// Resolves a spatial location into a linear `SliceIndex` via a spatial area.
pub trait IntoSliceIndex<T>: Sized {
    type Output: ?Sized;
    type Index: SliceIndex<[T], Output = Self::Output>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> Self::Index;
}

impl<T> IntoSliceIndex<T> for Index {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> usize { self.value() }
}

impl<T> IntoSliceIndex<T> for ops::Range<Index> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self.start.value()..self.end.value() }
}

impl<T> IntoSliceIndex<T> for ops::RangeTo<Index> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { ..self.end.value() }
}

impl<T> IntoSliceIndex<T> for ops::RangeFrom<Index> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self.start.value().. }
}

impl<T> IntoSliceIndex<T> for ops::RangeInclusive<Index> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self.start().value()..=self.end().value() }
}

impl<T> IntoSliceIndex<T> for ops::RangeToInclusive<Index> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { ..=self.end.value() }
}

impl<T> IntoSliceIndex<T> for Position {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> usize {
        area.into_index(self)
    }
}

impl<T> IntoSliceIndex<T> for Row {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::Range<usize> {
        area.start(self)..area.end(self)
    }
}

impl<T> IntoSliceIndex<T> for Area {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::Range<usize> {
        area.start(self.min)..area.end(self.max)
    }
}

impl<T> IntoSliceIndex<T> for ops::Range<Row> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::Range<usize> {
        area.start(self.start)..area.end(self.end)
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeTo<Row> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::RangeTo<usize> {
        ..area.end(self.end)
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeFrom<Row> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::RangeFrom<usize> {
        area.start(self.start)..
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::RangeInclusive<usize> {
        area.start(*self.start())..=area.end(*self.end())
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeToInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> ops::RangeToInclusive<usize> {
        ..=area.end(self.end)
    }
}


impl<T> IntoSliceIndex<T> for ops::RangeFull {
    type Output = [T];
    type Index = ops::RangeFull;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> ops::RangeFull { .. }
}


// Convenience for `Index` and `Position`

impl<T> IntoSliceIndex<T> for usize {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> usize { self }
}

impl<T> IntoSliceIndex<T> for ops::Range<usize> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeTo<usize> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeFrom<usize> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeToInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Spatial) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for PositionLike {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, area: &impl Spatial) -> usize {
        area.into_index(self)
    }
}
