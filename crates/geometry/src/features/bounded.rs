use crate::{Edges, Point, Rect, Size};
/// Provides the bounds of a geometry.
pub trait Bounded {
    type Coordinate;
    type Bounds;

    #[inline]
    fn min_x(&self) -> usize;

    #[inline]
    fn min_y(&self) -> usize;

    #[inline]
    fn max_x(&self) -> usize;

    #[inline]
    fn max_y(&self) -> usize;

    #[inline]
    fn min(&self) -> Self::Coordinate;

    #[inline]
    fn max(&self) -> Self::Coordinate;

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
    fn bounds(&self) -> Self::Bounds;

    #[inline]
    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}

impl Bounded for Rect {
    type Coordinate = Point;
    type Bounds = Rect;

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

    fn min(&self) -> Self::Coordinate {
        self.min
    }

    fn max(&self) -> Self::Coordinate {
        self.max
    }

    fn bounds(&self) -> Self::Bounds {
        *self
    }
}
// impl Bounded for Area {
//     type Point = Position;
//     type Bounds = Area;
// 
//     fn min_x(&self) -> usize {
//         self.min.col
//     }
// 
//     fn min_y(&self) -> usize {
//         self.min.row
//     }
// 
//     fn max_x(&self) -> usize {
//         self.max.col
//     }
// 
//     fn max_y(&self) -> usize {
//         self.max.row
//     }
// 
//     fn min(&self) -> Self::Point {
//         self.min
//     }
// 
//     fn max(&self) -> Self::Point {
//         self.max
//     }
// 
//     fn bounds(&self) -> Self::Bounds {
//         *self
//     }
// }
impl Bounded for Size {
    type Coordinate = Point;
    type Bounds = Rect;

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

    fn min(&self) -> Self::Coordinate {
        Point { x: 0, y: 0 }
    }

    fn max(&self) -> Self::Coordinate {
        Point {
            x: self.width,
            y: self.height,
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn bounds(&self) -> Self::Bounds {
        Rect::new(self.min(), self.max())
    }
}

impl Bounded for Edges {
    type Coordinate = Point;
    type Bounds = Rect;

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

    fn min(&self) -> Self::Coordinate {
        Point::ZERO
    }

    fn max(&self) -> Self::Coordinate {
        Point::new(self.horizontal(), self.vertical())
    }

    fn bounds(&self) -> Self::Bounds {
        Rect::new(self.min(), self.max())
    }
}
