use std::ops;
use std::slice::{SliceIndex};
use crate::{Area, Position, Context, IntoLocation, Row, Span, PositionLike, Index, Column};

/// Resolves a spatial location into a linear `SliceIndex` via a `Context`.
pub trait IntoSliceIndex<T>: Sized {
    type Output: ?Sized;
    type Index: SliceIndex<[T], Output = Self::Output>;

    /// Resolves a spatial location into a linear `SliceIndex` via a `Context`.
    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> Self::Index;
}

impl<T> IntoSliceIndex<T> for Index {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> usize { self.value() }
}

impl<T> IntoSliceIndex<T> for ops::Range<Index> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self.start.value()..self.end.value() }
}

impl<T> IntoSliceIndex<T> for ops::RangeTo<Index> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { ..self.end.value() }
}

impl<T> IntoSliceIndex<T> for ops::RangeFrom<Index> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self.start.value().. }
}

impl<T> IntoSliceIndex<T> for ops::RangeInclusive<Index> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self.start().value()..=self.end().value() }
}

impl<T> IntoSliceIndex<T> for ops::RangeToInclusive<Index> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { ..=self.end.value() }
}

impl<T> IntoSliceIndex<T> for Position {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> usize {
        ctx.into_index(self)
    }
}

impl<T> IntoSliceIndex<T> for Row {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::Range<usize> {
        ctx.start(self)..ctx.end(self)
    }
}

impl<T> IntoSliceIndex<T> for Area {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::Range<usize> {
        ctx.start(self.min)..ctx.end(self.max)
    }
}

impl<T> IntoSliceIndex<T> for ops::Range<Row> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::Range<usize> {
        ctx.start(self.start)..ctx.end(self.end)
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeTo<Row> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::RangeTo<usize> {
        ..ctx.end(self.end)
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeFrom<Row> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::RangeFrom<usize> {
        ctx.start(self.start)..
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::RangeInclusive<usize> {
        ctx.start(*self.start())..=ctx.end(*self.end())
    }
}

impl<T> IntoSliceIndex<T> for ops::RangeToInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> ops::RangeToInclusive<usize> {
        ..=ctx.end(self.end)
    }
}


impl<T> IntoSliceIndex<T> for ops::RangeFull {
    type Output = [T];
    type Index = ops::RangeFull;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> ops::RangeFull { .. }
}


// Convenience for `Index` and `Position`

impl<T> IntoSliceIndex<T> for usize {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> usize { self }
}

impl<T> IntoSliceIndex<T> for ops::Range<usize> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeTo<usize> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeFrom<usize> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for ops::RangeToInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, _: &impl Context) -> Self::Index { self }
}

impl<T> IntoSliceIndex<T> for PositionLike {
    type Output = T;
    type Index = usize;

    #[inline]
    fn into_slice_index(self, ctx: &impl Context) -> usize {
        ctx.into_index(self)
    }
}
