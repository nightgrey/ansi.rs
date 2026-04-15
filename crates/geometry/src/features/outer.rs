use crate::{Bounded, Coordinated, Edges, Point, Rect, Size};
use std::ops::Range;

pub trait Outer: Bounded {
    #[inline]
    /// Returns the x-coordinate of the left edge.
    fn top_left(&self) -> Self::Coordinate;
    #[inline]
    /// Returns the y-coordinate of the top edge.
    fn top_right(&self) -> Self::Coordinate;
    #[inline]
    /// Returns the y-coordinate of the bottom edge.
    fn bottom_right(&self) -> Self::Coordinate;
    #[inline]
    /// Returns the x-coordinate of the right edge.
    fn bottom_left(&self) -> Self::Coordinate;

    #[inline]
    /// Returns the y-coordinate of the top edge.
    fn top(&self) -> u16;
    #[inline]
    /// Returns the x-coordinate of the left edge.
    fn left(&self) -> u16;
    #[inline]
    /// Returns the y-coordinate of the bottom edge.
    fn bottom(&self) -> u16;
    #[inline]
    /// Returns the x-coordinate of the right edge.
    fn right(&self) -> u16;

    #[inline]
    fn range_x(&self) -> Range<u16> {
        self.left()..self.right()
    }
    #[inline]
    fn range_y(&self) -> Range<u16> {
        self.top()..self.bottom()
    }

    fn ranges(&self) -> impl Iterator<Item = (u16, u16)> {
        self.range_x().flat_map(move |x| self.range_y().map(move |y| (x, y)))
    }
}

impl<C: Coordinated, T: Bounded<Coordinate = C>> Outer for T {
    fn top_left(&self) -> Self::Coordinate {
        self.min()
    }

    fn top_right(&self) -> Self::Coordinate {
        Self::Coordinate::new(self.max_x(), self.min_y())
    }

    fn bottom_right(&self) -> Self::Coordinate {
        self.max()
    }

    fn bottom_left(&self) -> Self::Coordinate {
        Self::Coordinate::new(self.min_x(), self.max_y())
    }

    fn top(&self) -> u16 {
        self.min_y()
    }

    fn left(&self) -> u16 {
        self.min_x()
    }

    fn bottom(&self) -> u16 {
        self.max_y()
    }

    fn right(&self) -> u16 {
        self.max_x()
    }
}
