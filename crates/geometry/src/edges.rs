use std::ops::Add;

/// Edge insets for padding or margins.
///
/// Represents spacing on four sides.
///
/// # Example
///
/// ```rust
/// use geometry::Edges;
///
/// let edges = Edges::new(1, 2, 1, 2);  // top, right, bottom, left
/// assert_eq!(edges.horizontal(), 4);  // left + right
/// assert_eq!(edges.vertical(), 2);     // top + bottom
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Edges<T = usize> {
    /// Spacing from the top edge.
    pub top: T,

    /// Spacing from the right edge.
    pub right: T,

    /// Spacing from the bottom edge.
    pub bottom: T,

    /// Spacing from the left edge.
    pub left: T,
}

impl Edges {
    pub const ZERO: Self = Self {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };
}

impl<T> Edges<T> {
    /// Create edges with individual values for each side.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Edges;
    /// let edges = Edges::new(1, 2, 3, 4);  // top, right, bottom, left
    /// assert_eq!(edges.top, 1);
    /// assert_eq!(edges.right, 2);
    /// ```
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Create edges with different horizontal and vertical spacing.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal spacing (left and right)
    /// * `y` - Vertical spacing (top and bottom)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Edges;
    /// let edges = Edges::sides(2, 1);  // 2 on left/right, 1 on top/bottom
    /// assert_eq!(edges.left, 2);
    /// assert_eq!(edges.right, 2);
    /// assert_eq!(edges.top, 1);
    /// assert_eq!(edges.bottom, 1);
    /// ```
    pub const fn sides(x: T, y: T) -> Self where T: Copy {
        Self {
            top: y,
            right: x,
            bottom: y,
            left: x,
        }
    }

    /// Create edges with the same spacing on all sides.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Edges;
    /// let edges = Edges::all(2);  // 2 on all sides
    /// assert_eq!(edges.horizontal(), 4);
    /// assert_eq!(edges.vertical(), 4);
    /// ```
    pub const fn all(n: T) -> Self where T: Copy {
        Self {
            top: n,
            right: n,
            bottom: n,
            left: n,
        }
    }

    /// Calculate total horizontal spacing (left + right).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Edges;
    /// let edges = Edges::new(1, 2, 1, 3);
    /// assert_eq!(edges.horizontal(), 5);  // 3 + 2
    /// ```
    pub fn horizontal(&self) -> T where T: Copy + Add<Output = T> {
        self.left + self.right
    }

    /// Calculate total vertical spacing (top + bottom).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Edges;
    /// let edges = Edges::new(2, 1, 3, 1);
    /// assert_eq!(edges.vertical(), 5);  // 2 + 3
    /// ```
    pub fn vertical(&self) -> T where T: Copy + Add<Output = T> {
        self.top + self.bottom
    }
}
