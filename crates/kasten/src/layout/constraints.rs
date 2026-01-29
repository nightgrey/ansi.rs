use crate::Point;
use geometry::{Edges, Rect, Size};

/// Alignment along a single axis.
///
/// Used as part of [`Alignment`] to position content horizontally or vertically
/// within available space.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Align {
    /// Align to the start of the axis (left for horizontal, top for vertical).
    #[default]
    Start,

    /// Align to the center of the axis.
    Center,

    /// Align to the end of the axis (right for horizontal, bottom for vertical).
    End,
}

/// 2D alignment specification for horizontal and vertical axes.
///
/// Used with [`Node::Align`](crate::Node::Align) to position content within
/// available space.
///
/// # Example
///
/// ```rust
/// use kasten::{Alignment, Align, Node, Content};
///
/// // Center content both horizontally and vertically
/// let centered = Node::Align(
///     Alignment { x: Align::Center, y: Align::Center },
///     Box::new(Node::Base(Content::Text("Centered".into()))),
/// );
///
/// // Align to top-right
/// let top_right = Node::Align(
///     Alignment { x: Align::End, y: Align::Start },
///     Box::new(Node::Base(Content::Text("Top Right".into()))),
/// );
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Alignment {
    /// Horizontal alignment.
    pub x: Align,

    /// Vertical alignment.
    pub y: Align,
}

impl Alignment {
    /// Calculate the offset point to position `inner` within `outer` according to this alignment.
    ///
    /// Returns a [`Point`] offset that should be added to the outer bounds' min point
    /// to get the inner bounds' min point.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Alignment, Align, Size, Point};
    /// let alignment = Alignment { x: Align::Center, y: Align::Center };
    /// let outer = Size::new(20, 10);
    /// let inner = Size::new(10, 5);
    ///
    /// let offset = alignment.offset(outer, inner);
    /// assert_eq!(offset, Point::new(5, 2));  // Centered offset
    /// ```
    pub fn offset(&self, outer: Size, inner: Size) -> Point {
        Point {
            x: match self.x {
                Align::Start => 0,
                Align::Center => outer.width.saturating_sub(inner.width) / 2,
                Align::End => outer.width.saturating_sub(inner.width),
            },
            y: match self.y {
                Align::Start => 0,
                Align::Center => outer.height.saturating_sub(inner.height) / 2,
                Align::End => outer.height.saturating_sub(inner.height),
            },
        }
    }
}

/// A size constraint for a single dimension (width or height).
///
/// Constraints control how nodes size themselves during the measure and layout phases.
/// They can specify minimum sizes, maximum sizes, exact sizes, or flexible sizing.
///
/// # Variants
///
/// - **Auto**: Use the node's natural size
/// - **Min(n)**: At least `n` units
/// - **Max(n)**: At most `n` units
/// - **Fixed(n)**: Exactly `n` units
/// - **Between(min, max)**: Within the range `[min, max]`
/// - **Fill**: Expand to fill available space
///
/// # Example
///
/// ```rust
/// use kasten::{Constraint, Constraints};
///
/// // Fixed width, flexible height up to 20
/// let constraints = Constraints::new(
///     Constraint::Fixed(40),
///     Constraint::Max(20),
/// );
///
/// // Clamp a value to the constraint
/// let width = Constraint::Fixed(40);
/// assert_eq!(width.clamp(100), 40);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Constraint {
    /// Use the natural size of the content.
    ///
    /// The node measures itself based on its content without external constraints.
    #[default]
    Auto,

    /// Minimum size constraint.
    ///
    /// The node must be at least this many units, but can be larger.
    Min(usize),

    /// Maximum size constraint.
    ///
    /// The node must be at most this many units, but can be smaller.
    Max(usize),

    /// Fixed size constraint.
    ///
    /// The node must be exactly this many units.
    Fixed(usize),

    /// Range constraint.
    ///
    /// The node must be within the range `[min, max]`.
    Between(usize, usize),

    /// Fill available space.
    ///
    /// The node expands to use all available space provided by its parent.
    Fill,
}

impl Constraint {
    /// Clamp a value to satisfy this constraint.
    ///
    /// Returns a value that satisfies the constraint's bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Max(10).clamp(15), 10);
    /// assert_eq!(Constraint::Min(10).clamp(5), 10);
    /// assert_eq!(Constraint::Fixed(10).clamp(100), 10);
    /// assert_eq!(Constraint::Between(5, 15).clamp(3), 5);
    /// assert_eq!(Constraint::Auto.clamp(42), 42);
    /// ```
    pub fn clamp(&self, value: usize) -> usize {
        match self {
            Self::Auto => value,
            &Self::Min(min) => min.max(value),
            &Self::Max(max) => max.min(value),
            &Self::Fixed(fixed) => fixed.min(value),
            &Self::Between(min, max) => min.max(value).min(max),
            Self::Fill => value,
        }
    }

