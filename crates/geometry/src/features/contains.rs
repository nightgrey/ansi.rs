use crate::{Bounded, Point, PointLike, Rect, Size};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}
impl<T: Bounded> Contains<Point> for T {
    fn contains(&self, rhs: &Point) -> bool {
        rhs.x >= self.min_x()
            && rhs.x < self.max_x()
            && rhs.y >= self.min_y()
            && rhs.y < self.max_y()
    }
}

impl<T: Bounded> Contains<PointLike> for T {
    fn contains(&self, rhs: &PointLike) -> bool {
        rhs.0 >= self.min_x()
            && rhs.0 < self.max_x()
            && rhs.1 >= self.min_y()
            && rhs.1 < self.max_y()
    }
}
// impl<T: Bounded> Contains<Position> for T {
//     fn contains(&self, rhs: &Position) -> bool {
//         rhs.col >= self.min_x()
//             && rhs.col < self.max_x()
//             && rhs.row >= self.min_y()
//             && rhs.row < self.max_y()
//     }
// }

impl<T: Bounded, U: Bounded> Contains<U> for T {
    fn contains(&self, rhs: &U) -> bool {
        self.min_x() <= rhs.min_x()
            && self.max_x() >= rhs.max_x()
            && self.min_y() <= rhs.min_y()
            && self.max_y() >= rhs.max_y()
    }
}

impl Contains for Point {
    fn contains(&self, rhs: &Point) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }
}

impl Contains for PointLike {
    fn contains(&self, rhs: &(usize, usize)) -> bool {
        self.0 == rhs.0 && self.1 == rhs.1
    }
}
//
// impl Contains for Position {
//     fn contains(&self, rhs: &Position) -> bool {
//         self.col == rhs.col && self.row == rhs.row
//     }
// }
