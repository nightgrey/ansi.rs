use crate::{Coordinate, Edges, Point, Rect, SaturatingSub, Size};
use crate::Locatable;

/// Provides the bounds of a geometry.
pub trait Bounded {
    type Coordinate: Locatable;

    #[inline]
    fn min_x(&self) -> u16;

    #[inline]
    fn min_y(&self) -> u16;

    #[inline]
    fn max_x(&self) -> u16;

    #[inline]
    fn max_y(&self) -> u16;

    #[inline]
    fn min(&self) -> Self::Coordinate;

    #[inline]
    fn max(&self) -> Self::Coordinate;

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
        (self.width()).saturating_mul(self.height()) as usize
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn bounds(&self) -> Rect<Self::Coordinate> {
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

impl<C: Coordinate> Bounded for Rect<C> {
    type Coordinate = C;

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

    fn min(&self) -> Self::Coordinate {
        self.min
    }

    fn max(&self) -> Self::Coordinate {
        self.max
    }

    fn bounds(&self) ->  Rect<Self::Coordinate> {
        *self
    }
}

impl Bounded for Size {
    type Coordinate = Point;

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

    fn min(&self) -> Self::Coordinate {
        Point { x: 0, y: 0 }
    }

    fn max(&self) -> Self::Coordinate {
        Point {
            x: self.width,
            y: self.height,
        }
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn bounds(&self) -> Rect {
        Rect::bounds(self.min(), self.max())
    }
}

impl Bounded for Edges {
    type Coordinate = Point;

    fn min_x(&self) -> u16 {
        0
    }

    fn min_y(&self) -> u16 {
        0
    }

    fn max_x(&self) -> u16 {
        self.horizontal()
    }

    fn max_y(&self) -> u16 {
        self.vertical()
    }

    fn min(&self) -> Self::Coordinate {
        Point::ZERO
    }

    fn max(&self) -> Self::Coordinate {
        Point::new(self.horizontal(), self.vertical())
    }
}

impl Bounded for Point {
    type Coordinate = Self;
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

    fn min(&self) -> Self::Coordinate {
        *self
    }

    fn max(&self) -> Self::Coordinate {
        Point { x: self.x + 1, y: self.y + 1 }
    }

    fn bounds(&self) -> Rect {
        Rect::bounds(Bounded::min(self), Bounded::max(self))
    }
}