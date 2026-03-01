use std::fmt::{Display, Formatter};
use crate::{Column, Point, Row};
use std::ops::{Add, AddAssign, Sub};

/// A position in buffer coordinates (row, column).
///
/// Unlike [`Point`] which uses (x, y) screen coordinates, `Position` uses
/// (row, col) buffer indexing where:
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

pub struct Position {
    /// Vertical position (row index, 0 = top).
    pub row: usize,

    /// Horizontal position (column index, 0 = left).
    pub col: usize,
}

/// Type alias for tuple-based positions: `(row, col)`.
///
/// Used for convenient position construction from tuples.
pub type PositionLike = (usize, usize);

impl Position {
    /// The origin position (0, 0).
    pub const ZERO: Self = Self::MIN;
    pub const ONE: Self = Self { row: 1, col: 1 };
    /// The minimum possible position (usize::MIN, usize::MIN).
    pub const MIN: Self = Self { row: usize::MIN, col: usize::MIN };
    /// The maximum possible position (usize::MAX, usize::MAX).
    pub const MAX: Self = Self { row: usize::MAX, col: usize::MAX };

    /// Create a new position at the given row and column.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Position;
    /// let pos = Position::new(10, 20);
    /// assert_eq!(pos.row, 10);
    /// assert_eq!(pos.col, 20);
    /// ```
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Create a new position at the given index inside a rectangular region.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Position;
    /// let pos = Position::from_index(10, 5);
    /// assert_eq!(pos.row, 2);
    /// assert_eq!(pos.col, 0);
    /// ```
    pub const fn from_index(index: usize, width: usize) -> Self {
        Self {
            row: index / width,
            col: index % width,
        }
    }

    /// Calculate Manhattan distance (L1 norm) from origin: `row + col`.
    ///
    /// This is the "taxicab" distance — the minimum number of horizontal and
    /// vertical moves needed to reach this position from the origin.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Position;
    /// let pos = Position::new(3, 4);
    /// assert_eq!(pos.manhattan(), 7);  // 3 + 4
    /// ```
    #[inline]
    pub fn manhattan(self) -> usize {
        self.row + self.col
    }

    /// Calculate Chebyshev distance (L∞ norm) from origin: `max(row, col)`.
    ///
    /// Also known as chessboard distance — the number of king moves needed
    /// to reach this position from the origin.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::Position;
    /// let pos = Position::new(3, 7);
    /// assert_eq!(pos.chebyshev(), 7);  // max(3, 7)
    /// ```
    #[inline]
    pub fn chebyshev(self) -> usize {
        let r = self.row;
        let c = self.col;
        if r > c { r } else { c }
    }

    /// Add two positions with overflow checking.
    ///
    /// Returns `None` if overflow would occur.
    fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            row: self.row.checked_add(rhs.row)?,
            col: self.col.checked_add(rhs.col)?,
        })
    }

    pub const  fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            row: self.row.saturating_sub(rhs.row),
            col: self.col.saturating_sub(rhs.col),
        }
    }

    /// Add two positions with saturating arithmetic.
    ///
    /// If overflow would occur, saturates at `usize::MAX`.
    fn saturating_add(self, rhs: Self) -> Self {
        Self {
            row: self.row.saturating_add(rhs.row),
            col: self.col.saturating_add(rhs.col),
        }
    }
}

impl From<PositionLike> for Position {
    fn from(value: PositionLike) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Point> for Position {
    fn from(value: Point) -> Self {
        Self::new(value.y, value.x)
    }
}

impl const From<Row> for Position {
    fn from(value: Row) -> Self {
        Self::new(value.0, 0)
    }
}

impl From<Column> for Position {
    fn from(value: Column) -> Self {
        Self::new(0, value.0)
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl AddAssign for Position {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<usize> for Position {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            row: self.row,
            col: self.col + rhs,
        }
    }
}

impl const PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl const Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.row.cmp(&other.row) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}
impl  Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.row, self.col)
    }
}