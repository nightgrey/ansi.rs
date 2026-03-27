use crate::{Bounded, Edges, Location, Point, Rect, Size};
use std::ops::Range;

pub trait Outer: Bounded {
    #[inline]
    /// Returns the x-coordinate of the left edge.
    fn top_left(&self) -> Self::Point;
    #[inline]
    /// Returns the y-coordinate of the top edge.
    fn top_right(&self) -> Self::Point;
    #[inline]
    /// Returns the y-coordinate of the bottom edge.
    fn bottom_right(&self) -> Self::Point;
    #[inline]
    /// Returns the x-coordinate of the right edge.
    fn bottom_left(&self) -> Self::Point;
}

pub trait Sides: Bounded {
    #[inline]
    /// Returns the y-coordinate of the top edge.
    fn top(&self) -> usize;
    #[inline]
    /// Returns the x-coordinate of the left edge.
    fn left(&self) -> usize;
    #[inline]
    /// Returns the y-coordinate of the bottom edge.
    fn bottom(&self) -> usize;
    #[inline]
    /// Returns the x-coordinate of the right edge.
    fn right(&self) -> usize;
}

impl<T: Bounded> Sides for T {
    fn top(&self) -> usize {
        self.min_y()
    }

    fn left(&self) -> usize {
        self.min_x()
    }

    fn bottom(&self) -> usize {
        self.max_y()
    }

    fn right(&self) -> usize {
        self.max_x()
    }
}
impl<T: Bounded> Outer for T {
    fn top_right(&self) -> Self::Point {
        Self::Point::new(self.max_x(), self.min_y())
    }

    fn top_left(&self) -> Self::Point {
        self.min()
    }

    fn bottom_right(&self) -> Self::Point {
        self.max()
    }

    fn bottom_left(&self) -> Self::Point {
        Self::Point::new(self.min_x(), self.max_y())
    }
}

pub trait Ranges: Sides {
    #[inline]
    fn range_x(&self) -> Range<usize> {
        self.left()..self.right()
    }
    #[inline]
    fn range_y(&self) -> Range<usize> {
        self.top()..self.bottom()
    }

    fn ranges(&self) -> impl Iterator<Item = (usize, usize)> {
        self.range_x().flat_map(move |x| self.range_y().map(move |y| (x, y)))
    }
}

impl<T: Sides> Ranges for T {}
