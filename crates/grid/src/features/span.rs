use std::ops;
use std::ops::Bound;
use crate::{Column, Position, Row, Location, Bounds, IntoLocation, Context, Index};

/// Maps a location to its linear index range within a context.
///
/// Unlike `std::ops::RangeBounds` (which describes range endpoints),
/// `Span` answers: "what contiguous slice of indices does this location
/// occupy within this context?"
pub const trait Span<T = Position> {
    #[inline]
    fn start(&self, location: T) -> usize;

    #[inline]
    fn end(&self, location: T) -> usize;

    #[inline]
    fn range(&self, location: T) -> ops::Range<usize> where T: Copy {
        self.start(location)..self.end(location)
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
    fn into_bounds(&self, location: T) -> (Bound<usize>, Bound<usize>) where T: Copy {
        (Bound::Included(self.start(location)), Bound::Excluded(self.end(location)))
    }
}

impl<T: [const] Context> const Span<Bounds> for T {
    fn start(&self, location: Bounds) -> usize {
        self.into_index(location.min)
    }

    fn end(&self, location: Bounds) -> usize {
        self.into_index(location.max)
    }
}

impl<T: [const] Context> const Span<Row> for T {
    fn start(&self, location: Row) -> usize {
        self.into_index(location)
    }

    fn end(&self, location: Row) -> usize {
        self.into_index(location) + self.width()
    }
}

impl<T: [const] Context> const Span<Position> for T {
    fn start(&self, location: Position) -> usize {
        self.into_index(location)
    }

    fn end(&self, location: Position) -> usize {
        self.into_index(location) + 1
    }
}

impl<T: [const] Context> const Span<Column> for T {
    fn start(&self, location: Column) -> usize {
        location.value()
    }

    fn end(&self, location: Column) -> usize {
        // A column spans `height` non-contiguous cells; as a contiguous
        // span this doesn't fully make sense, so we return the index
        // one-past the last row's cell in this column.
        self.into_index(Position::new(self.max().row.saturating_sub(1), location.value())) + 1
    }
}

impl<T: [const] Context> const Span<Index> for T {
    fn start(&self, location: Index) -> usize {
        location.value()
    }

    fn end(&self, location: Index) -> usize {
        location.value() + 1
    }
}
