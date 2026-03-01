use std::ops;
use std::ops::Bound;
use crate::{Column, IntoLocation, Position, Row, Location, Bounds};

pub const trait RangeBounds<T = Position> {
    #[inline]
    fn start(&self, location: T) -> usize;

    #[inline]
    fn end(&self, location: T) -> usize;

    #[inline]
    fn into_range(&self, location: T) -> ops::Range<usize> where T: [const] Clone {
        let start = self.start(location.clone());
        let end = self.end(location);
        start..end
    }

    #[inline]
    fn start_bound(&self, location: T) -> Bound<usize> {
        Bound::Included(self.start(location))
    }

    #[inline]
    fn end_bound(&self, location: T) -> Bound<usize> {
        Bound::Excluded(self.end(location))
    }

    #[inline]
    fn into_bounds(&self, location: T) -> (Bound<usize>, Bound<usize>) where T: [const] Clone {
        let start = self.start(location.clone());
        let end = self.end(location);
        (Bound::Included(start), Bound::Excluded(end))
    }
}

impl RangeBounds<Row> for Bounds {
    fn start(&self, location: Row) -> usize {
        self.into_index(location)
    }

    fn end(&self, location: Row) -> usize {
        self.into_index(location) + self.width()
    }

    fn start_bound(&self, location: Row) -> Bound<usize> {
        Bound::Included(self.into_index(location))
    }
    fn end_bound(&self, location: Row) -> Bound<usize> {
        Bound::Excluded(self.into_index((self.into_index(location) + self.width())))
    }

}

impl RangeBounds<Position> for Bounds {
    fn start(&self, location: Position) -> usize {
        self.into_index(location)
    }

    fn end(&self, location: Position) -> usize {
        self.into_index(location) + 1
    }

    fn start_bound(&self, location: Position) -> Bound<usize> {
        Bound::Included(self.into_index(location))
    }

    fn end_bound(&self, location: Position) -> Bound<usize> {
        Bound::Excluded(self.into_index(location) + 1)
    }

}

impl RangeBounds<Column> for Bounds {
    fn start(&self, location: Column) -> usize {
        self.into_index(location)
    }

    fn end(&self, location: Column) -> usize {
        self.into_index(location) * self.height()
    }

    fn start_bound(&self, location: Column) -> Bound<usize> {
        Bound::Included(self.into_index(location))
    }

    fn end_bound(&self, location: Column) -> Bound<usize> {
        Bound::Excluded(self.into_index(location.value() + self.height()))
    }

}

impl RangeBounds<usize> for Bounds {
    fn start(&self, location: usize) -> usize {
        self.into_index(location)
    }

    fn end(&self, location: usize) -> usize {
        self.into_index(location) + 1
    }

    fn start_bound(&self, location: usize) -> Bound<usize> {
        Bound::Included(self.into_index(location))
    }

    fn end_bound(&self, location: usize) -> Bound<usize> {
        Bound::Excluded(self.into_index(location + 1))
    }

}

pub const trait RangeBoundsWithin: Sized + Location {
    #[inline]
    fn start(self, ctx: &impl [const] RangeBounds<Self>) -> usize {
        ctx.start(self)
    }

    #[inline]
    fn end(self, ctx: &impl [const] RangeBounds<Self>) -> usize {
        ctx.end(self)
    }

    #[inline]
    fn into_range(self, ctx: &impl [const] RangeBounds<Self>) -> ops::Range<usize> where Self: [const] Clone {
        ctx.into_range(self)
    }

    #[inline]
    fn start_bound(self, ctx: &impl [const] RangeBounds<Self>) -> Bound<usize> {
        ctx.start_bound(self)
    }

    #[inline]
    fn end_bound(self, ctx: &impl [const] RangeBounds<Self>) -> Bound<usize> {
        ctx.end_bound(self)
    }

    #[inline]
    fn into_bounds(self, ctx: &impl [const] RangeBounds<Self>) -> (Bound<usize>, Bound<usize>) where Self: [const] Clone {
        ctx.into_bounds(self)
    }
}
