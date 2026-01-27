use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, Sub};
use crate::{Point,  Rect, Size};
use std::ops::Range;

pub type PositionLike = (usize, usize);

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub const ZERO: Self = Self { row: 0, col: 0 };
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Manhattan distance (L1 norm): `|row| + |col|`
    #[inline]
    pub fn manhattan(self) -> usize {
        self.row + self.col
    }

    /// Chebyshev distance (L∞ norm): `max(|row|, |col|)`
    ///
    /// Also known as chessboard distance — the number of king moves.
    #[inline]
    pub fn chebyshev(self) -> usize {
        let r = self.row;
        let c = self.col;
        if r > c { r } else { c }
    }

    fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            row: self.row.checked_add(rhs.row)?,
            col: self.col.checked_add(rhs.col)?,
        })
    }

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

#[derive(Debug)]
pub struct Region {
    pub min: Position,
    pub max: Position,
}

impl Region {
    pub const ZERO: Self = Self { min: Position::ZERO, max: Position::ZERO };

    pub fn new(min: Position, max: Position) -> Self {
        Self { min, max }
    }

    pub const fn min(&self) -> Position {
        self.min
    }

    pub const fn max(&self) -> Position {
        self.max
    }

    pub const fn x(&self) -> usize {
        self.min.col
    }

    pub const fn y(&self) -> usize {
        self.min.row
    }

    pub const fn width(&self) -> usize {
        self.max.col.saturating_sub(self.min.col)
    }

    pub const fn height(&self) -> usize {
        self.max.row.saturating_sub(self.min.row)
    }

    pub const fn area(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    pub const fn contains(&self, point: &Position) -> bool {
        self.min.col <= point.col && point.col < self.max.col
            && self.min.row <= point.row && point.row < self.max.row
    }

    pub const fn size(&self) -> Size {
        Size {
            width: self.max.col.saturating_sub(self.min.col),
            height: self.max.row.saturating_sub(self.min.row),
        }
    }

    pub const fn iter(&self) -> RegionIter {
        RegionIter::new(self)
    }
}

impl From<Rect> for Region {
    fn from(value: Rect) -> Self {
        Self::new(Position::from(value.min), Position::from(value.max))
    }
}

impl From<Range<Position>> for Region {
    fn from(value: Range<Position>) -> Self {
        Self::new(value.start, value.end)
    }
}

impl IntoIterator for Region {
    type Item = Position;
    type IntoIter = RegionIter;

    fn into_iter(self) -> Self::IntoIter {
        RegionIter::new(&self)
    }
}

impl IntoIterator for &Region {
    type Item = Position;
    type IntoIter = RegionIter;

    fn into_iter(self) -> Self::IntoIter {
        RegionIter::new(self)
    }
}

/// Iterator over positions in a region, row-by-row.
#[derive(Clone, Debug)]
pub struct RegionIter {
    // Region bounds (immutable)
    pub row: Range<usize>,
    pub col: Range<usize>,

    start: usize,
    end: usize,
}

impl RegionIter {
    #[inline]
    const fn new(region: &Region) -> Self {
        let width = region.width();
        let height = region.height();

        let row = region.min.row..region.max.row;
        let col = region.min.col..region.max.col;

        Self {
            row,
            col,
            start: 0,
            end: height * width,
        }
    }

    fn to_position(&self, index: usize) -> Position {
        let width = (self.col.end - self.col.start);
        if width == 0 {
            return Position::ZERO;
        }

        Position {
            row: self.row.start + index / width,
            col: self.col.start + index % width,
        }
    }
}


impl Iterator for RegionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let coord = self.to_position(self.start);
        self.start += 1;
        Some(coord)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.start = self.start.saturating_add(n);
        self.next()
    }
}

impl DoubleEndedIterator for RegionIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        self.end -= 1;
        Some(self.to_position(self.end))
    }
}

impl ExactSizeIterator for RegionIter {
    fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}
impl FusedIterator for RegionIter {}
