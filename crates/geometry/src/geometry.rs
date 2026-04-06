use crate::features::*;
use crate::{Axis, Column, Edges, Point, Rect, Row, Size, Zero, One, Min, Max, PointLike};


pub trait Geometry: Sized + Copy + Zero + One + Min + Max {}

impl Geometry for Rect {}
impl Geometry for Size {}
impl Geometry for Edges {}
impl Geometry for Axis {}

impl Geometry for Point {}
impl Geometry for Row {}
impl Geometry for Column {}
impl Geometry for PointLike {}

pub trait Bounds: Geometry + Bounded {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self;
}

impl Bounds for Rect {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Rect {
            min: Point { x, y },
            max: Point {
                x: x + width,
                y: y + height,
            },
        }
    }
}
impl Bounds for Size {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Size {
            width,
            height,
        }
    }
}
impl Bounds for Edges {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Edges {
            top: y,
            right: width,
            bottom: height,
            left: x,
        }
    }
}




pub trait Coordinate: Geometry + Coordinated {}

impl Coordinate for Point {}
impl Coordinate for Row {}
impl Coordinate for Column {}
impl Coordinate for PointLike {}