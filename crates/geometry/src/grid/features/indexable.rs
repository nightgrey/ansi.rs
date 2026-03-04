use std::ops;
use std::slice::{SliceIndex};
use super::{SpatialIndex};
use crate::{Bounds, Position, Context, IntoLocation, Row, Span, PositionLike, Index, Column};

/// Resolves a spatial location into a linear `SliceIndex` via a `Context`.
pub trait Indexable<T>: Sized {
    type Output: ?Sized;
    type Index: SliceIndex<[T], Output = Self::Output>;

    /// Resolves a spatial location into a linear `SliceIndex` via a `Context`.
    #[inline]
    fn resolve(self, ctx: &impl Context) -> Self::Index;
}

impl<T> Indexable<T> for Position {
    type Output = T;
    type Index = usize;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> usize {
        ctx.into_index(self)
    }
}

impl<T> Indexable<T> for PositionLike {
    type Output = T;
    type Index = usize;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> usize {
        ctx.into_index(self)
    }
}

impl<T> Indexable<T> for Index {
    type Output = T;
    type Index = usize;

    #[inline]
    fn resolve(self, _: &impl Context) -> usize { self.value()}
}

impl<T> Indexable<T> for Row {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::Range<usize> {
        ctx.start(self)..ctx.end(self)
    }
}

impl<T> Indexable<T> for Bounds {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::Range<usize> {
        ctx.start(self.min)..ctx.end(self.max)
    }
}

impl<T> Indexable<T> for ops::Range<Row> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::Range<usize> {
        ctx.start(self.start)..ctx.end(self.end)
    }
}

impl<T> Indexable<T> for ops::RangeTo<Row> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::RangeTo<usize> {
        ..ctx.end(self.end)
    }
}

impl<T> Indexable<T> for ops::RangeFrom<Row> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::RangeFrom<usize> {
        ctx.start(self.start)..
    }
}

impl<T> Indexable<T> for ops::RangeInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::RangeInclusive<usize> {
        ctx.start(*self.start())..=ctx.end(*self.end())
    }
}

impl<T> Indexable<T> for ops::RangeToInclusive<Row> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn resolve(self, ctx: &impl Context) -> ops::RangeToInclusive<usize> {
        ..=ctx.end(self.end)
    }
}


impl<T> Indexable<T> for ops::RangeFull {
    type Output = [T];
    type Index = ops::RangeFull;

    #[inline]
    fn resolve(self, _: &impl Context) -> ops::RangeFull { .. }
}


impl<T> Indexable<T> for usize {
    type Output = T;
    type Index = usize;

    #[inline]
    fn resolve(self, _: &impl Context) -> Self::Index { self }
}

impl<T> Indexable<T> for ops::Range<usize> {
    type Output = [T];
    type Index = ops::Range<usize>;

    #[inline]
    fn resolve(self, _: &impl Context) -> Self::Index { self }
}

impl<T> Indexable<T> for ops::RangeTo<usize> {
    type Output = [T];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn resolve(self, _: &impl Context) -> Self::Index { self }
}

impl<T> Indexable<T> for ops::RangeFrom<usize> {
    type Output = [T];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn resolve(self, _: &impl Context) -> Self::Index { self }
}

impl<T> Indexable<T> for ops::RangeInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn resolve(self, _: &impl Context) -> Self::Index { self }
}

impl<T> Indexable<T> for ops::RangeToInclusive<usize> {
    type Output = [T];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn resolve(self, _: &impl Context) -> Self::Index { self }
}
