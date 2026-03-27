use crate::{Bounded, Edges, Location, Point, Size, Step, Steps, Zero};

/// An axis-aligned rectangle for screen-space coordinates.
///
/// Rectangles are represented as half-open ranges: `[min, max)`.
/// The `min` point is inclusive, the `max` point is exclusive.
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

    pub fn iter(self) -> Steps<Self, T>
    where
        Self: Bounded<Point = T> + Step<T>,
    {
        Steps::new(self)
    }
}

impl<T: [const] Zero> const Rect<T> {
    /// An empty rectangle at the origin.
    pub const ZERO: Self = Self {
        min: T::ZERO,
        max: T::ZERO,
    };
}

 impl<T: [const] Location> const Rect<T> {
    pub fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            min: T::new(x, y),
            max: T::new(x + width, y + height),
        }
    }
}

impl Rect {
    pub const fn shrink(self, edges: Edges) -> Self {
        let min_x = self.min.x.saturating_add(edges.left);
        let min_y = self.min.y.saturating_add(edges.top);
        let max_x = self.max.x.saturating_sub(edges.right);
        let max_y = self.max.y.saturating_sub(edges.bottom);

        Rect {
            min: Point {
                x: min_x.min(max_x),
                y: min_y.min(max_y),
            },
            max: Point { x: max_x, y: max_y },
        }
    }
}

impl From<Size> for Rect {
    fn from(value: Size) -> Self {
        Self::new(Point::ZERO, Point::new(value.width, value.height))
    }
}

impl<T: Location> IntoIterator for &Rect<T>
where
    Rect<T>: Bounded<Point = T> + Step<T>,
    Steps<Rect<T>, T>: Iterator,
{
    type Item = <Steps<Rect<T>, T> as Iterator>::Item;
    type IntoIter = Steps<Rect<T>, T>;

    fn into_iter(self) -> Self::IntoIter {
        Steps::new(*self)
    }
}
