use crate::{Bounded, Bounds, Contains, Point, Rect, Sides, Size, Zero};

pub trait Intersect<Rhs = Self> {
    type Output;

    fn intersect(&self, rhs: &Rhs) -> Self::Output;

    fn clip(&self, rhs: &Rhs) -> Rhs::Output
    where
        Rhs: Intersect<Self>,
        Self: Sized,
    {
        rhs.intersect(self)
    }
}

impl<T: Bounded> Intersect<Point> for T {
    type Output = Point;

    fn intersect(&self, other: &Point) -> Self::Output {
        if self.contains(other) {
            *other
        } else {
            Point::ZERO
        }
    }
}
impl<T: Bounded, U: Bounded> Intersect<T> for U {
    type Output = Rect;

    fn intersect(&self, other: &T) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Self::Output::ZERO;
        }

        let x1 = self.min_x().max(other.min_x());
        let y1 = self.min_y().max(other.min_y());
        let x2 = self.max_x().min(other.max_x());
        let y2 = self.max_y().min(other.max_y());

        let mut w = x2.saturating_sub(x1);
        let mut h = y2.saturating_sub(y1);

        if w < 0 {
            w = 0;
        }

        if h < 0 {
            h = 0;
        }

        if w > usize::MAX {
            w = usize::MAX;
        }

        if h > usize::MAX {
            h = usize::MAX;
        }

        Rect {
            min: Point { x: x1, y: y1 },
            max: Point { x: x1 + w, y: y1 + h },
        }
    }
}
