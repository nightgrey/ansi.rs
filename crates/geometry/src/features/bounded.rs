use crate::{Edges, Point, Rect, Size};
/// Provides the bounds of a geometry.
pub trait Bounded {
    #[inline]
    fn min_x(&self) -> usize;

    #[inline]
    fn min_y(&self) -> usize;

    #[inline]
    fn max_x(&self) -> usize;

    #[inline]
    fn max_y(&self) -> usize;

    #[inline]
    fn min(&self) -> Point {
        Point { x: self.min_x(), y: self.min_y() }
    }

    #[inline]
    fn max(&self) -> Point {
        Point { x: self.max_x(), y: self.max_y() }
    }

    #[inline]
    fn x(&self) -> usize {
        self.min_x()
    }

    #[inline]
    fn y(&self) -> usize {
        self.min_y()
    }

    #[inline]
    fn width(&self) -> usize {
        self.max_x().saturating_sub(self.min_x())
    }

    #[inline]
    fn height(&self) -> usize {
        self.max_y().saturating_sub(self.min_y())
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
    fn bounds(&self) -> Rect {
        Rect {
            min: self.min(),
            max: self.max(),
        }
    }
    
    #[inline]
    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}

impl Bounded for Rect {
    fn min_x(&self) -> usize {
        self.min.x
    }

    fn min_y(&self) -> usize {
        self.min.y
    }

    fn max_x(&self) -> usize {
        self.max.x
    }

    fn max_y(&self) -> usize {
        self.max.y
    }

    fn min(&self) -> Point {
        self.min
    }

    fn max(&self) -> Point {
        self.max
    }

    fn bounds(&self) -> Rect {
        *self
    }

}
impl Bounded for Size {
    fn min_x(&self) -> usize {
        0
    }

    fn min_y(&self) -> usize {
        0
    }

    fn max_x(&self) -> usize {
        self.width
    }

    fn max_y(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl Bounded for Edges {
    fn min_x(&self) -> usize {
        0
    }

    fn min_y(&self) -> usize {
        0
    }

    fn max_x(&self) -> usize {
        self.horizontal()
    }

    fn max_y(&self) -> usize {
        self.vertical()
    }
}