use std::ops::{Add, AddAssign};
use crate::position::Position;

pub type PointLike = (usize, usize);

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<PointLike> for Point {
    fn from(value: PointLike) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Position> for Point {
    fn from(value: Position) -> Self {
        Self::new(value.col, value.row)
    }
}


impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Point {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize
}

impl Size {
    pub const ZERO: Self = Self { width: 0, height: 0 };

    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}


#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub const ZERO: Self = Self { min: Point::ZERO, max: Point::ZERO };

    pub  fn new(min: impl Into<Point>, max: impl Into<Point>) -> Self {
        Self { min: min.into(), max: max.into() }
    }

    pub const fn x(&self) -> usize {
        self.min.x
    }

    pub const fn y(&self) -> usize {
        self.min.y
    }

    pub const fn width(&self) -> usize {
        self.max.x.saturating_sub(self.min.x)
    }

    pub const fn height(&self) -> usize {
        self.max.y.saturating_sub(self.min.y)
    }

    pub const fn area(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    pub const fn contains(&self, point: &Point) -> bool {
        self.min.x <= point.x && point.x < self.max.x
            && self.min.y <= point.y && point.y < self.max.y
    }

    pub const fn size(&self) -> Size {
        Size {
            width: self.max.x.saturating_sub(self.min.x),
            height: self.max.y.saturating_sub(self.min.y),
        }
    }
}
