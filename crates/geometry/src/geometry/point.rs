use crate::Size;
use number::{AssignOps, Number, One, Ops, SaturatingAdd, SaturatingOps, SaturatingSub, Zero};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Type alias for tuple-based points: `(x, y)`.
pub type PointLike<T = u16> = (T, T);

impl<T> From<PointLike<T>> for Point<T> {
    fn from(value: PointLike<T>) -> Self {
        Self::new(value.0, value.1)
    }
}

/// A 2D point in screen-space coordinates.
///
/// - `x` => left to right (column)
/// - `y` => top to bottom (row)
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
#[derive(Copy)]
#[derive_const(Clone, Default, PartialEq, Eq)]
pub struct Point<T = u16> {
    /// Horizontal position (column).
    pub x: T,

    /// Vertical position (row).
    pub y: T,
}

impl<T> Point<T> {
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
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: One> Point<T> {
    pub const ONE: Self = Point {
        x: T::ONE,
        y: T::ONE,
    };
}

impl<T: Zero> Point<T> {
    pub const ZERO: Self = Point {
        x: T::ZERO,
        y: T::ZERO,
    };
}

impl<T: Ops> Add for Point<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: AssignOps> AddAssign for Point<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Ops> Sub for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: AssignOps> SubAssign for Point<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: SaturatingOps> SaturatingAdd for Point<T> {
    fn saturating_add(self, rhs: Self) -> Self {
        Self {
            x: self.x.saturating_add(rhs.x),
            y: self.y.saturating_add(rhs.y),
        }
    }
}
impl<T: SaturatingOps> SaturatingSub for Point<T> {
    fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            x: self.x.saturating_sub(rhs.x),
            y: self.y.saturating_sub(rhs.y),
        }
    }
}

impl<T: Ord> PartialOrd for Point<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Point<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.y.cmp(&other.y) {
            std::cmp::Ordering::Equal => self.x.cmp(&other.x),
            ord => ord,
        }
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl<T: Debug> Debug for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}, {:?}]", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::rect::Rect;
    use crate::geometry::size::Size;
    use crate::{Bound, Contains};

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
        let r = Rect::bounds(Point::new(10, 5), Point::new(30, 25));
        assert_eq!(r.min, Point::new(10, 5));
        assert_eq!(r.max, Point::new(30, 25));
    }

    #[test]
    fn test_rect_width_height() {
        let r = Rect::bounds(Point::new(10, 5), Point::new(30, 25));
        assert_eq!(r.width(), 20);
        assert_eq!(r.height(), 20);
    }

    #[test]
    fn test_rect_inverted_returns_zero() {
        // Inverted rectangle should return 0 width/height
        let r = Rect::bounds(Point::new(30, 25), Point::new(10, 5));
        assert_eq!(r.width(), 0);
        assert_eq!(r.height(), 0);
    }

    #[test]
    fn test_rect_contains_point() {
        let r = Rect::bounds(Point::new(10, 10), Point::new(20, 20));

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
        let r = Rect::bounds(Point::new(0, 0), Point::new(10, 5));
        assert_eq!(r.len(), 50);

        let empty = Rect::bounds(Point::new(5, 5), Point::new(5, 5));
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_rect_size() {
        let r = Rect::bounds(Point::new(10, 5), Point::new(30, 25));
        let size = r.size();
        assert_eq!(size.width, 20);
        assert_eq!(size.height, 20);
    }

    #[test]
    fn test_rect_zero() {
        assert_eq!(Rect::bounds(Point::ZERO, Point::ZERO).width(), 0);
        assert_eq!(Rect::bounds(Point::ZERO, Point::ZERO).height(), 0);
        assert_eq!(Rect::bounds(Point::ZERO, Point::ZERO).len(), 0);
    }

    #[test]
    fn test_rect_saturating_operations() {
        // Test that operations use saturating arithmetic
        let r = Rect::bounds(Point::new(10, 10), Point::new(5, 5)); // Inverted
        assert_eq!(r.width(), 0); // saturating_sub prevents underflow
        assert_eq!(r.height(), 0);
        assert_eq!(r.len(), 0); // saturating_mul
    }
}
