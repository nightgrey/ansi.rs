use std::ops::Range;
use crate::{Rect, Size, Bounded};

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

impl Sides for Rect {
    fn top(&self) -> usize {
        self.min.y
    }

    fn left(&self) -> usize {
        self.min.x
    }

    fn bottom(&self) -> usize {
        self.max.y
    }

    fn right(&self) -> usize {
        self.max.x
    }
}

impl Sides for Size {
    fn top(&self) -> usize {
        0
    }

    fn left(&self) -> usize {
        0
    }

    fn bottom(&self) -> usize {
        self.height
    }

    fn right(&self) -> usize {
        self.width
    }
}

pub trait Ranges: Sides {
    #[inline]
    fn horizontal(&self) -> Range<usize> {
        self.left()..self.right()
    }
    #[inline]
    fn vertical(&self) -> Range<usize> {
        self.top()..self.bottom()
    }
}

impl<T: Sides> Ranges for T {}