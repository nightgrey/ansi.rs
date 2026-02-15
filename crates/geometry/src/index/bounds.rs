use crate::{Point, Position, Rect, Size};
use std::collections::Bound;
use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, IntoBounds, Sub};
use std::ops::{ RangeBounds};

/// A rectangular bounds of buffer positions.
///
/// A bounds is defined by min and max positions, representing a half-open range:
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
/// let bounds = Bounds::new(Position::new(0, 0), Position::new(2, 3));
/// assert_eq!(bounds.width(), 3);
/// assert_eq!(bounds.height(), 2);
/// assert_eq!(bounds.area(), 6);
///
/// // Iterate over all positions
/// let positions: Vec<_> = bounds.into_iter().collect();
/// assert_eq!(positions.len(), 6);
/// assert_eq!(positions[0], Position::new(0, 0));  // Top-left
/// assert_eq!(positions[2], Position::new(0, 2));  // First row, last column
/// assert_eq!(positions[3], Position::new(1, 0));  // Second row, first column
/// ```
#[derive_const(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bounds {
    /// Minimum (top-left) position (inclusive).
    pub min: Position,

    /// Maximum (bottom-right) position (exclusive).
    pub max: Position,
}

impl Bounds {
    /// An empty bounds at the origin.
    pub const ZERO: Self = Self {
        min: Position::ZERO,
        max: Position::ZERO,
    };

    /// Create a new bounds from min and max positions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::new(Position::new(5, 10), Position::new(15, 30));
    /// ```
    pub const fn new(min: Position, max: Position) -> Self {
        Self { min, max }
    }

    /// Create a new bounds from its top-left corner and size.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::rect(5, 10, 10, 20);
    /// ```
    pub const fn rect(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self::new(Position::new(x, y), Position::new(x + width, y + height))
    }

