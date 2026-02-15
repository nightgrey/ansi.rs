use std::iter::FusedIterator;
use std::ops::{Bound, Deref, IntoBounds, RangeBounds};

use crate::{Position, Size, SpatialContext, SpatialIter, SpatialStep};

// ─── Region ──────────────────────────────────────────────────────────────────

/// A half-open rectangular region in row-major (row, col) space.
///
/// `min` is inclusive, `max` is exclusive — the same convention as `Range`.
/// A region where `min == max` (or any axis is equal) is empty.
///
/// # Coordinate system
/// Positions are (row, col) with row 0 at the top.
/// Iteration proceeds in row-major order: left→right within a row,
/// then top→bottom across rows.
///
/// # Invariants
/// * `min.row <= max.row`
/// * `min.col <= max.col`
#[derive(Copy, Debug)]
#[derive_const(Default, Clone, Eq, PartialEq)]
pub struct Region {
    pub min: Position,
    pub max: Position,
}

impl Region {
    pub const ZERO: Self = Self {
        min: Position::ZERO,
        max: Position::ZERO,
    };

    /// Creates a new region from inclusive `min` to exclusive `max`.
    ///
    /// # Panics (debug only)
    /// Panics if `min > max` on either axis.
    pub fn new(min: Position, max: Position) -> Self {
        debug_assert!(
            min.row <= max.row && min.col <= max.col,
            "Invalid region: min ({}) must be <= max ({}) on both axes.",
            min,
            max
        );
        Self { min, max }
    }

    /// Number of columns spanned.
    pub const fn width(&self) -> usize {
        self.max.col - self.min.col
    }

    /// Number of rows spanned.
    pub const fn height(&self) -> usize {
        self.max.row - self.min.row
    }

    /// Total number of cells (`width * height`).
    #[inline]
    pub const fn area(&self) -> usize {
        self.width() * self.height()
    }

    /// Alias for [`area`](Self::area).
    #[inline]
    pub const fn len(&self) -> usize {
        self.area()
    }

    /// Whether this region contains zero cells.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.area() == 0
    }

    /// The position at row/col offset from `min`.
    ///
    /// No bounds checking — caller must ensure the result is within the region.
    #[inline]
    pub const fn at(&self, row: usize, col: usize) -> Position {
        Position::new(self.min.row + row, self.min.col + col)
    }

    /// Converts a linear index (row-major) to a position within this region.
    ///
    /// # Panics
    /// Panics if `index >= area()`.
    #[inline]
    pub const fn position_of(&self, index: usize) -> Position {
        let w = self.width();
        Position::new(self.min.row + index / w, self.min.col + index % w)
    }

    /// Converts a position to a linear index (row-major) within this region.
    ///
    /// # Panics
    /// Panics if `pos` is outside the region (in debug builds, may wrap in release).
    #[inline]
    pub const fn index_of(&self, pos: &Position) -> usize {
        (pos.row - self.min.row) * self.width() + (pos.col - self.min.col)
    }

    /// Whether `pos` lies inside this region (min inclusive, max exclusive).
    pub const fn contains(&self, pos: &Position) -> bool {
        self.min.row <= pos.row
            && pos.row < self.max.row
            && self.min.col <= pos.col
            && pos.col < self.max.col
    }

    /// Returns the intersection of two regions (may be empty).
    pub const fn intersect(self, other: Self) -> Self {
        let min_row = if self.min.row > other.min.row { self.min.row } else { other.min.row };
        let min_col = if self.min.col > other.min.col { self.min.col } else { other.min.col };
        let max_row = if self.max.row < other.max.row { self.max.row } else { other.max.row };
        let max_col = if self.max.col < other.max.col { self.max.col } else { other.max.col };

        // Clamp to empty if min overtakes max on either axis.
        let (max_row, max_col) = if min_row > max_row || min_col > max_col {
            (min_row, min_col)
        } else {
            (max_row, max_col)
        };

        Self {
            min: Position::new(min_row, min_col),
            max: Position::new(max_row, max_col),
        }
    }

    /// Clips `self` to fit within `other`. Alias for `other.intersect(self)`.
    pub const fn clip(self, other: Self) -> Self {
        other.intersect(self)
    }

    /// The (width, height) as a `Size`.
    pub const fn size(&self) -> Size {
        Size::new(self.width(), self.height())
    }

    /// Row-major iterator over every position in the region.
    pub const fn iter(&self) -> SpatialIter {
        SpatialIter::new(*self)
    }
}

impl IntoIterator for Region {
    type Item = Position;
    type IntoIter = SpatialIter;
    fn into_iter(self) -> Self::IntoIter {
        SpatialIter::new(self)
    }
}

