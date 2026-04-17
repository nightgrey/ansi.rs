use crate::{Anchor, Edges, Point, Rect, Size};

/// A geometry with an axis-aligned min/max bounding rectangle in half-open
/// `[min, max)` coordinates.
pub trait Bound {
    type Point: Anchor;
  
    fn min_x(&self) -> u16;
    fn min_y(&self) -> u16;
    fn max_x(&self) -> u16;
    fn max_y(&self) -> u16;

    fn min(&self) -> Self::Point;
    fn max(&self) -> Self::Point;
 
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
}

impl<C: Anchor> Bound for Rect<C> {
    type Point = C;

    fn min_x(&self) -> u16 { self.min.x() }
    fn min_y(&self) -> u16 { self.min.y() }
    fn max_x(&self) -> u16 { self.max.x() }
    fn max_y(&self) -> u16 { self.max.y() }

    fn min(&self) -> Self::Point { self.min }
    fn max(&self) -> Self::Point { self.max }

    fn bounds(&self) -> Rect<Self::Point> {
        *self
    }
}

impl Bound for Size {
    type Point = Point;

    fn min_x(&self) -> u16 { 0 }
    fn min_y(&self) -> u16 { 0 }
    fn max_x(&self) -> u16 { self.width }
    fn max_y(&self) -> u16 { self.height }

    fn min(&self) -> Self::Point { Point { x: 0, y: 0 } }
    fn max(&self) -> Self::Point {
        Point {
            x: self.width,
            y: self.height,
        }
    }

    fn width(&self) -> u16 { self.width }
    fn height(&self) -> u16 { self.height }
}

impl Bound for Edges {
    type Point = Point;

    fn min_x(&self) -> u16 { 0 }
    fn min_y(&self) -> u16 { 0 }
    fn max_x(&self) -> u16 { self.horizontal() }
    fn max_y(&self) -> u16 { self.vertical() }

    fn min(&self) -> Self::Point { Point::ZERO }
    fn max(&self) -> Self::Point {
        Point::new(self.horizontal(), self.vertical())
    }
}

impl Bound for Point {
    type Point = Self;

    fn min_x(&self) -> u16 { self.x }
    fn min_y(&self) -> u16 { self.y }
    fn max_x(&self) -> u16 { self.x + 1 }
    fn max_y(&self) -> u16 { self.y + 1 }

    fn min(&self) -> Self::Point { *self }
    fn max(&self) -> Self::Point {
        Point { x: self.x + 1, y: self.y + 1 }
    }
}
