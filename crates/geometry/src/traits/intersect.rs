use crate::{Bound, Point, Rect};
use number::Zero;
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

impl<B: Bound, U: Bound> Intersect<U> for B {
    type Output = Rect;

    fn intersect(&self, other: &U) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Rect::ZERO;
        }

        let x1 = self.min_x().max(other.min_x());
        let y1 = self.min_y().max(other.min_y());
        let x2 = self.max_x().min(other.max_x());
        let y2 = self.max_y().min(other.max_y());

        let w = x2.saturating_sub(x1);
        let h = y2.saturating_sub(y1);

        Rect {
            min: Point { x: x1, y: y1 },
            max: Point { x: x1 + w, y: y1 + h },
        }
    }
}
