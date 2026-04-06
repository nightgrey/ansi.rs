use crate::{Bounded, Column, Coordinated, Point, PointLike, Rect, Row, Size};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<B: Bounded> Contains<Point> for B {
    fn contains(&self, rhs: &Point) -> bool {
        rhs.x() >= self.min_x()
            && rhs.x() < self.max_x()
            && rhs.y() >= self.min_y()
            && rhs.y() < self.max_y()
    }
}

impl<B: Bounded> Contains<PointLike> for B {
    fn contains(&self, rhs: &PointLike) -> bool {
        rhs.0 >= self.min_x()
            && rhs.0 < self.max_x()
            && rhs.1 >= self.min_y()
            && rhs.1 < self.max_y()
    }
}


impl<B: Bounded> Contains<Row> for B {
    fn contains(&self, rhs: &Row) -> bool {
        rhs.value() >= self.min_y()
            && rhs.value() < self.max_y()
    }
}

impl<B: Bounded> Contains<Column> for B {
    fn contains(&self, rhs: &Column) -> bool {
        rhs.value() >= self.min_x()
            && rhs.value() < self.max_x()
    }
}

impl<B: Bounded, U: Bounded> Contains<U> for B {
    fn contains(&self, rhs: &U) -> bool {
        rhs.min_x() >= self.min_x()
            && rhs.max_x() <= self.max_x()
            && rhs.min_y() >= self.min_y()
            && rhs.max_y() <= self.max_y()
    }
}
