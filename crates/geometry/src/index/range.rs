use std::iter::FusedIterator;
use std::ops::{Range, Bound, RangeFrom, RangeTo, RangeInclusive, RangeToInclusive, RangeBounds, Bound::*};
use crate::Position;

/// An iterator over a 2D rectangular spatial range.
///
/// Iterates in row-major order (left-to-right, then top-to-bottom).
/// The range is half-open [start, end) in both dimensions.
#[derive(Clone, Debug)]
pub struct SpatialRangeIter {
    range: Range<Position>,
    current: Position,
}

impl SpatialRangeIter {
    /// Creates a new iterator from the given bounds.
    ///
    /// # Bounds interpretation
    /// - `Included(p)`: Use `p` directly as the bound
    /// - `Excluded(p)`: For start, moves to (p.row+1, p.col+1); for end, uses `p` directly
    /// - `Unbounded`: Uses `Position::MIN` for start, `Position::MAX` for end
    ///
    /// # Panics
    /// Panics in debug mode if the calculated start > end.
    pub fn new<R>(range: &R) -> Self
    where
        R: RangeBounds<Position>
    {
        let start = match range.start_bound() {
            Included(&p) => p,
            Excluded(&p) => Position {
                row: p.row.saturating_add(1),
                col: p.col.saturating_add(1)
            },
            Unbounded => Position::MIN,
        };

        let end = match range.end_bound() {
            Included(&p) => Position {
                row: p.row.saturating_add(1),
                col: p.col.saturating_add(1)
            },
            Excluded(&p) => p,
            Unbounded => Position::MAX,
        };

        debug_assert!(
            start.row <= end.row && start.col <= end.col,
            "Invalid spatial range: start {:?} > end {:?}", start, end
        );

        Self { current: start, range: start..end }
    }
}

impl Iterator for SpatialRangeIter {
    type Item = Position;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check if exhausted
        if self.is_empty() {
            return None;
        }

        let pos = self.current;

        // Advance cursor
        self.current.col += 1;
        if self.current.col >= self.range.end.col {
            self.current.col = self.range.start.col;
            self.current.row += 1;
        }

        Some(pos)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        Some(Position {
            row: self.range.end.row.saturating_sub(1),
            col: self.range.end.col.saturating_sub(1),
        })
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // Optimized nth: jump rows if possible
        if self.is_empty() {
            return None;
        }

        let width = self.range.end.col.saturating_sub(self.range.start.col) as usize;
        if width == 0 {
            return None;
        }

        let current_offset = (self.current.row - self.range.start.row) as usize * width
            + (self.current.col - self.range.start.col) as usize;
        let target_offset = current_offset + n;

        let target_row = target_offset / width;
        let target_col = target_offset % width;

        if target_row >= self.range.end.row.saturating_sub(self.range.start.row) as usize {
            self.current = self.range.end; // Mark as exhausted
            return None;
        }

        self.current = Position {
            row: self.range.start.row + target_row,
            col: self.range.start.col + target_col,
        };

        self.next()
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.last()
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl ExactSizeIterator for SpatialRangeIter {
    #[inline]
    fn len(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let width = self.range.end.col.saturating_sub(self.range.start.col) as usize;
        let remaining_rows = self.range.end.row.saturating_sub(self.current.row) as usize;
        let consumed_in_first_row = self.current.col.saturating_sub(self.range.start.col) as usize;

        remaining_rows.saturating_mul(width).saturating_sub(consumed_in_first_row)
    }

    /// Returns true if the range is empty (contains no positions).
    fn is_empty(&self) -> bool {
        self.current.row >= self.range.end.row
            || self.range.start.col >= self.range.end.col
    }
}

impl FusedIterator for SpatialRangeIter {}

/// Extension trait for types that can be iterated as spatial ranges.
pub trait SpatialRange: Sized + RangeBounds<Position> {
    /// Returns an iterator over the spatial range.
    fn iter(&self) -> SpatialRangeIter {
        SpatialRangeIter::new(self)
    }

    /// Returns true if the range contains no positions.
    fn is_empty(&self) -> bool {
        SpatialRangeIter::new(self).next().is_none()
    }
}

impl SpatialRange for Range<Position> {}
impl SpatialRange for RangeFrom<Position> {}
impl SpatialRange for RangeTo<Position> {}
impl SpatialRange for RangeInclusive<Position> {}
impl SpatialRange for RangeToInclusive<Position> {}
impl SpatialRange for (Bound<Position>, Bound<Position>) {}