use std::ops::{Add, AddAssign, Div, DivAssign, Sub};
use crate::{Number, Zero};

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


impl<T: Add<T, Output = T>> Add<Size<T>> for Size<T> {
    type Output = Size<T>;

    fn add(self, rhs: Size<T>) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T: AddAssign<T>> AddAssign<Size<T>> for Size<T> {
    fn add_assign(&mut self, rhs: Size<T>) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl<T: Sub<T, Output = T> + Copy> Sub<T> for Size<T> {
    type Output = Size<T>;

    fn sub(self, rhs: T) -> Self::Output {
        Size {
            width: self.width - rhs,
            height: self.height - rhs,
        }
    }
}

impl<T: Div<T, Output = T> + Copy> Div<T> for Size<T> {
    type Output = Size<T>;

    fn div(self, rhs: T) -> Self::Output {
        Size {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl<T: DivAssign<T> + Copy> DivAssign<T> for Size<T> {
    fn div_assign(&mut self, rhs: T) {
        self.width /= rhs;
        self.height /= rhs;
    }
}