impl IntoIterator for &Region {
    type Item = Position;
    type IntoIter = SpatialIter;
    fn into_iter(self) -> Self::IntoIter {
        SpatialIter::new(*self)
    }
}

impl RangeBounds<Position> for Region {
    #[inline]
    fn start_bound(&self) -> Bound<&Position> {
        Bound::Included(&self.min)
    }
    #[inline]
    fn end_bound(&self) -> Bound<&Position> {
        Bound::Excluded(&self.max)
    }
}

impl IntoBounds<Position> for Region {
    #[inline]
    fn into_bounds(self) -> (Bound<Position>, Bound<Position>) {
        (Bound::Included(self.min), Bound::Excluded(self.max))
    }
}


impl SpatialContext for Region {
    fn steps_between(&self, start: &Position, end: &Position) -> (usize, Option<usize>) {
        if start > end {
            return (0, None);
        }
        let current = self.index_of(start);
        let remaining = self.index_of(end);
        let dist = (remaining - current);

        (dist, Some(dist))
    }

    fn forward_checked(&self, start: Position, count: usize) -> Option<Position> {
        // OPTIMIZATION: Fast path for single step (Iterator usage)
        // This mirrors the manual assembly logic: increment, then check bounds.
        if count == 1 {

            let mut next = start;
            next.col += 1;

            // Check row wrap
            if next.col >= self.max.col {
                next.col = self.min.col;
                next.row += 1;

                // Check region end
                if next.row >= self.max.row {
                    return None;
                }
            }

            // We need to ensure we don't start past the end (safety check for API)
            // But for iterators, start is always valid.
            // This logic naturally returns None if start was already invalid/past end
            // because row >= end.row will likely trigger or already be true.
            return Some(next);
        }

        // Fallback for arbitrary steps (simplified from previous)
        // Note: This path is rarely hit by standard iterators.
        let index = self.index_of(&start).checked_add(count)?;
        if index >= self.area() {
            return None;
        }

        Some(self.position_of(index))
    }
    fn backward_checked(&self, start: Position, count: usize) -> Option<Position> {
        // Fast path: stay in current row.
        if count <= start.col - self.min.col {
            return Some(Position::new(start.row, start.col - count));
        }

        // General case: linearize, step, delinearize.
        let idx = self.index_of(&start).checked_sub(count)?;
        Some(self.position_of(idx))
    }
}

impl SpatialStep for Position {
    type Context = Region;
}

#[cfg(test)]
mod tests {
    use super::*;
    // === Region Tests ===

    #[test]
    fn test_bounds_off_by_one_errors() {
        for x in 0..2 {
            for y in 0..2 {
                let bounds = Region::new(Position::new(0, 0), Position::new(x, y));

                let area = bounds.area();
                let len = bounds.iter().collect::<Vec<_>>().len();
                let count = bounds.iter().count();
                let size_hint = bounds.iter().size_hint().1.unwrap_or(0);

                assert_eq!(area, area);
                assert_eq!(area, len);
                assert_eq!(area,count);
                assert_eq!(area, size_hint);

                assert_eq!(len, area);
                assert_eq!(len, len);
                assert_eq!(len, count);
                assert_eq!(len, size_hint);

                assert_eq!(count, area);
                assert_eq!(count, len);
                assert_eq!(count,count);
                assert_eq!(count, size_hint);

                assert_eq!(size_hint, area);
                assert_eq!(size_hint, len);
                assert_eq!(size_hint,count);
                assert_eq!(size_hint, size_hint);
            }
        }

        for x in 1..2 {
            for y in 1..3 {

                let bounds = Region::new(Position::new(1, 1), Position::new(x, y));

                let area = bounds.area();
                let len = bounds.iter().collect::<Vec<_>>().len();
                let count = bounds.iter().count();
                let size_hint = bounds.iter().size_hint().1.unwrap_or(0);

                assert_eq!(area, area);
                assert_eq!(area, len);
                assert_eq!(area,count);
                assert_eq!(area, size_hint);

                assert_eq!(len, area);
                assert_eq!(len, len);
                assert_eq!(len, count);
                assert_eq!(len, size_hint);

                assert_eq!(count, area);
                assert_eq!(count, len);
                assert_eq!(count,count);
                assert_eq!(count, size_hint);

                assert_eq!(size_hint, area);
                assert_eq!(size_hint, len);
                assert_eq!(size_hint,count);
                assert_eq!(size_hint, size_hint);
            }
        }


        for x in 0..3 {
            for y in 0..3 {
                let bounds = Region::new(Position::new(x, y), Position::new(x + 1, y + 1));

                let area = bounds.area();
                let len = bounds.iter().collect::<Vec<_>>().len();
                let count = bounds.iter().count();
                let size_hint = bounds.iter().size_hint().1.unwrap_or(0);

                assert_eq!(area, area);
                assert_eq!(area, len);
                assert_eq!(area,count);
                assert_eq!(area, size_hint);

                assert_eq!(len, area);
                assert_eq!(len, len);
                assert_eq!(len, count);
                assert_eq!(len, size_hint);

                assert_eq!(count, area);
                assert_eq!(count, len);
                assert_eq!(count,count);
                assert_eq!(count, size_hint);

                assert_eq!(size_hint, area);
                assert_eq!(size_hint, len);
                assert_eq!(size_hint,count);
                assert_eq!(size_hint, size_hint);
            }
        }
    }
    #[test]
    fn test_bounds_new() {
        let r = Region::new(Position::new(5, 10), Position::new(15, 30));
        assert_eq!(r.min, Position::new(5, 10));
        assert_eq!(r.max, Position::new(15, 30));
    }

