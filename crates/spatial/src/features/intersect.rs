use geometry::Size;
use crate::{Contains, Position, Area, Spatial};

pub trait Intersect<Rhs = Self> {
    type Output;

    fn intersect(&self, rhs: &Rhs) -> Self::Output;

    fn clip(&self, rhs: &Rhs) -> Rhs::Output where Rhs: Intersect<Self>, Self: Sized {
        rhs.intersect(self)
    }
}

impl Intersect<Position> for Area {
    type Output = Position;

    fn intersect(&self, other: &Position) -> Self::Output {
        if self.contains(other) {
            *other
        } else {
            Position::ZERO
        }
    }
}

impl Intersect<Size> for Area {
    type Output = Self;

    fn intersect(&self, other: &Size) -> Self::Output {
        self.intersect(&Area::new(Position::ZERO, Position::new(other.height, other.width)))
    }
}

impl<C: Spatial, Rhs: Spatial> Intersect<Rhs> for C {
    type Output = Area;

    fn intersect(&self, other: &Rhs) -> Self::Output {
        if self.width() == 0 || self.height() == 0 || other.width() == 0 || other.height() == 0 {
            return Area::ZERO;
        }

        let mut r = Area::ZERO;

        let x1 = self.min().col.max(other.min().col);
        let y1 = self.min().row.max(other.min().row);
        let x2 = self.max().col.min(other.max().col);
        let y2 = self.max().row.min(other.max().row);

        r.min.col = x1;
        r.min.row = y1;

        let mut w = x2 - x1;
        let mut h = y2 - y1;

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

        r.max.col = r.min.col + w;
        r.max.row = r.min.row + h;

        r
    }
}

