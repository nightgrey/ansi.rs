use crate::{Bounds, Column, Locatable, Point, PointLike, Position, PositionLike, Rect, Row, Size};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<B: Bounds, L: Locatable> Contains<L> for B {
    fn contains(&self, rhs: &L) -> bool {
        rhs.x() >= self.min_x()
            && rhs.x() <= self.max_x()
            && rhs.y() >= self.min_y()
            && rhs.y() <= self.max_y()
    }
}
