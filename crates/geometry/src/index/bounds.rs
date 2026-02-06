use std::collections::Bound;
use crate::{Point, Position, Rect, Size, PointsIter};
use std::iter::FusedIterator;
use std::ops::{Range, RangeBounds};
use std::ops::{Add, AddAssign, Sub};
use super::{PositionsIter};

/// A rectangular region of buffer positions.
///
/// A region is defined by min and max positions, representing a half-open range:
/// `[min, max)`. The min position is inclusive, the max position is exclusive.
///
/// Regions are iterable and yield positions in row-major order (left-to-right,
/// top-to-bottom).
///
/// # Example
///
/// ```rust
/// use geometry::{Bounds, Position};
///
/// let region = Bounds::new(Position::new(0, 0), Position::new(2, 3));
/// assert_eq!(region.width(), 3);
/// assert_eq!(region.height(), 2);
/// assert_eq!(region.area(), 6);
///
/// // Iterate over all positions
/// let positions: Vec<_> = region.into_iter().collect();
/// assert_eq!(positions.len(), 6);
/// assert_eq!(positions[0], Position::new(0, 0));  // Top-left
/// assert_eq!(positions[2], Position::new(0, 2));  // First row, last column
/// assert_eq!(positions[3], Position::new(1, 0));  // Second row, first column
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bounds {
    /// Minimum (top-left) position (inclusive).
    pub min: Position,

    /// Maximum (bottom-right) position (exclusive).
    pub max: Position,
}

impl Bounds {
    /// An empty region at the origin.
    pub const ZERO: Self = Self {
        min: Position::ZERO,
        max: Position::ZERO,
    };

    /// Create a new region from min and max positions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let region = Bounds::new(Position::new(5, 10), Position::new(15, 30));
    /// ```
    pub const fn new(min: Position, max: Position) -> Self {
        Self { min, max }
    }
    
    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self::new(Position::new(x, y), Position::new(x + width, y + height))
    }

    /// Get the minimum (top-left) position.
    pub const fn min(&self) -> Position {
        self.min
    }

    /// Get the maximum (bottom-right) position.
    pub const fn max(&self) -> Position {
        self.max
    }

    /// Get the starting column (left edge).
    pub const fn x(&self) -> usize {
        self.min.col
    }

    /// Get the starting row (top edge).
    pub const fn y(&self) -> usize {
        self.min.row
    }

    /// Calculate the width of the region.
    ///
    /// Returns 0 if the region is inverted (min.col > max.col).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let region = Bounds::new(Position::new(0, 5), Position::new(0, 15));
    /// assert_eq!(region.width(), 10);
    /// ```
    pub const fn width(&self) -> usize {
        self.max.col.saturating_sub(self.min.col)
    }

    /// Calculate the height of the region.
    ///
    /// Returns 0 if the region is inverted (min.row > max.row).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let region = Bounds::new(Position::new(5, 0), Position::new(20, 0));
    /// assert_eq!(region.height(), 15);
    /// ```
    pub const fn height(&self) -> usize {
        self.max.row.saturating_sub(self.min.row)
    }

    /// Calculate the area of the region (width × height).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let region = Bounds::new(Position::new(0, 0), Position::new(4, 5));
    /// assert_eq!(region.area(), 20);  // 4 rows × 5 cols
    /// ```
    pub const fn area(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    /// Check if a position is contained within this region.
    ///
    /// The region is treated as half-open: `[min, max)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let region = Bounds::new(Position::new(0, 0), Position::new(10, 10));
    ///
    /// assert!(region.contains(&Position::new(0, 0)));    // min (inclusive)
    /// assert!(region.contains(&Position::new(5, 5)));    // inside
    /// assert!(!region.contains(&Position::new(10, 10))); // max (exclusive)
    /// ```
    pub const fn contains(&self, point: &Position) -> bool {
        self.min.col <= point.col
            && point.col < self.max.col
            && self.min.row <= point.row
            && point.row < self.max.row
    }

    /// Get the size of the region.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position, Size};
    /// let region = Bounds::new(Position::new(0, 0), Position::new(24, 80));
    /// assert_eq!(region.size(), Size::new(80, 24));  // width, height
    /// ```
    pub const fn size(&self) -> Size {
        Size {
            width: self.max.col.saturating_sub(self.min.col),
            height: self.max.row.saturating_sub(self.min.row),
        }
    }

    /// Create an iterator over all positions in this region.
    ///
    /// Positions are yielded in row-major order (left-to-right, top-to-bottom).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let region = Bounds::new(Position::new(0, 0), Position::new(2, 2));
    /// let positions: Vec<_> = region.iter().collect();
    /// assert_eq!(positions, vec![
    ///     Position::new(0, 0), Position::new(0, 1),
    ///     Position::new(1, 0), Position::new(1, 1),
    /// ]);
    /// ```
    pub  fn iter(&self) -> PositionsIter {
        PositionsIter::new(*self)
    }
}

