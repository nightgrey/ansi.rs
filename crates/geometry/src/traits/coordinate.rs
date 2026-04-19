use crate::{Point, PointLike, Position, PositionLike};
use std::ops::Deref;

/// Two-dimensional screen-space coordinate
///
/// - `x` => left to right (column)
/// - `y` => top to bottom (row)
pub trait Coordinate: Copy + PartialEq + Eq + PartialOrd + Ord {
    fn new(x: u16, y: u16) -> Self;

    #[inline]
    fn x(&self) -> u16;
    #[inline]
    fn y(&self) -> u16;

    #[inline]
    fn set_x(&mut self, x: u16);
    #[inline]
    fn set_y(&mut self, y: u16);

    #[inline]
    fn set(&mut self, x: u16, y: u16) {
        self.set_x(x);
        self.set_y(y);
    }
}

impl Coordinate for Point {
    fn new(x: u16, y: u16) -> Self {
        Point::new(x, y)
    }
    fn x(&self) -> u16 {
        self.x
    }
    fn y(&self) -> u16 {
        self.y
    }
    fn set_x(&mut self, x: u16) {
        self.x = x;
    }
    fn set_y(&mut self, y: u16) {
        self.y = y;
    }
}

impl Coordinate for PointLike {
    fn new(x: u16, y: u16) -> Self {
        (x, y)
    }
    fn x(&self) -> u16 {
        self.0
    }
    fn y(&self) -> u16 {
        self.1
    }
    fn set_x(&mut self, x: u16) {
        self.0 = x;
    }
    fn set_y(&mut self, y: u16) {
        self.1 = y;
    }
}

impl Coordinate for Position {
    fn new(x: u16, y: u16) -> Self {
        Position::new(y as usize, x as usize)
    }
    fn x(&self) -> u16 {
        self.col as u16
    }
    fn y(&self) -> u16 {
        self.row as u16
    }
    fn set_x(&mut self, x: u16) {
        self.col = x as usize;
    }
    fn set_y(&mut self, y: u16) {
        self.row = y as usize;
    }
}

impl Coordinate for PositionLike {
    fn new(x: u16, y: u16) -> Self {
        (y as usize, x as usize)
    }
    fn x(&self) -> u16 {
        self.1 as u16
    }
    fn y(&self) -> u16 {
        self.0 as u16
    }
    fn set_x(&mut self, x: u16) {
        self.1 = x as usize;
    }
    fn set_y(&mut self, y: u16) {
        self.0 = y as usize;
    }
}
