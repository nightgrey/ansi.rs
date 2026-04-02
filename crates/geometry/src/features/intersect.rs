use crate::{Bounded, Contains, Point,  Rect, Sides, Size};

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

impl Intersect<Size> for Rect {
    type Output = Self;

    fn intersect(&self, other: &Size) -> Self::Output {
        self.intersect(&Rect::new(
            Point::ZERO,
            Point::new(other.width, other.height),
        ))
    }
}

impl Intersect<Rect> for Rect {
    type Output = Self;

    fn intersect(&self, other: &Rect<Point>) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Rect::new(Point::ZERO, Point::ZERO);
        }

        let mut r = Rect::new(Point::ZERO, Point::ZERO);

        let x1 = self.min.x.max(other.min.x);
        let y1 = self.min.y.max(other.min.y);
        let x2 = self.max.x.min(other.max.x);
        let y2 = self.max.y.min(other.max.y);

        r.min.x = x1;
        r.min.y = y1;


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

        r.max.x = r.min.x + w;
        r.max.y = r.min.y + h;

        r
    }
}