    /// Create a new bounds from a range of positions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::ranged((Position::new(5, 10)..Position::new(15, 30)));
    /// ```
    pub const fn ranged(range: impl [const] IntoBounds<Position>) -> Self {
        let (start, end) = range.into_bounds();

        let min = match start {
            Bound::Included(p) => p,
            Bound::Excluded(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Unbounded => Position { row: 0, col: 0 },
        };

        let max = match end {
            Bound::Included(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Excluded(p) => p,
            Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
        };

        let width = min.row.saturating_sub(max.row);

        Bounds { min: wrap(min, width), max: wrap(max, width) }
    }

    /// Create a new bounds that is within another bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::within((Position::new(5, 10)..Position::new(15, 30)), (Position::new(0, 0)..Position::new(20, 20)));
    /// ```
    pub const fn within(range: impl [const] IntoBounds<Position>, within: impl [const] IntoBounds<Position>) -> Self {
        let (start, end) = range.into_bounds();
        let (within_start, within_end) = within.into_bounds();

        let min = match start {
            Bound::Included(p) => p,
            Bound::Excluded(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Unbounded => match within_start {
                Bound::Included(p) => p,
                Bound::Excluded(p) => Position { row: p.row, col: p.col + 1 },
                Bound::Unbounded => Position { row: 0, col: 0 },
            },
        };

        let max = match end {
            Bound::Included(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Excluded(p) => p,
            Bound::Unbounded => match within_end {
                Bound::Included(p) => Position { row: p.row, col: p.col + 1 },
                Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
            },
        };

        let width = min.row.saturating_sub(max.row);

        Bounds { min: wrap(min, width), max: wrap(max, width) }
    }

    /// Get the starting column (left edge).
    pub const fn x(&self) -> usize {
        self.min.col
    }

    /// Get the starting row (top edge).
    pub const fn y(&self) -> usize {
        self.min.row
    }

    /// Calculate the width of the bounds.
    ///
    /// Returns 0 if the bounds is inverted (min.col > max.col).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::new(Position::new(0, 5), Position::new(0, 15));
    /// assert_eq!(bounds.width(), 10);
    /// ```
    pub const fn width(&self) -> usize {
        self.max.col.saturating_sub(self.min.col)
    }

    /// Calculate the height of the bounds.
    ///
    /// Returns 0 if the bounds is inverted (min.row > max.row).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::new(Position::new(5, 0), Position::new(20, 0));
    /// assert_eq!(bounds.height(), 15);
    /// ```
    pub const fn height(&self) -> usize {
        self.max.row.saturating_sub(self.min.row)
    }

    /// Calculate the area of the bounds (width × height).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::new(Position::new(0, 0), Position::new(4, 5));
    /// assert_eq!(bounds.area(), 20);  // 4 rows × 5 cols
    /// ```
    pub const fn area(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    /// Check if a position is contained within this bounds.
    ///
    /// The bounds is treated as half-open: `[min, max)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::new(Position::new(0, 0), Position::new(10, 10));
    ///
    /// assert!(bounds.contains(&Position::new(0, 0)));    // min (inclusive)
    /// assert!(bounds.contains(&Position::new(5, 5)));    // inside
    /// assert!(!bounds.contains(&Position::new(10, 10))); // max (exclusive)
    /// ```
    pub const fn contains(&self, point: &Position) -> bool {
        self.min.col <= point.col
            && point.col < self.max.col
            && self.min.row <= point.row
            && point.row < self.max.row
    }


    /// Calculate the intersection of two boundss.
    pub const fn intersect(self, other: Self) -> Self {
        Self::new(
            Position::new(self.min.col.max(other.min.col), self.min.row.max(other.min.row)),
            Position::new(self.max.col.min(other.max.col), self.max.row.min(other.max.row)),
        )
    }

    /// Clip this bounds to fit within another bounds.
    pub fn clip(self, other: Self) -> Self {
        other.intersect(self)
    }
    /// Get the size of the bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position, Size};
    /// let bounds = Bounds::new(Position::new(0, 0), Position::new(24, 80));
    /// assert_eq!(bounds.size(), Size::new(80, 24));  // width, height
    /// ```
    pub const fn size(&self) -> Size {
        Size {
            width: self.max.col.saturating_sub(self.min.col),
            height: self.max.row.saturating_sub(self.min.row),
        }
    }
}


impl RangeBounds<Position> for Bounds {
    fn start_bound(&self) -> Bound<&Position> {
        Bound::Included(&self.min)
    }

    fn end_bound(&self) -> Bound<&Position> {
        Bound::Excluded(&self.max)
    }
}

impl IntoBounds<Position> for Bounds {
    fn into_bounds(self) -> (Bound<Position>, Bound<Position>) {
        (Bound::Included(self.min), Bound::Excluded(self.max))
    }
}

impl From<(Bound<Position>, Bound<Position>)> for Bounds {
    fn from(value: (Bound<Position>, Bound<Position>)) -> Self {
        Self::ranged(value)
    }
}

impl From<Rect> for Bounds {
    fn from(value: Rect) -> Self {
        Self::new(Position::from(value.min), Position::from(value.max))
    }
}

// row wrapping for linear bounds
const fn wrap(p: Position, width: usize) -> Position {
    if p.col >= width {
        Position {
            row: p.row + p.col / width,
            col: p.col % width,
        }
    } else { p }
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
    fn test_bounds_new() {
        let r = Bounds::new(Position::new(5, 10), Position::new(15, 30));
        assert_eq!(r.min, Position::new(5, 10));
        assert_eq!(r.max, Position::new(15, 30));
    }

    #[test]
    fn test_bounds_width_height() {
        let r = Bounds::new(Position::new(0, 0), Position::new(5, 10));
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 5);
    }

    #[test]
    fn test_bounds_area() {
        let r = Bounds::new(Position::new(0, 0), Position::new(4, 5));
        assert_eq!(r.area(), 20); // 4 * 5
    }

    #[test]
    fn test_bounds_contains() {
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
    fn test_bounds_size() {
        let r = Bounds::new(Position::new(0, 0), Position::new(24, 80));
        let size = r.size();
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }

    #[test]
    fn test_bounds_x_y() {
        let r = Bounds::new(Position::new(15, 25), Position::new(40, 60));
        assert_eq!(r.x(), 25); // min col
        assert_eq!(r.y(), 15); // min row
    }

    #[test]
    fn test_bounds_from_rect() {
        let rect = Rect::new((10, 5), (30, 25));
        let bounds: Bounds = rect.into();

        assert_eq!(bounds.min, Position::new(5, 10)); // (y, x)
        assert_eq!(bounds.max, Position::new(25, 30));
    }

    // === SpatialIter Tests ===

}
