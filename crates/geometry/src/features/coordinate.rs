use crate::{Column, Point, PointLike, Rect, Row};

pub trait Coordinated {
    fn x(&self) -> usize;
    fn y(&self) -> usize;
}

impl Coordinated for Point {
    fn x(&self) -> usize {
        self.x
    }
    fn y(&self) -> usize {
        self.y
    }
}

impl Coordinated for PointLike {
    fn x(&self) -> usize {
        self.0
    }
    fn y(&self) -> usize {
        self.1
    }
}

impl Coordinated for Row {
    fn x(&self) -> usize {
        0
    }
    fn y(&self) -> usize {
        self.value()
    }
}

impl Coordinated for Column {
    fn x(&self) -> usize {
        self.value()
    }
    fn y(&self) -> usize {
        0
    }
}