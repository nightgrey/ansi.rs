use crate::{Coordinate, Point, Rect, Resolve, Size, Steps};

/// A geometry with an axis-aligned min/max bounding rectangle in half-open
/// `[min, max)` coordinates.
pub trait Bound {
    type Point: Coordinate;

    fn min_x(&self) -> u16;
    fn min_y(&self) -> u16;
    fn max_x(&self) -> u16;
    fn max_y(&self) -> u16;

    fn min(&self) -> Self::Point {
        Coordinate::new(self.min_x(), self.min_y())
    }

    fn max(&self) -> Self::Point {
        Coordinate::new(self.max_x(), self.max_y())
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
    fn bounds(&self) -> Rect<Self::Point> {
        Rect::bounds(self.min(), self.max())
    }

    #[inline]
    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    fn steps(self) -> Steps<Self::Point, Self>
    where
        Self: Resolve<Self::Point, usize> + Resolve<usize, Self::Point> + Sized,
    {
        Steps::new(self)
    }
}

impl<P: Coordinate> Bound for Rect<P> {
    type Point = P;

    fn min_x(&self) -> u16 {
        self.min.x()
    }
    fn min_y(&self) -> u16 {
        self.min.y()
    }
    fn max_x(&self) -> u16 {
        self.max.x()
    }
    fn max_y(&self) -> u16 {
        self.max.y()
    }

    fn min(&self) -> Self::Point {
        self.min
    }
    fn max(&self) -> Self::Point {
        self.max
    }

    fn bounds(&self) -> Rect<Self::Point> {
        *self
    }
}

impl<P: Coordinate> Bound for P {
    type Point = Self;

    fn min_x(&self) -> u16 {
        Coordinate::x(self)
    }
    fn min_y(&self) -> u16 {
        Coordinate::y(self)
    }
    fn max_x(&self) -> u16 {
        Coordinate::x(self) + 1
    }
    fn max_y(&self) -> u16 {
        Coordinate::y(self) + 1
    }
}

impl Bound for Size {
    type Point = Point;

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