impl<T: RangeBounds<Position>> From<T> for Bounds {
    fn from(value: T) -> Self {
        use std::ops::Bound;

        let start = match value.start_bound() {
            Bound::Unbounded => Position::new(usize::MIN, usize::MIN),
            Bound::Included(&p) | Bound::Excluded(&p) => p,
        };

        let end = match value.end_bound() {
            Bound::Unbounded => Position::new(usize::MAX, usize::MAX),
            Bound::Included(&p) => Position::new(p.row + 1, p.col + 1),
            Bound::Excluded(&p) => p,
        };

        Self::new(start, end)
    }
}
impl From<Rect> for Bounds {
    fn from(value: Rect) -> Self {
        Self::new(Position::from(value.min), Position::from(value.max))
    }
}

impl IntoIterator for Bounds {
    type Item = Position;
    type IntoIter = PositionsIter;

    fn into_iter(self) -> Self::IntoIter {
        PositionsIter::new(self)
    }
}

impl IntoIterator for &Bounds {
    type Item = Position;
    type IntoIter = PositionsIter;

    fn into_iter(self) -> Self::IntoIter {
        PositionsIter::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Position Tests ===

    #[test]
    fn test_position_new() {
        let p = Position::new(5, 10);
        assert_eq!(p.row, 5);
        assert_eq!(p.col, 10);
    }

    #[test]
    fn test_position_zero() {
        assert_eq!(Position::ZERO, Position::new(0, 0));
    }

    #[test]
    fn test_position_manhattan_distance() {
        let p = Position::new(3, 4);
        assert_eq!(p.manhattan(), 7);

        let origin = Position::ZERO;
        assert_eq!(origin.manhattan(), 0);
    }

    #[test]
    fn test_position_chebyshev_distance() {
        let p = Position::new(3, 7);
        assert_eq!(p.chebyshev(), 7); // max(3, 7)

        let p2 = Position::new(8, 4);
        assert_eq!(p2.chebyshev(), 8); // max(8, 4)

        let origin = Position::ZERO;
        assert_eq!(origin.chebyshev(), 0);
    }

    #[test]
    fn test_position_addition() {
        let p1 = Position::new(2, 3);
        let p2 = Position::new(4, 5);
        let result = p1 + p2;
        assert_eq!(result, Position::new(6, 8));
    }

    #[test]
    fn test_position_add_assign() {
        let mut p = Position::new(2, 3);
        p += Position::new(4, 5);
        assert_eq!(p, Position::new(6, 8));
    }

    #[test]
    fn test_position_from_tuple() {
        let p: Position = (5, 10).into();
        assert_eq!(p, Position::new(5, 10));
    }

    #[test]
    fn test_position_from_point() {
        let point = Point::new(10, 5); // x=10, y=5
        let pos: Position = point.into();
        assert_eq!(pos, Position::new(5, 10)); // row=y=5, col=x=10
    }

    // === Region Tests ===

    #[test]
    fn test_region_new() {
        let r = Bounds::new(Position::new(5, 10), Position::new(15, 30));
        assert_eq!(r.min, Position::new(5, 10));
        assert_eq!(r.max, Position::new(15, 30));
    }

    #[test]
    fn test_region_width_height() {
        let r = Bounds::new(Position::new(0, 0), Position::new(5, 10));
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 5);
    }

