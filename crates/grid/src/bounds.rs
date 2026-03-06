use std::iter::FusedIterator;
use std::ops::{Deref, IntoBounds, RangeBounds};
use crate::{Position, Steps, Step, Cursor, Context, Intersect, Contains};

/// Half-open (min..=max) rectangular bounds in row-major space.
#[derive(Copy, Debug)]
#[derive_const(Default, Clone, Eq, PartialEq)]
pub struct Bounds {
    pub min: Position,
    pub max: Position,
}

impl Bounds {
    pub const ZERO: Self = Self {
        min: Position::ZERO,
        max: Position::ZERO,
    };

    /// Creates a new region from inclusive `min` to exclusive `max`.
    ///
    /// # Panics (debug only)
    /// Panics if `min > max` on either axis.
    pub const fn new(min: Position, max: Position) -> Self {
        Self { min, max }
    }

    /// Create a new bounds from its top-left corner and size.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geometry::{Bounds, Position};
    /// let bounds = Bounds::corners(5, 10, 10, 20);
    /// ```
    pub const fn corners(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self::new(Position::new(x, y), Position::new(x + width, y + height))
    }

    /// Row-major iterator over every position in the region.
    pub const fn iter(&self) -> Steps {
       self.positions()
    }
}

impl IntoIterator for &Bounds {
    type Item = Position;
    type IntoIter = Steps;
    fn into_iter(self) -> Self::IntoIter {
        Steps::new(self)
    }
}

impl const Context for Bounds {
    fn min(&self) -> Position { self.min }
    fn max(&self) -> Position { self.max }

    fn bounds(&self) -> Bounds { *self }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_new() {
        let r = Bounds::new(Position::new(5, 10), Position::new(15, 30));
        assert_eq!(r.min, Position::new(5, 10));
        assert_eq!(r.max, Position::new(15, 30));
    }

    #[test]
    fn test_bounds_width_height() {
        let r = Bounds::new(Position::new(0, 0), Position::new(5, 10));
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 5);
    }

    #[test]
    fn test_bounds_area() {
        let r = Bounds::new(Position::new(0, 0), Position::new(4, 5));
        assert_eq!(r.area(), 20); // 4 * 5
    }

    #[test]
    fn test_bounds_contains() {
        let r = Bounds::new(Position::new(10, 10), Position::new(20, 20));

        // Inside
        assert!(r.contains(&Position::new(15, 15)));

        // Min edge (inclusive)
        assert!(r.contains(&Position::new(10, 10)));

        // Max edge (exclusive)
        assert!(!r.contains(&Position::new(20, 20)));

        // Outside
        assert!(!r.contains(&Position::new(25, 25)));
        assert!(!r.contains(&Position::new(5, 5)));
    }

    #[test]
    fn test_bounds_size() {
        let r = Bounds::new(Position::new(0, 0), Position::new(24, 80));
        let size = r.size();
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }
}
