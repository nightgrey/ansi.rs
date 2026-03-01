use std::iter::FusedIterator;
use std::ops::{Bound, Deref, IntoBounds, RangeBounds};

use crate::{Position, Size, Iter, Step, Cursor};

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
pub struct Bounds {
    pub min: Position,
    pub max: Position,
}

impl Bounds {
    pub const ZERO: Self = Self {
        min: Position::ZERO,
        max: Position::ZERO,
    };

    /// Creates a new region from inclusive `min` to exclusive `max`.
    ///
    /// # Panics (debug only)
    /// Panics if `min > max` on either axis.
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

    /// Returns the intersection of two regions (may be empty).
    pub const fn intersect(self, other: Self) -> Self {
        let min_row = self.min.row.min(other.min.row);
        let min_col = self.min.col.min(other.min.col);
        let max_row = self.max.row.max(other.max.row);
        let max_col = self.max.col.max(other.max.col);

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

    /// Row-major iterator over every position in the region.
    pub const fn iter(&self) -> Iter {
        Iter::new(*self)
    }

    /// Cursor over every position in the region.
    pub const fn cursor(&self, position: Position) -> Cursor<'_, Position> {
        Cursor::new(self, position)
    }

}

impl IntoIterator for &Bounds {
    type Item = Position;
    type IntoIter = Iter;
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(*self)
    }
}

impl RangeBounds<Position> for Bounds {
    #[inline]
    fn start_bound(&self) -> Bound<&Position> {
        Bound::Included(&self.min)
    }
    #[inline]
    fn end_bound(&self) -> Bound<&Position> {
        Bound::Excluded(&self.max)
    }
}

impl IntoBounds<Position> for Bounds {
    #[inline]
    fn into_bounds(self) -> (Bound<Position>, Bound<Position>) {
        (Bound::Included(self.min), Bound::Excluded(self.max))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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

    // #[test]
    // fn test_bounds_from_rect() {
    //     let rect = Rect::new((10, 5), (30, 25));
    //     let bounds: Region = rect.into();
    //
    //     assert_eq!(bounds.min, Position::new(5, 10)); // (y, x)
    //     assert_eq!(bounds.max, Position::new(25, 30));
    // }

    // === SpatialIter Tests ===

}
