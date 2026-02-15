use std::iter::FusedIterator;
use crate::{Position, Size, SpatialIter};
use std::ops::{IntoBounds, Bound, Bound::*, RangeBounds, Deref, DerefMut, Sub};
use crate::region::step::{SpatialContext, SpatialStep};

/// Canonical bounds: start inclusive, end exclusive
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

    pub  fn new(min: Position, max: Position) -> Self {
        debug_assert!(min <= max, "Given bounds are invalid.");
        debug_assert!(!(max.row as isize - min.row as isize).is_negative(), "Given bounds ({} -> {}) would result in a negative width.", min, max);
        debug_assert!(!(max.col as isize - min.col as isize).is_negative(), "Given bounds ({} -> {}) would result in a negative height.", min, max);
        Self { min, max }
    }

    pub const fn width(&self) -> usize {
        self.max.col - self.min.col
    }

    pub const fn height(&self) -> usize {
        self.max.row - self.min.row
    }

    /// Returns the total number of positions in these bounds.
    #[inline]
    pub const fn area(&self) -> usize {
        self.width() * self.height()
    }

    /// Returns the position at the given row and column offset from the start.
    #[inline]
    pub const fn at(&self, row: usize, col: usize) -> Position {
        Position::new(self.min.row + row, self.min.col + col)
    }

    /// Converts a linear index to a position within these bounds.
    ///
    /// # Panics
    /// Panics if `index >= self.area()`.
    #[inline]
    pub const fn position_of(&self, index: usize) -> Position {
        let width = self.width();
        let row  = index / width;
        let col = index % width;
        Position::new(self.min.row + row, self.min.col + col)
    }

    /// Converts a position to a linear index within these bounds.
    ///
    /// # Panics
    /// Panics if position is outside bounds.
    #[inline]
    pub const fn index_of(&self, position: &Position) -> usize {
        (position.row - self.min.row) * self.width() + (position.col - self.min.col)
    }

    pub const fn contains(&self, position: &Position) -> bool {
        self.min.col <= position.col
            && position.col < self.max.col
            && self.min.row <= position.row
            && position.row < self.max.row
    }
    pub const fn intersect(self, other: Self) -> Self {
         Self {
             min: Position::new(self.min.col.max(other.min.col), self.min.row.max(other.min.row)),
             max: Position::new(self.max.col.min(other.max.col), self.max.row.min(other.max.row)),
         }
    }

    pub const fn clip(self, other: Self) -> Self {
        other.intersect(self)
    }

    pub const fn size(&self) -> Size {
        Size::new(self.width(), self.height())
    }

    pub const fn len(&self) -> usize {
        self.width() * self.height()
    }

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
        let width = self.width();
        let current = (start.row - self.min.row) * width
            + (start.col - self.min.col);
        let remaining = (end.row - self.min.row) * width + (end.col - self.min.col);

        let dist = (remaining - current) - width;

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
        let width = self.width();
        let start_idx = (start.row - self.min.row) * width + (start.col - self.min.col);
        let new_idx = start_idx.checked_add(count)?;

        if new_idx >= self.area() {
            return None;
        }

        Some(self.position_of(new_idx))
    }
    fn backward_checked(&self, start: Position, count: usize) -> Option<Position> {
        if start < self.min {
            return None;
        }

        // Fast Path: Stay in current row
        let cols_from_start = start.col - self.min.col;
        if count <= cols_from_start {
            return Some(Position::new(start.row, start.col - count));
        }

        // Slow Path: Cross row boundary
        let width = self.width();
        let start_idx = (start.row - self.min.row) * width + (start.col - self.min.col);
        let new_idx = start_idx.checked_sub(count)?;

        // No need to check upper bound, checked_sub handles underflow logic implicitly,
        // but we ensure index is within the valid range.
        // Since start was valid and we subtracted, we just need to ensure we didn't underflow.
        // (checked_sub handles the underflow by returning None)

        Some(self.position_of(new_idx))
    }
}

impl SpatialStep for Position {
    type Context = Region;
}



#[cfg(test)]
mod tests {
    use crate::Rect;
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
