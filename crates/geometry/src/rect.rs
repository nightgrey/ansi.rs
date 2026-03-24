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
#[derive(Copy, Debug)]
#[derive_const(Default, Clone, Eq, PartialEq)]
pub struct Rect<T = Point> {
    /// Minimum (top-left) point (inclusive).
    pub min: T,

    /// Maximum (bottom-right) point (exclusive).
    pub max: T,
}

impl<T> Rect<T> {
    /// Create a new rectangle from min and max points.
    ///
    /// Accepts anything convertible to [`Point`], including tuples.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Rect, Point};
    /// let rect1 = Rect::new(Point::new(0, 0), Point::new(10, 10));
    /// let rect2 = Rect::new(Point::new(0, 0), Point::new(10, 10));
    /// assert_eq!(rect1, rect2);
    /// ```
    pub const fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

}

impl Rect {
    /// An empty rectangle at the origin.
    pub const ZERO: Self = Self {
        min: Point::ZERO,
        max: Point::ZERO,
    };

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            min: Point::new(x, y),
            max: Point::new(x + width, y + height),
        }
    }

    pub const fn shrink(self, edges: Edges) -> Self {
        let min_x = self.min.x.saturating_add(edges.left);
        let min_y = self.min.y.saturating_add(edges.top);
        let max_x = self.max.x.saturating_sub(edges.right);
        let max_y = self.max.y.saturating_sub(edges.bottom);

        Rect {
            min: Point { x: min_x.min(max_x), y: min_y.min(max_y) },
            max: Point { x: max_x, y: max_y },
        }

    }
}

