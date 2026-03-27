use std::ops::{Add, AddAssign, Div, DivAssign, Sub};
use crate::Zero;

/// A 2D size representing width and height.
///
/// Used to represent the dimensions of rectangles, nodes, and other 2D regions.
///
/// # Example
///
/// ```rust
/// use geometry::Size;
///
/// let size = Size::new(80, 24);
/// assert_eq!(size.width, 80);
/// assert_eq!(size.height, 24);
/// ```
#[derive(Copy, Debug)]
#[derive_const(Clone, Default, PartialEq, Eq)]
pub struct Size<T = usize> {
    /// Width in columns.
    pub width: T,

    /// Height in rows.
    pub height: T,
}

impl<T> Size<T> {
    /// Create a new size with the given dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Size;
    /// let size = Size::new(40, 12);
    /// ```
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
    
    /// Create a size with equal width and height.
    pub const fn both(value: T) -> Self where T: Copy {
        Self::new(value, value)
    }
}

impl<T: Zero> Size<T> {
    /// A size of zero (0×0).
    pub const ZERO: Self = Self {
        width: T::ZERO,
        height: T::ZERO,
    };
}

impl<U, T: Add<U, Output = T>> Add<Size<U>> for Size<T> {
    type Output = Size<T>;

    fn add(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<U, T: AddAssign<U>> AddAssign<Size<U>> for Size<T> {
    fn add_assign(&mut self, rhs: Size<U>) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl<U: Copy, T: Sub<U, Output = T>> Sub<U> for Size<T> {
    type Output = Size<T>;

    fn sub(self, rhs: U) -> Self::Output {
        Size {
            width: self.width - rhs,
            height: self.height - rhs,
        }
    }
}

impl<U: Copy, T: Div<U, Output = T>> Div<U> for Size<T> {
    type Output = Size<T>;

    fn div(self, rhs: U) -> Self::Output {
        Size {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl<U: Copy, T: DivAssign<U>> DivAssign<U> for Size<T> {
    fn div_assign(&mut self, rhs: U) {
        self.width /= rhs;
        self.height /= rhs;
    }
}