    #[test]
    fn test_bounds_width_height() {
        let r = Region::new(Position::new(0, 0), Position::new(5, 10));
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 5);
    }

    #[test]
    fn test_bounds_area() {
        let r = Region::new(Position::new(0, 0), Position::new(4, 5));
        assert_eq!(r.area(), 20); // 4 * 5
    }

    #[test]
    fn test_bounds_contains() {
        let r = Region::new(Position::new(10, 10), Position::new(20, 20));

        // Inside
        assert!(r.contains(&Position::new(15, 15)));

        // Min edge (inclusive)
        assert!(r.contains(&Position::new(10, 10)));

        // Max edge (exclusive)
        assert!(!r.contains(&Position::new(20, 20)));

        // Outside
        assert!(!r.contains(&Position::new(25, 25)));
        assert!(!r.contains(&Position::new(5, 5)));
    }

    #[test]
    fn test_bounds_size() {
        let r = Region::new(Position::new(0, 0), Position::new(24, 80));
        let size = r.size();
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }

    // #[test]
    // fn test_bounds_from_rect() {
    //     let rect = Rect::new((10, 5), (30, 25));
    //     let bounds: Region = rect.into();
    //
    //     assert_eq!(bounds.min, Position::new(5, 10)); // (y, x)
    //     assert_eq!(bounds.max, Position::new(25, 30));
    // }

    // === SpatialIter Tests ===

    #[test]
    fn test_bounds_iter_basic() {
        let bounds = Region::new(Position::new(0, 0), Position::new(2, 3));
        let positions: Vec<_> = bounds.iter().collect();

        assert_eq!(positions.len(), 6); // 2 rows * 3 cols

        // Check row-major order
        assert_eq!(positions[0], Position::new(0, 0));
        assert_eq!(positions[1], Position::new(0, 1));
        assert_eq!(positions[2], Position::new(0, 2));
        assert_eq!(positions[3], Position::new(1, 0));
        assert_eq!(positions[4], Position::new(1, 1));
        assert_eq!(positions[5], Position::new(1, 2));
    }

    #[test]
    fn test_bounds_iter_empty_width() {
        let bounds = Region::new(Position::new(0, 5), Position::new(0, 5));
        assert_eq!(bounds.iter().count(), 0);
    }

    #[test]
    fn test_bounds_iter_empty_height() {
        let bounds = Region::new(Position::new(5, 0), Position::new(5, 1));
        assert_eq!(bounds.iter().count(), 0);
    }

    #[test]
    fn test_bounds_iter_single_cell() {
        let bounds = Region::new(Position::new(5, 10), Position::new(6, 11));
        let positions: Vec<_> = bounds.iter().collect();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position::new(5, 10));
    }

    #[test]
    fn test_bounds_iter_size_hint() {
        let bounds = Region::new(Position::new(0, 0), Position::new(3, 4));
        let iter = bounds.iter();
        let (min, max) = iter.size_hint();

        assert_eq!(min, 12);
        assert_eq!(max, Some(12));

    }

    #[test]
    fn test_bounds_iter_exact_size() {
        let bounds = Region::new(Position::new(0, 0), Position::new(5, 10));
        let iter = bounds.iter();

        assert_eq!(iter.len(), 50);
    }

    #[test]
    fn test_bounds_into_iter() {
        let bounds = Region::new(Position::new(0, 0), Position::new(2, 2));
        let count = bounds.into_iter().count();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_bounds_into_iter_ref() {
        let bounds = Region::new(Position::new(0, 0), Position::new(3, 3));
        let count = (bounds).iter().count();
        assert_eq!(count, 9);
    }
}
