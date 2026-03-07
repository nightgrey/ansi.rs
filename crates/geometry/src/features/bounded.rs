use crate::{Point, Rect, Size};
/// Provides the bounds of a geometry.
pub trait Bounded {
    fn bounds(&self) -> Rect;

    fn min(&self) -> Point;

    fn max(&self) -> Point;

    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

impl Bounded for Rect {
    fn bounds(&self) -> Rect {
        *self
    }

    fn min(&self) -> Point {
        self.min
    }

    fn max(&self) -> Point {
        self.max
    }

    fn width(&self) -> usize {
        self.max.x.saturating_sub(self.min.x)
    }

    fn height(&self) -> usize {
        self.max.y.saturating_sub(self.min.y)
    }
    
    fn len(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    fn is_empty(&self) -> bool {
        self.min == self.max
    }
}
impl Bounded for Size {
    fn bounds(&self) -> Rect {
        Rect::new(Point::ZERO, Point::new(self.width, self.height))
    }

    fn min(&self) -> Point {
        Point::ZERO
    }

    fn max(&self) -> Point {
        Point::new(self.width, self.height)
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn len(&self) -> usize {
        self.width.saturating_mul(self.height)
    }

    fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}
