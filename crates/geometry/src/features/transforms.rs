use crate::{Point, Rect};

pub trait Transform<T = Point>  {
    type Output;
    fn translate(self, by: T) -> Self::Output;
}

impl Transform<Point> for Rect<Point> {
    type Output = Self;

    fn translate(self, by: Point) -> Self::Output {
        Self { min: self.min + by, max: self.max + by }
    }
}