    /// Get the minimum value for this constraint, if any.
    ///
    /// Returns `Some(n)` for Min, Between, and Fixed constraints, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Min(10).min(), Some(10));
    /// assert_eq!(Constraint::Between(5, 15).min(), Some(5));
    /// assert_eq!(Constraint::Max(10).min(), None);
    /// ```
    pub fn min(&self) -> Option<usize> {
        match *self {
            Self::Min(min) | Self::Between(min, ..) | Self::Fixed(min) => Some(min),
            _ => None,
        }
    }

    /// Get the minimum value for this constraint, or a default if none.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Min(10).min_or(0), 10);
    /// assert_eq!(Constraint::Auto.min_or(5), 5);
    /// ```
    pub fn min_or(&self, default: usize) -> usize {
        self.min().unwrap_or(default)
    }

    /// Get the maximum value for this constraint, if any.
    ///
    /// Returns `Some(n)` for Max, Between, and Fixed constraints, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Max(10).max(), Some(10));
    /// assert_eq!(Constraint::Between(5, 15).max(), Some(15));
    /// assert_eq!(Constraint::Min(10).max(), None);
    /// ```
    pub fn max(&self) -> Option<usize> {
        match *self {
            Self::Max(max) | Self::Between(_, max) | Self::Fixed(max) => Some(max),
            _ => None,
        }
    }

    /// Get the maximum value for this constraint, or a default if none.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Max(10).max_or(0), 10);
    /// assert_eq!(Constraint::Auto.max_or(100), 100);
    /// ```
    pub fn max_or(&self, default: usize) -> usize {
        self.max().unwrap_or(default)
    }

    /// Get the fixed value for this constraint, if it is Fixed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Fixed(10).fixed(), Some(10));
    /// assert_eq!(Constraint::Max(10).fixed(), None);
    /// ```
    pub fn fixed(&self) -> Option<usize> {
        match self {
            Self::Fixed(fixed) => Some(*fixed),
            _ => None,
        }
    }

    /// Merge this constraint with another, resolving conflicts.
    ///
    /// This is used during layout to compose constraints from different sources:
    ///
    /// - If `self` is `Auto`, inherit `other`
    /// - If `self` is `Fill`, expand to `other`'s maximum
    /// - Otherwise, use `self`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// let parent = Constraint::Max(100);
    /// let child = Constraint::Min(20);
    /// let result = child.constrain(parent);
    /// assert_eq!(result, Constraint::Min(20));  // Child's constraint wins
    ///
    /// let auto = Constraint::Auto;
    /// let result = auto.constrain(parent);
    /// assert_eq!(result, Constraint::Max(100));  // Inherits parent
    /// ```
    pub fn constrain(&self, other: Constraint) -> Constraint {
        match (*self, other) {
            // self is Auto → inherit other
            (Self::Auto, other) => other,
            // self is Fill → expand to other's max
            (Self::Fill, other) => Constraint::Fixed(other.max_or(usize::MAX)),
            // self specifies → use self, but clamp to other bounds
            (constraint, _) => constraint,
        }
    }

    /// Shrink the constraint by a given amount.
    ///
    /// For numeric constraints (Min, Max, Fixed, Between), subtracts the amount
    /// using saturating subtraction. Auto and Fill are unchanged.
    ///
    /// This is used when applying padding or margins to reduce available space.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraint;
    /// assert_eq!(Constraint::Fixed(20).shrink(5), Constraint::Fixed(15));
    /// assert_eq!(Constraint::Max(10).shrink(3), Constraint::Max(7));
    /// assert_eq!(Constraint::Auto.shrink(10), Constraint::Auto);
    /// ```
    pub fn shrink(&self, amount: usize) -> Self {
        match *self {
            Self::Auto | Self::Fill => *self,
            Self::Min(n) => Self::Min(n.saturating_sub(amount)),
            Self::Max(n) => Self::Max(n.saturating_sub(amount)),
            Self::Fixed(n) => Self::Fixed(n.saturating_sub(amount)),
            Self::Between(min, max) => {
                Self::Between(min.saturating_sub(amount), max.saturating_sub(amount))
            }
        }
    }
}

/// 2D size constraints for width and height.
///
/// Combines two [`Constraint`] values to control sizing in both dimensions.
/// Commonly used throughout the layout system to propagate sizing requirements.
///
/// # Constructor Convenience
///
/// The `Fixed`, `Max`, `Min`, and `Auto` methods use PascalCase (like type constructors)
/// for consistency with other constructor-style methods in Rust.
///
/// # Example
///
/// ```rust
/// use kasten::{Constraints, Constraint};
///
/// // Fixed size
/// let fixed = Constraints::Fixed(80, 24);
///
/// // Max constraints
/// let max = Constraints::Max(100, 50);
///
/// // Mixed constraints
/// let mixed = Constraints::new(
///     Constraint::Fixed(40),
///     Constraint::Max(20),
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Constraints {
    /// Width constraint.
    pub width: Constraint,

    /// Height constraint.
    pub height: Constraint,
}

