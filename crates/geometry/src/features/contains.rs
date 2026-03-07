use crate::{Point, Rect, Size};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
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

impl Contains for Point {
    fn contains(&self, rhs: &Point) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }
}

impl Contains<Point> for Size {
    fn contains(&self, rhs: &Point) -> bool {
        0 <= rhs.x && rhs.x <= self.width && 0 <= rhs.y && rhs.y <= self.height
    }
}

impl Contains<Size> for Size {
    fn contains(&self, rhs: &Size) -> bool {
        rhs.width <= self.width && rhs.height <= self.height
    }
}

/// Tests if a geometry is completely within another geometry.
pub trait Within<Rhs = Self> {
    fn is_within(&self, rhs: &Rhs) -> bool;
}

impl<A, B> Within<B> for A
where
    B: Contains<A>,
{
    fn is_within(&self, b: &B) -> bool {
        b.contains(self)
    }
}
