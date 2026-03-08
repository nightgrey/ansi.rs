use std::ops::Range;
use crate::{Spatial};

pub const trait Sides {
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

impl<S: [const] Spatial> const Sides for S {
    fn top(&self) -> usize {
        self.min().row
    }

    fn left(&self) -> usize {
        self.min().col
    }

    fn bottom(&self) -> usize {
        self.max().row
    }

    fn right(&self) -> usize {
        self.max().col
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