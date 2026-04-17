use crate::{Sides, Column, Edges, Point, PointLike, Rect, Row, Size};
use number::{Zero, One, Min, Max, Ops, AssignOps, SaturatingOps, SaturatingAdd, SaturatingSub};

use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Type alias for tuple-based positions: `(row, col)`.
///
/// Used for convenient position construction from tuples.
pub type PositionLike<T = usize> = (T, T);

impl<T> From<PositionLike<T>> for Position<T> {
    fn from(value: PositionLike<T>) -> Self {
        Self::new(value.0, value.1)
    }
}

/// A position in index coordinates.
///
/// Unlike [`Point`] which uses (x, y) screen coordinates,
/// `Position` uses row and column:
///
/// - `row` is the vertical position (0 = top)
/// - `col` is the horizontal position (0 = left)
///
/// This matches typical indexing conventions.
///
/// ## Interopability
/// [`Point`] and [`Position`] can be converted between each other, but note the field order difference if doing so manually.
///
/// ```rust
/// # use geometry::{Point, Position};
/// let point = Point::new(10, 5);
/// let position = Position::new(5, 10);
///
/// // Point => `Position { row: point.y, col: point.x }`
/// assert_eq!((position.row, position.col), (point.y, point.x));
///
/// // Position => Point { x: position.col, y: position.row }
/// assert_eq!((point.x, point.y), (position.col, position.row));
/// ```
///
/// You can also use the `From` and `Into` traits to convert between them.
///
/// # Example
///
/// ```rust
/// # use geometry::{Point, Position};
///
/// let position = Position::new(5, 10);
/// assert_eq!(position.row, 5);
/// assert_eq!(position.col, 10);
/// ```
#[derive(Copy, Debug)]
#[derive_const(Clone, Default, PartialEq, Eq)]
pub struct Position<T = usize> {
    /// Vertical position
    pub row: T,

    /// Horizontal position
    pub col: T,
}

impl<T> Position<T> {
    /// Create a new point at the given coordinates.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Position;
    /// let p = Position::new(5, 10);
    /// assert_eq!(p.row, 5);
    /// assert_eq!(p.col, 10);
    /// ```
    pub const fn new(row: T, col: T) -> Self {
        Self { row, col }
    }
}

impl<T: One> Position<T> {
    pub const ONE: Self = Position { row: T::ONE, col: T::ONE };
}

impl<T: Zero> Position<T> {
    pub const ZERO: Self = Position { row: T::ZERO, col: T::ZERO };
}

impl<T: Ops> Add for Position<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl<T: AssignOps> AddAssign for Position<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.row += rhs.row;
        self.col += rhs.col;
    }
}

impl<T: Ops> Sub for Position<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            row: self.row - rhs.row,
            col: self.col - rhs.col,
        }
    }
}

impl<T: AssignOps> SubAssign for Position<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.row -= rhs.row;
        self.col -= rhs.col;
    }
}

impl<T: Ops + SaturatingOps> SaturatingAdd for Position<T> {
    fn saturating_add(self, rhs: Self) -> Self {
        Self {
            row: self.row.saturating_add(rhs.row),
            col: self.col.saturating_add(rhs.col),
        }
    }
}
impl<T: Ops + SaturatingOps> SaturatingSub for Position<T> {
    fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            row: self.row.saturating_sub(rhs.row),
            col: self.col.saturating_sub(rhs.col),
        }
    }
}

impl<T: Ord> PartialOrd for Position<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Position<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.row.cmp(&other.row) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}

impl<T: Display> Display for Position<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}x{}]", self.row, self.col)
    }
}

impl From<Position> for Row {
    fn from(value: Position) -> Self {
        Self(value.row)
    }
}

impl From<Row> for Position {
    fn from(value: Row) -> Self {
        Self::new(value.0, 0)
    }
}

impl From<Position> for Column {
    fn from(value: Position) -> Self {
        Self(value.col)
    }
}

impl From<Column> for Position {
    fn from(value: Column) -> Self {
        Self::new(0, value.0)
    }
}

/// An axis-aligned rectangle for buffer-space coordinates.
///
/// Areas are represented as half-open ranges: `[min, max)`.
/// The `min` position is inclusive, the `max` position is exclusive.
pub type Area = Rect<Position>;

