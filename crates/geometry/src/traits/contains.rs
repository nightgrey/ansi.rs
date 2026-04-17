use crate::{Bound, Location};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<B: Bound, P: Location> Contains<P> for B {
    fn contains(&self, rhs: &P) -> bool {
        rhs.x() >= self.min_x()
            && rhs.x() < self.max_x()
            && rhs.y() >= self.min_y()
            && rhs.y() < self.max_y()
    }
}
