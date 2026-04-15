use crate::{Point, PointLike, Position};

pub trait Coordinated: Into<Point> + From<Point> + Copy {
    fn new(x: u16, y: u16) -> Self;
    fn x(&self) -> u16;
    fn y(&self) -> u16;
}

impl Coordinated for Point {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
    fn x(&self) -> u16 {
        self.x
    }
    fn y(&self) -> u16 {
        self.y
    }
}

impl Coordinated for PointLike {
    fn new(x: u16, y: u16) -> Self {
        (x, y)
    }
    fn x(&self) -> u16 {
        self.0
    }
    fn y(&self) -> u16 {
        self.1
    }
}

impl Coordinated for Position {
    fn new(row: u16, col: u16) -> Self {
        Self::new(col as usize, row as usize)
    }
    
    fn x(&self) -> u16 {
        self.col as u16
    }
    
    fn y(&self) -> u16 {
        self.row as u16
    }
}

impl From<PointLike> for Point {
    fn from(value: PointLike) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Point> for PointLike {
    fn from(value: Point) -> Self {
        (value.x, value.y)
    }
}
