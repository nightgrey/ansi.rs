use crate::{Area, Point, PointLike, Position, PositionLike, Rect, Size};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl Contains<PositionLike> for Area {
    fn contains(&self, rhs: &PositionLike) -> bool {
        rhs.0 >= self.min.col && rhs.0 < self.max.col && rhs.1 >= self.min.row && rhs.1 < self.max.row
    }
}

impl Contains<PointLike> for Rect {
    fn contains(&self, rhs: &PointLike) -> bool {
        rhs.0 >= self.min.x && rhs.0 < self.max.x && rhs.1 >= self.min.y && rhs.1 < self.max.y
    }
}

impl Contains<Position> for Area {
    fn contains(&self, rhs: &Position) -> bool {
        rhs.col >= self.min.col && rhs.col < self.max.col && rhs.row >= self.min.row && rhs.row < self.max.row
    }
}
impl Contains<Point> for Rect {
    fn contains(&self, rhs: &Point) -> bool {
        rhs.x >= self.min.x && rhs.x < self.max.x && rhs.y >= self.min.y && rhs.y < self.max.y
    }
}

impl Contains<Rect> for Rect {
    fn contains(&self, rhs: &Rect) -> bool {
        self.min.x <= rhs.min.x
            && self.max.x >= rhs.max.x
            && self.min.y <= rhs.min.y
            && self.max.y >= rhs.max.y
    }
}
impl Contains<Area> for Area {
    fn contains(&self, rhs: &Area) -> bool {
        self.min.col <= rhs.min.col
            && self.max.col >= rhs.max.col
            && self.min.row <= rhs.min.row
            && self.max.row >= rhs.max.row
    }
}

impl Contains for Point {
    fn contains(&self, rhs: &Point) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }
}

impl Contains for Position {
    fn contains(&self, rhs: &Position) -> bool {
        self.col == rhs.col && self.row == rhs.row
    }
}
impl Contains<Point> for Size {
    fn contains(&self, rhs: &Point) -> bool {
        0 <= rhs.x && rhs.x <= self.width && 0 <= rhs.y && rhs.y <= self.height
    }
}
impl Contains<Position> for Size {
    fn contains(&self, rhs: &Position) -> bool {
        0 <= rhs.col && rhs.col <= self.width && 0 <= rhs.row && rhs.row <= self.height
    }
}
impl Contains<Size> for Size {
    fn contains(&self, rhs: &Size) -> bool {
        rhs.width <= self.width && rhs.height <= self.height
    }
}