    #[test]
    fn test_region_area() {
        let r = Bounds::new(Position::new(0, 0), Position::new(4, 5));
        assert_eq!(r.area(), 20); // 4 * 5
    }

    #[test]
    fn test_region_contains() {
        let r = Bounds::new(Position::new(10, 10), Position::new(20, 20));

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
    fn test_region_size() {
        let r = Bounds::new(Position::new(0, 0), Position::new(24, 80));
        let size = r.size();
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }

    #[test]
    fn test_region_x_y() {
        let r = Bounds::new(Position::new(15, 25), Position::new(40, 60));
        assert_eq!(r.x(), 25); // min col
        assert_eq!(r.y(), 15); // min row
    }

    #[test]
    fn test_region_from_rect() {
        let rect = Rect::new((10, 5), (30, 25));
        let region: Bounds = rect.into();

        assert_eq!(region.min, Position::new(5, 10)); // (y, x)
        assert_eq!(region.max, Position::new(25, 30));
    }

    #[test]
    fn test_region_from_range() {
        let start = Position::new(0, 0);
        let end = Position::new(5, 10);
        let region: Bounds = (start..end).into();

        assert_eq!(region.min, start);
        assert_eq!(region.max, end);
    }

    // === SpatialIter Tests ===

    #[test]
    fn test_region_iter_basic() {
        let region = Bounds::new(Position::new(0, 0), Position::new(2, 3));
        let positions: Vec<_> = region.iter().collect();

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
    fn test_region_iter_empty_width() {
        let region = Bounds::new(Position::new(0, 5), Position::new(2, 5));
        let positions: Vec<_> = region.iter().collect();
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn test_region_iter_empty_height() {
        let region = Bounds::new(Position::new(5, 0), Position::new(5, 10));
        let positions: Vec<_> = region.iter().collect();
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn test_region_iter_single_cell() {
        let region = Bounds::new(Position::new(5, 10), Position::new(6, 11));
        let positions: Vec<_> = region.iter().collect();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position::new(5, 10));
    }

    #[test]
    fn test_region_iter_size_hint() {
        let region = Bounds::new(Position::new(0, 0), Position::new(3, 4));
        let iter = region.iter();
        let (min, max) = iter.size_hint();

        assert_eq!(min, 12);
        assert_eq!(max, Some(12));
    }

    #[test]
    fn test_region_iter_double_ended() {
        let region = Bounds::new(Position::new(0, 0), Position::new(2, 2));
        let mut iter = region.iter();

        // Forward
        assert_eq!(iter.next(), Some(Position::new(0, 0)));

        // Backward
        assert_eq!(iter.next_back(), Some(Position::new(1, 1)));

        // Forward again
        assert_eq!(iter.next(), Some(Position::new(0, 1)));

        // Backward again
        assert_eq!(iter.next_back(), Some(Position::new(1, 0)));

        // Should be exhausted
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_region_iter_exact_size() {
        let region = Bounds::new(Position::new(0, 0), Position::new(5, 10));
        let iter = region.iter();

        assert_eq!(iter.len(), 50);
    }

    #[test]
    fn test_region_into_iter() {
        let region = Bounds::new(Position::new(0, 0), Position::new(2, 2));
        let count = region.into_iter().count();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_region_into_iter_ref() {
        let region = Bounds::new(Position::new(0, 0), Position::new(3, 3));
        let count = (&region).into_iter().count();
        assert_eq!(count, 9);
    }
}
