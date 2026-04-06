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
}

impl Bounds for Rect {

}
impl Bounds for Size {

}
impl Bounds for Edges {

}


pub trait Coordinate: Geometry + Coordinated {}

impl Coordinate for Point {}
impl Coordinate for Row {}
impl Coordinate for Column {}
impl Coordinate for PointLike {}