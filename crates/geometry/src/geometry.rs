use crate::features::*;
use crate::{Sides, Column, Edges, Point, Rect, Row, Size, Zero, One, Min, Max, PointLike};


pub trait Geometry: Sized + Copy + Zero + One + Min + Max {}
pub trait Bounds: Geometry + Bounded {}
pub trait Coordinate: Geometry + Coordinated {}

impl Geometry for Rect {}
impl Geometry for Size {}
impl Geometry for Edges {}
impl Geometry for Sides {}

impl Geometry for Point {}
impl Geometry for Row {}
impl Geometry for Column {}
impl Geometry for PointLike {}

impl Bounds for Rect {}
impl Bounds for Size {}
impl Bounds for Edges {}

impl Coordinate for Point {}
impl Coordinate for PointLike {}