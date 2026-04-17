use std::fmt::{Debug, Formatter};
use std::ops::{Add, Sub};
use number::{Ops , SaturatingOps, Zero, SaturatingAdd, SaturatingSub};
use crate::{Location, Bound, Edges, Point, Resolve, Size, Step, Steps};

/// An axis-aligned rectangle for screen-space coordinates.
///
/// Rectangles are represented as half-open ranges: `[min, max)`.
/// The `min` point is inclusive, the `max` point is exclusive.
#[derive(Copy)]
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
    /// let rect1 = Rect::bounds(Point::new(0, 0), Point::new(10, 10));
    /// let rect2 = Rect::bounds(Point::new(0, 0), Point::new(10, 10));
    /// assert_eq!(rect1, rect2);
    /// ```
    pub const fn bounds(min: T, max: T) -> Self {
        Self { min, max }
    }

}

impl const Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            min: Point::new(x, y),
            max: Point::new(x + width, y + height),
        }
    }

    pub fn set_height(&mut self, height: u16) {
        self.max.y = self.min.y + height;
    }

    pub fn set_width(&mut self, width: u16) {
        self.max.x = self.min.x + width;
    }
}

impl<T: Zero> From<Size<T>> for Rect<Point<T>> {
    fn from(value: Size<T>) -> Self {
        Self::bounds(Point::ZERO, Point::new(value.width, value.height))
    }
}

impl<T: Ops> Add<Rect<T>> for Rect<T> {
    type Output = Self;

    fn add(self, rhs: Rect<T>) -> Self {
        Self {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

impl<T: Ops> Sub<Rect<T>> for Rect<T> {
    type Output = Self;

    fn sub(self, rhs: Rect<T>) -> Self {
        Self {
            min: self.min - rhs.min,
            max: self.max - rhs.max,
        }
    }
}

impl<T: SaturatingOps> SaturatingAdd<Rect<T>> for Rect<T> {
    fn saturating_add(self, rhs: Rect<T>) -> Self {
        let min = self.min.saturating_add(rhs.min);
        let max = self.max.saturating_add(rhs.max);

        Rect { min, max }
    }
}

impl<T: SaturatingOps> SaturatingSub<Rect<T>> for Rect<T> {
    fn saturating_sub(self, rhs: Rect<T>) -> Self {
        let min = self.min.saturating_sub(rhs.min);
        let max = self.max.saturating_sub(rhs.max);

        Rect { min, max }
    }
}


impl<T: Ops + Copy> Add<Point<T>> for Rect<Point<T>> {
    type Output = Self;

    fn add(self, rhs: Point<T>) -> Self {
        Self {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl<T: Ops + Copy> Sub<Point<T>> for Rect<Point<T>> {
    type Output = Self;

    fn sub(self, rhs: Point<T>) -> Self {
        Self {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl<T: Ops> Add<Edges<T>> for Rect<Point<T>> {
    type Output = Self;

    fn add(self, rhs: Edges<T>) -> Self {
        let min_x = self.min.x - rhs.left;
        let min_y = self.min.y - rhs.top;
        let max_x = self.max.x + rhs.right;
        let max_y = self.max.y + rhs.bottom;

        Rect {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }
}

impl<T: Ops + Copy> Sub<Edges<T>> for Rect<Point<T>> {
    type Output = Self;

    fn sub(self, rhs: Edges<T>) -> Self {
        let min_x = self.min.x + rhs.left;
        let min_y = self.min.y + rhs.top;
        let max_x = self.max.x - rhs.right;
        let max_y = self.max.y - rhs.bottom;

        Rect {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }
}

impl<T: Debug> Debug for Rect<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Rect").field(&self.min).field(&self.max).finish()
    }
}