use crate::{AssignOps, Column, One, Ops, Point, Rect, Row, SaturatingAdd, SaturatingOps, SaturatingSub, Size, Zero};
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Type alias for tuple-based positions: `(row, col)`.
///
/// Used for convenient position construction from tuples.
pub type PositionLike<T = usize> = (T, T);

impl From<PositionLike> for Position {
    fn from(value: PositionLike) -> Self {
        Self::new(value.0, value.1)
    }
}

/// A position in row/column coordinates.
///
/// Unlike [`Point`] which uses (x, y) screen coordinates,
/// `Position` uses row and column:
///
/// - `row` is the vertical position (0 = top)
/// - `col` is the horizontal position (0 = left)
///
/// This matches typical buffer and array indexing conventions.
///
/// # Coordinate System Difference
///
/// - `Point`: (x, y) where x=column, y=row
/// - `Position`: (row, col) where row=y, col=x
///
/// These can be converted between each other, but note the field order difference.
///
/// # Example
///
/// ```rust
/// use geometry::Position;
///
/// let pos = Position::new(5, 10);  // row 5, column 10
/// assert_eq!(pos.row, 5);
/// assert_eq!(pos.col, 10);
///
/// let manhattan = pos.manhattan();  // Distance from origin
/// assert_eq!(manhattan, 15);
/// ```
#[derive(Copy, Debug)]
#[derive_const(Clone, Default, PartialEq, Eq)]
pub struct Position<T = usize> {
    /// Vertical position (row index, 0 = top).
    pub row: T,

    /// Horizontal position (column index, 0 = left).
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

impl From<Position> for Point {
    fn from(value: Position) -> Self {
        Point::new(value.col as u16, value.row as u16)
    }
}

impl From<Point> for Position {
    fn from(value: Point) -> Self {
        Self::new(value.y as usize, value.x as usize)
    }
}


/// An axis-aligned rectangle for buffer-space coordinates.
///
/// Areas are represented as half-open ranges: `[min, max)`.
/// The `min` position is inclusive, the `max` position is exclusive.
pub type Area = Rect<Position>;

