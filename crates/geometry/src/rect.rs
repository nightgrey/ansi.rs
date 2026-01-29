use crate::size::Size;
use crate::{Edges, Point};

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
    pub const ZERO: Self = Self {
        min: Point::ZERO,
        max: Point::ZERO,
    };

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
    pub fn new(min: impl Into<Point>, max: impl Into<Point>) -> Self {
        Self {
            min: min.into(),
            max: max.into(),
        }
    }

    /// Create a rectangle from its bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Rect;
    /// let rect = Rect::bounds(10, 5, 20, 15);
    /// assert_eq!(rect.min, Point::new(10, 5));
    /// assert_eq!(rect.max, Point::new(30, 20));
    /// ```
    pub fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            min: Point::new(x, y),
            max: Point::new(x + width, y + height),
        }
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
        self.min.x <= point.x
            && point.x < self.max.x
            && self.min.y <= point.y
            && point.y < self.max.y
    }

    /// Shrink the rectangle by the given edges.
    pub const fn shrink(&self, edges: &Edges) -> Self {
        Self {
            min: Point {
                x: self.min.x + edges.left,
                y: self.min.y + edges.top,
            },
            max: Point {
                x: self.max.x.saturating_sub(edges.right),
                y: self.max.y.saturating_sub(edges.bottom),
            },
        }
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
