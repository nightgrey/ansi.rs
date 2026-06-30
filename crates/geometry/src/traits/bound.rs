use crate::{Point, Rect, Resolve, Size, Steps};

/// A geometry with an axis-aligned min/max bounding rectangle in half-open
/// `[min, max)` coordinates.
pub trait Bounded {
    fn min_x(&self) -> u16;
    fn min_y(&self) -> u16;
    fn max_x(&self) -> u16;
    fn max_y(&self) -> u16;

    #[inline]
    fn min(&self) -> Point {
        Point::new(self.min_x(), self.min_y())
    }

    #[inline]
    fn max(&self) -> Point {
        Point::new(self.max_x(), self.max_y())
    }

    #[inline]
    fn width(&self) -> u16 {
        self.max_x().saturating_sub(self.min_x())
    }

    #[inline]
    fn height(&self) -> u16 {
        self.max_y().saturating_sub(self.min_y())
    }

    #[inline]
    fn len(&self) -> usize {
        self.width().saturating_mul(self.height()) as usize
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn bounds(&self) -> Rect {
        Rect::bounds(self.min(), self.max())
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
    fn min_x(&self) -> u16 {
        self.min.x
    }
    fn min_y(&self) -> u16 {
        self.min.y
    }
    fn max_x(&self) -> u16 {
        self.max.x
    }
    fn max_y(&self) -> u16 {
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

impl Bounded for Point {
    fn min_x(&self) -> u16 {
        self.x
    }
    fn min_y(&self) -> u16 {
        self.y
    }
    fn max_x(&self) -> u16 {
        self.x + 1
    }
    fn max_y(&self) -> u16 {
        self.y + 1
    }
}

impl Bounded for Size {
    fn min_x(&self) -> u16 {
        0
    }
    fn min_y(&self) -> u16 {
        0
    }
    fn max_x(&self) -> u16 {
        self.width
    }
    fn max_y(&self) -> u16 {
        self.height
    }

    fn width(&self) -> u16 {
        self.width
    }
    fn height(&self) -> u16 {
        self.height
    }
}