#[allow(non_snake_case)]
impl Constraints {
    /// Create constraints with fixed width and height.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraints;
    /// let constraints = Constraints::Fixed(80, 24);
    /// ```
    pub const fn Fixed(width: usize, height: usize) -> Self {
        Self::new(Constraint::Fixed(width), Constraint::Fixed(height))
    }

    /// Create constraints with maximum width and height.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraints;
    /// let constraints = Constraints::Max(100, 50);
    /// ```
    pub const fn Max(width: usize, height: usize) -> Self {
        Self::new(Constraint::Max(width), Constraint::Max(height))
    }

    /// Create constraints with minimum width and height.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraints;
    /// let constraints = Constraints::Min(20, 10);
    /// ```
    pub const fn Min(width: usize, height: usize) -> Self {
        Self::new(Constraint::Min(width), Constraint::Min(height))
    }

    /// Create Auto constraints (natural sizing).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraints;
    /// let constraints = Constraints::Auto();
    /// ```
    pub const fn Auto() -> Self {
        Self::new(Constraint::Auto, Constraint::Auto)
    }

    /// Create constraints with individual width and height constraints.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Constraints, Constraint};
    /// let constraints = Constraints::new(
    ///     Constraint::Fixed(40),
    ///     Constraint::Max(20),
    /// );
    /// ```
    pub const fn new(width: Constraint, height: Constraint) -> Self {
        Self { width, height }
    }

    /// Clamp width and height values to satisfy these constraints.
    ///
    /// Returns a [`Size`] with both dimensions clamped.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Constraints, Size};
    /// let constraints = Constraints::Max(10, 10);
    /// let size = constraints.clamp(15, 20);
    /// assert_eq!(size, Size::new(10, 10));
    /// ```
    pub fn clamp(&self, width: usize, height: usize) -> Size {
        Size {
            width: self.width.clamp(width),
            height: self.height.clamp(height),
        }
    }

    /// Merge these constraints with another set, resolving conflicts.
    ///
    /// Delegates to [`Constraint::constrain`] for each dimension.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::Constraints;
    /// let parent = Constraints::Max(100, 50);
    /// let child = Constraints::Fixed(40, 20);
    /// let result = child.constrain(parent);
    /// assert_eq!(result, Constraints::Fixed(40, 20));
    /// ```
    pub fn constrain(&self, other: Constraints) -> Constraints {
        Self {
            width: self.width.constrain(other.width),
            height: self.height.constrain(other.height),
        }
    }

    /// Shrink constraints by edge insets (for padding/margins).
    ///
    /// Reduces width by horizontal edges and height by vertical edges.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Constraints, Edges};
    /// let constraints = Constraints::Fixed(20, 10);
    /// let edges = Edges::all(2);
    /// let shrunk = constraints.shrink(&edges);
    /// assert_eq!(shrunk, Constraints::Fixed(16, 6));
    /// ```
    pub fn shrink(&self, insets: &Edges) -> Self {
        Self {
            width: self.width.shrink(insets.horizontal()),
            height: self.height.shrink(insets.vertical()),
        }
    }

    /// Get the minimum size from these constraints.
    ///
    /// Returns a [`Size`] with the minimum width and height (or 0 if no minimum).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Constraints, Size};
    /// let constraints = Constraints::new(
    ///     kasten::Constraint::Min(10),
    ///     kasten::Constraint::Between(5, 20),
    /// );
    /// assert_eq!(constraints.min(), Size::new(10, 5));
    /// ```
    pub fn min(&self) -> Size {
        Size {
            width: self.width.min_or(0),
            height: self.height.min_or(0),
        }
    }

    /// Get the maximum size from these constraints.
    ///
    /// Returns a [`Size`] with the maximum width and height (or 0 if no maximum).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Constraints, Size};
    /// let constraints = Constraints::Max(100, 50);
    /// assert_eq!(constraints.max(), Size::new(100, 50));
    /// ```
    pub fn max(&self) -> Size {
        Size {
            width: self.width.max_or(0),
            height: self.height.max_or(0),
        }
    }
}

impl From<Rect> for Constraints {
    /// Convert a [`Rect`] to fixed constraints matching its dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Rect, Constraints};
    /// let rect = Rect::new((0, 0), (80, 24));
    /// let constraints = Constraints::from(rect);
    /// assert_eq!(constraints, Constraints::Fixed(80, 24));
    /// ```
    fn from(value: Rect) -> Self {
        Self::Fixed(value.width(), value.height())
    }
}
