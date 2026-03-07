use crate::{Size};
use std::ops::{Add, AddAssign};

/// Type alias for tuple-based points: `(x, y)`.
///
/// This allows constructing [`Point`] from tuples conveniently:
///
/// ```rust
/// # use geometry::{Point, Rect};
/// let rect = Rect::new((0, 0), (10, 20));  // Uses PointLike
/// ```
pub type PointLike = (usize, usize);

/// A 2D point in screen-space coordinates.
///
/// Points use (x, y) coordinates where:
/// - `x` increases left to right (column)
/// - `y` increases top to bottom (row)
///
/// This matches terminal coordinate systems where (0, 0) is the top-left corner.
///
/// # Example
///
/// ```rust
/// use geometry::Point;
///
/// let p1 = Point::new(10, 5);
/// let p2 = Point::new(3, 2);
/// let sum = p1 + p2;
///
/// assert_eq!(sum, Point::new(13, 7));
/// ```
#[derive(Copy, Debug)]
#[derive_const(Clone, Default, PartialEq, Eq)]
pub struct Point {
    /// Horizontal position (column).
    pub x: usize,

    /// Vertical position (row).
    pub y: usize,
}

impl Point {
    /// The origin point (0, 0).
    pub const ZERO: Self = Self { x: 0, y: 0 };

    /// Create a new point at the given coordinates.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Point;
    /// let p = Point::new(5, 10);
    /// assert_eq!(p.x, 5);
    /// assert_eq!(p.y, 10);
    /// ```
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<Size> for Point {
    fn from(value: Size) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<PointLike> for Point {
    fn from(value: PointLike) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Point {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bounded, Contains};
    use super::*;
    use crate::rect::Rect;
    use crate::size::Size;

    // === Point Tests ===

    #[test]
    fn test_point_new() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    #[test]
    fn test_point_zero() {
        assert_eq!(Point::ZERO, Point::new(0, 0));
    }

    #[test]
    fn test_point_addition() {
        let p1 = Point::new(5, 10);
        let p2 = Point::new(3, 7);
        let result = p1 + p2;
        assert_eq!(result, Point::new(8, 17));
    }

    #[test]
    fn test_point_add_assign() {
        let mut p = Point::new(5, 10);
        p += Point::new(3, 7);
        assert_eq!(p, Point::new(8, 17));
    }

    #[test]
    fn test_point_from_tuple() {
        let p: Point = (10, 20).into();
        assert_eq!(p, Point::new(10, 20));
    }

    // === Size Tests ===

    #[test]
    fn test_size_new() {
        let s = Size::new(80, 24);
        assert_eq!(s.width, 80);
        assert_eq!(s.height, 24);
    }

    #[test]
    fn test_size_zero() {
        assert_eq!(Size::ZERO, Size::new(0, 0));
    }

    // === Rect Tests ===

    #[test]
    fn test_rect_new() {
        let r = Rect::new((10, 5), (30, 25));
        assert_eq!(r.min, Point::new(10, 5));
        assert_eq!(r.max, Point::new(30, 25));
    }

    #[test]
    fn test_rect_width_height() {
        let r = Rect::new((10, 5), (30, 25));
        assert_eq!(r.width(), 20);
        assert_eq!(r.height(), 20);
    }

    #[test]
    fn test_rect_inverted_returns_zero() {
        // Inverted rectangle should return 0 width/height
        let r = Rect::new((30, 25), (10, 5));
        assert_eq!(r.width(), 0);
        assert_eq!(r.height(), 0);
    }

    #[test]
    fn test_rect_contains_point() {
        let r = Rect::new((10, 10), (20, 20));

        // Inside
        assert!(r.contains(&Point::new(15, 15)));

        // On min edge (inclusive)
        assert!(r.contains(&Point::new(10, 10)));

        // On max edge (exclusive)
        assert!(!r.contains(&Point::new(20, 20)));

        // Outside
        assert!(!r.contains(&Point::new(25, 25)));
        assert!(!r.contains(&Point::new(5, 5)));
    }

    #[test]
    fn test_rect_area() {
        let r = Rect::new((0, 0), (10, 5));
        assert_eq!(r.len(), 50);

        let empty = Rect::new((5, 5), (5, 5));
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_rect_size() {
        let r = Rect::new((10, 5), (30, 25));
        let size = r.size();
        assert_eq!(size.width, 20);
        assert_eq!(size.height, 20);
    }

    #[test]
    fn test_rect_x_y() {
        let r = Rect::new((15, 25), (40, 60));
        assert_eq!(r.x(), 15);
        assert_eq!(r.y(), 25);
    }

    #[test]
    fn test_rect_zero() {
        assert_eq!(Rect::ZERO.width(), 0);
        assert_eq!(Rect::ZERO.height(), 0);
        assert_eq!(Rect::ZERO.len(), 0);
    }

    #[test]
    fn test_rect_saturating_operations() {
        // Test that operations use saturating arithmetic
        let r = Rect::new((10, 10), (5, 5)); // Inverted
        assert_eq!(r.width(), 0); // saturating_sub prevents underflow
        assert_eq!(r.height(), 0);
        assert_eq!(r.len(), 0); // saturating_mul
    }
}
