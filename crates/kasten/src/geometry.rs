use std::ops::{Add, AddAssign};
use crate::position::Position;

/// Type alias for tuple-based points: `(x, y)`.
///
/// This allows constructing [`Point`] from tuples conveniently:
///
/// ```rust
/// # use kasten::{Point, Rect};
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
/// use kasten::Point;
///
/// let p1 = Point::new(10, 5);
/// let p2 = Point::new(3, 2);
/// let sum = p1 + p2;
///
/// assert_eq!(sum, Point::new(13, 7));
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq)]
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
    /// # use kasten::Point;
    /// let p = Point::new(5, 10);
    /// assert_eq!(p.x, 5);
    /// assert_eq!(p.y, 10);
    /// ```
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<PointLike> for Point {
    fn from(value: PointLike) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Position> for Point {
    fn from(value: Position) -> Self {
        Self::new(value.col, value.row)
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
    use super::*;

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
        assert_eq!(r.area(), 50);

        let empty = Rect::new((5, 5), (5, 5));
        assert_eq!(empty.area(), 0);
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
        assert_eq!(Rect::ZERO.area(), 0);
    }

    #[test]
    fn test_rect_saturating_operations() {
        // Test that operations use saturating arithmetic
        let r = Rect::new((10, 10), (5, 5)); // Inverted
        assert_eq!(r.width(), 0); // saturating_sub prevents underflow
        assert_eq!(r.height(), 0);
        assert_eq!(r.area(), 0); // saturating_mul
    }
}

/// A 2D size representing width and height.
///
/// Used to represent the dimensions of rectangles, nodes, and other 2D regions.
///
/// # Example
///
/// ```rust
/// use kasten::Size;
///
/// let size = Size::new(80, 24);
/// assert_eq!(size.width, 80);
/// assert_eq!(size.height, 24);
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Size {
    /// Width in columns.
    pub width: usize,

    /// Height in rows.
    pub height: usize
}

impl Size {
    /// A size of zero (0×0).
    pub const ZERO: Self = Self { width: 0, height: 0 };

    /// Create a new size with the given dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Size;
    /// let size = Size::new(40, 12);
    /// ```
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}


/// An axis-aligned rectangle defined by min and max points.
///
/// Rectangles are represented as half-open ranges: `[min, max)`.
/// The `min` point is inclusive, the `max` point is exclusive.
///
/// # Inverted Rectangles
///
/// If `min > max` in any dimension, the rectangle is considered "inverted".
/// Methods like [`width()`](Self::width) and [`height()`](Self::height) use
/// saturating subtraction to return 0 for inverted dimensions.
///
/// # Example
///
/// ```rust
/// use kasten::{Rect, Point, Size};
///
/// let rect = Rect::new((10, 5), (30, 25));
/// assert_eq!(rect.width(), 20);
/// assert_eq!(rect.height(), 20);
/// assert_eq!(rect.area(), 400);
///
/// let point = Point::new(15, 10);
/// assert!(rect.contains(&point));
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rect {
    /// Minimum (top-left) point (inclusive).
    pub min: Point,

    /// Maximum (bottom-right) point (exclusive).
    pub max: Point,
}

impl Rect {
    /// An empty rectangle at the origin.
    pub const ZERO: Self = Self { min: Point::ZERO, max: Point::ZERO };

    /// Create a new rectangle from min and max points.
    ///
    /// Accepts anything convertible to [`Point`], including tuples.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Rect, Point};
    /// let rect1 = Rect::new((0, 0), (10, 10));
    /// let rect2 = Rect::new(Point::new(0, 0), Point::new(10, 10));
    /// assert_eq!(rect1, rect2);
    /// ```
    pub  fn new(min: impl Into<Point>, max: impl Into<Point>) -> Self {
        Self { min: min.into(), max: max.into() }
    }

    /// Get the x-coordinate of the rectangle (left edge).
    ///
    /// Equivalent to `self.min.x`.
    pub const fn x(&self) -> usize {
        self.min.x
    }

    /// Get the y-coordinate of the rectangle (top edge).
    ///
    /// Equivalent to `self.min.y`.
    pub const fn y(&self) -> usize {
        self.min.y
    }

    /// Calculate the width of the rectangle.
    ///
    /// Returns 0 if the rectangle is inverted (min.x > max.x).
    /// Uses saturating subtraction to handle this case.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Rect;
    /// let rect = Rect::new((5, 0), (15, 0));
    /// assert_eq!(rect.width(), 10);
    /// ```
    pub const fn width(&self) -> usize {
        self.max.x.saturating_sub(self.min.x)
    }

    /// Calculate the height of the rectangle.
    ///
    /// Returns 0 if the rectangle is inverted (min.y > max.y).
    /// Uses saturating subtraction to handle this case.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Rect;
    /// let rect = Rect::new((0, 5), (0, 20));
    /// assert_eq!(rect.height(), 15);
    /// ```
    pub const fn height(&self) -> usize {
        self.max.y.saturating_sub(self.min.y)
    }

    /// Calculate the area of the rectangle.
    ///
    /// Returns `width() * height()`. For inverted rectangles, returns 0.
    /// Uses saturating multiplication to prevent overflow.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Rect;
    /// let rect = Rect::new((0, 0), (10, 5));
    /// assert_eq!(rect.area(), 50);
    /// ```
    pub const fn area(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    /// Check if a point is contained within the rectangle.
    ///
    /// The rectangle is treated as a half-open range: `[min, max)`.
    /// Points on the min edges are included, points on the max edges are excluded.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Rect, Point};
    /// let rect = Rect::new((0, 0), (10, 10));
    ///
    /// assert!(rect.contains(&Point::new(0, 0)));    // min edge (inclusive)
    /// assert!(rect.contains(&Point::new(5, 5)));    // inside
    /// assert!(!rect.contains(&Point::new(10, 10))); // max edge (exclusive)
    /// assert!(!rect.contains(&Point::new(15, 5)));  // outside
    /// ```
    pub const fn contains(&self, point: &Point) -> bool {
        self.min.x <= point.x && point.x < self.max.x
            && self.min.y <= point.y && point.y < self.max.y
    }

    /// Get the size of the rectangle as a [`Size`] struct.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Rect, Size};
    /// let rect = Rect::new((0, 0), (80, 24));
    /// assert_eq!(rect.size(), Size::new(80, 24));
    /// ```
    pub const fn size(&self) -> Size {
        Size {
            width: self.max.x.saturating_sub(self.min.x),
            height: self.max.y.saturating_sub(self.min.y),
        }
    }
}
