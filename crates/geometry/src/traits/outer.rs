use crate::{Bound, Coordinate};
use std::ops::Range;

pub trait Outer: Bound {
    /// Returns the top-left corner.
    fn top_left(&self) -> Self::Point;
    /// Returns the top-right corner.
    fn top_right(&self) -> Self::Point;
    /// Returns the bottom-right corner.
    fn bottom_right(&self) -> Self::Point;
    /// Returns the bottom-left corner.
    fn bottom_left(&self) -> Self::Point;

    /// Returns the y-coordinate of the top edge.
    fn top(&self) -> u16;
    /// Returns the x-coordinate of the left edge.
    fn left(&self) -> u16;
    /// Returns the y-coordinate of the bottom edge.
    fn bottom(&self) -> u16;
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
        self.range_x()
            .flat_map(move |x| self.range_y().map(move |y| (x, y)))
    }
}

impl<B: Bound<Point = P>, P: Coordinate> Outer for B {
    fn top_left(&self) -> Self::Point {
        self.min()
    }
    fn top_right(&self) -> Self::Point {
        Self::Point::new(self.max_x(), self.min_y())
    }
    fn bottom_right(&self) -> Self::Point {
        self.max()
    }
    fn bottom_left(&self) -> Self::Point {
        Self::Point::new(self.min_x(), self.max_y())
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
