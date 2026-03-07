use crate::{Contains, Point, Rect, Sides, Size};

pub trait Intersect<Rhs = Self> {
    type Output;

    fn intersect(&self, rhs: &Rhs) -> Option<Self::Output>;

    fn intersect_or(&self, rhs: &Rhs, default: Self::Output) -> Self::Output {
        self.intersect(rhs).unwrap_or(default)
    }

    fn intersect_or_default(&self, rhs: &Rhs) -> Self::Output
    where
        Self::Output: Default,
    {
        self.intersect(rhs).unwrap_or_default()
    }
}

impl Intersect<Point> for Rect {
    type Output = Point;

    fn intersect(&self, other: &Point) -> Option<Self::Output> {
        if self.contains(other) {
            Some(*other)
        } else {
            None
        }
    }
}

impl Intersect<Size> for Rect {
    type Output = Self;

    fn intersect(&self, other: &Size) -> Option<Self::Output> {
        self.intersect(&Rect::new(Point::ZERO, *other))
    }
}

impl Intersect for Rect {
    type Output = Self;

    fn intersect(&self, other: &Rect) -> Option<Self::Output> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if left < right && top < bottom {
            Some(Self::new((left, top), (right, bottom)))
        } else {
            None
        }
    }
}
