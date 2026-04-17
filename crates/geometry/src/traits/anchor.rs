use crate::{Point, PointLike, Position, PositionLike};

/// Anchor /// Represents a point on the screen./
/// Provides two dimensional screen-space coordinates with common functions./// It is used to represent a point on the screen.t
////// # Examples,
/// - `x` => left to right (column)
/// - `y` => top to bottom (row)
pub trait Anchor: Copy + PartialEq + Eq + PartialOrd + Ord {
    fn new(x: u16, y: u16) -> Self;

    fn x(&self) -> u16;
    fn y(&self) -> u16;

    fn set_x(&mut self, x: u16);
    fn set_y(&mut self, y: u16);
    #[inline]
    fn set(&mut self, x: u16, y: u16) {
        self.set_x(x);
        self.set_y(y);
    }

    #[inline]
    fn point(&self) -> Point {
        Point::new(self.x(), self.y())
    }

    #[inline]
    fn position(&self) -> Position {
        Position::new(self.y() as usize, self.x() as usize)
    }
}

impl Anchor for Point {
    fn x(&self) -> u16 { self.x }
    fn y(&self) -> u16 { self.y }
    fn set_x(&mut self, x: u16) { self.x = x; }
    fn set_y(&mut self, y: u16) { self.y = y; }
    fn new(x: u16, y: u16) -> Self { Point::new(x, y) }
}

impl Anchor for PointLike {
    fn x(&self) -> u16 { self.0 }
    fn y(&self) -> u16 { self.1 }
    fn set_x(&mut self, x: u16) { self.0 = x; }
    fn set_y(&mut self, y: u16) { self.1 = y; }
    fn new(x: u16, y: u16) -> Self { (x, y) }
}

impl Anchor for Position {
    fn x(&self) -> u16 { self.col as u16 }
    fn y(&self) -> u16 { self.row as u16 }
    fn set_x(&mut self, x: u16) { self.col = x as usize; }
    fn set_y(&mut self, y: u16) { self.row = y as usize; }
    fn new(x: u16, y: u16) -> Self { Position::new(y as usize, x as usize) }
}

impl Anchor for PositionLike {
    fn x(&self) -> u16 { self.1 as u16 }
    fn y(&self) -> u16 { self.0 as u16 }
    fn set_x(&mut self, x: u16) { self.1 = x as usize; }
    fn set_y(&mut self, y: u16) { self.0 = y as usize; }
    fn new(x: u16, y: u16) -> Self { (y as usize, x as usize) }
}
