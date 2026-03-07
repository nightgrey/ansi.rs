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
/// use geometry::{Rect, Point, Size};
///
/// let rect = Rect::new((10, 5), (30, 25));
/// assert_eq!(rect.width(), 20);
/// assert_eq!(rect.height(), 20);
/// assert_eq!(rect.len(), 400);
///
/// let point = Point::new(15, 10);
/// assert!(rect.contains(&point));
/// ```
#[derive(Copy, Debug)]
#[derive_const(Default, Clone, Eq, PartialEq)]
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
    /// # use geometry::{Rect, Point};
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
    /// # use geometry::{Rect, Point};
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
    /// # use geometry::Rect;
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
    /// # use geometry::Rect;
    /// let rect = Rect::new((0, 5), (0, 20));
    /// assert_eq!(rect.height(), 15);
    /// ```
    pub const fn height(&self) -> usize {
        self.max.y.saturating_sub(self.min.y)
    }

    /// Get the size of the rectangle as a [`Size`] struct.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Rect, Size};
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
