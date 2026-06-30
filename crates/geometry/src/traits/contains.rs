use crate::{Bounded, Point};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<B: Bounded> Contains<Point> for B {
    fn contains(&self, rhs: &Point) -> bool {
        rhs.x >= self.min_x()
            && rhs.x < self.max_x()
            && rhs.y >= self.min_y()
            && rhs.y < self.max_y()
    }
}
