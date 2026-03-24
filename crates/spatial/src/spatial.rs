use geometry::{Rect, Bounded as GeometryBounded, Size};
use crate::{Area, Position, Steps, Sides};

/// Type that represents a spatial area.
pub const trait Spatial: [const] Sides {
    #[inline]
    fn min(&self) -> Position;

    #[inline]
    fn max(&self) -> Position;

    #[inline]
    fn width(&self) -> usize {
        self.max().col.saturating_sub(self.min().col)
    }

    #[inline]
    fn height(&self) -> usize {
        self.max().row.saturating_sub(self.min().row)
    }

    #[inline]
    fn len(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn size(&self) -> Size {
        Size::new(self.width(), self.height())
    }

    fn area(&self) -> Area {
        Area::new(self.min(), self.max())
    }

    fn positions(&self) -> Steps where Self: Sized {
        Steps::new(self)
    }
}

impl Spatial for Rect {
    fn min(&self) -> Position {
        Position::from(self.min)
    }

    fn max(&self) -> Position {
        Position::from(self.max)
    }

    fn area(&self) -> Area {
        Area::from(*self)
    }
}

