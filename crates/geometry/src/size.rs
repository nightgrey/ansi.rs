use std::ops::{Add, AddAssign, Div, DivAssign};

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
    pub height: usize,
}

impl Size {
    /// A size of zero (0×0).
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
    };

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

impl Div<usize> for Size {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl DivAssign<usize> for Size {
    fn div_assign(&mut self, rhs: usize) {
        *self = *self / rhs;
    }
}
