use std::iter::FusedIterator;
use std::ops::{Deref, IntoBounds, RangeBounds};
use geometry::{Point, Rect};
use crate::{Position, Steps, Step, Spatial, Sides};

/// Half-open (min..=max) area.
#[derive(Copy, Debug)]
#[derive_const(Default, Clone, Eq, PartialEq)]
pub struct Area {
    pub min: Position,
    pub max: Position,
}

impl Area {
    pub const ZERO: Self = Self {
        min: Position::ZERO,
        max: Position::ZERO,
    };

    /// Creates a new area from inclusive `min` to exclusive `max`.
    ///
    /// # Panics (debug only)
    /// Panics if `min > max` on either axis.
    pub const fn new(min: Position, max: Position) -> Self {
        Self { min, max }
    }

    /// Create a new area from its top-left corner and size.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use spatial::{Area, Position};
    /// let area = Area::bounds(5, 10, 10, 20);
    /// ```
    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self::new(Position::new(x, y), Position::new(x + width, y + height))
    }

    /// Row-major iterator over every position in the area.
    pub  fn iter(&self) -> Steps {
       self.positions()
    }
}

impl IntoIterator for Area {
    type Item = Position;
    type IntoIter = Steps;
    fn into_iter(self) -> Self::IntoIter {
        Steps::new(&self)
    }
}

impl IntoIterator for &Area {
    type Item = Position;
    type IntoIter = Steps;
    fn into_iter(self) -> Self::IntoIter {
        Steps::new(self)
    }
}


impl const Spatial for Area {
    fn area(&self) -> Area {
        *self
    }

    fn min(&self) -> Position {
        self.min
    }

    fn max(&self) -> Position {
        self.max
    }

    fn width(&self) -> usize {
        self.max.col - self.min.col
    }

    fn height(&self) -> usize {
        self.max.row - self.min.row
    }

    fn len(&self) -> usize {
        self.width().saturating_mul(self.height())
    }

    fn is_empty(&self) -> bool {
        self.min == self.max
    }
}

impl From<Rect> for Area {
    fn from(value: Rect) -> Self {
        Area::new(
            Position::from(value.min),
            Position::from(value.max),
        )
    }
}

impl From<Area> for Rect {
    fn from(value: Area) -> Self {
        Rect::new(
            Point::from(value.min),
            Point::from(value.max),
        )
    }
}
#[cfg(test)]
mod tests {
    use crate::{Contains};
    use super::*;

    #[test]
    fn test_area_new() {
        let r = Area::new(Position::new(5, 10), Position::new(15, 30));
        assert_eq!(r.min, Position::new(5, 10));
        assert_eq!(r.max, Position::new(15, 30));
    }

    #[test]
    fn test_area_width_height() {
        let r = Area::new(Position::new(0, 0), Position::new(5, 10));
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 5);
    }

    #[test]
    fn test_area_area() {
        let r = Area::new(Position::new(0, 0), Position::new(4, 5));
        assert_eq!(r.len(), 20); // 4 * 5
    }

    #[test]
    fn test_area_contains() {
        let r = Area::new(Position::new(10, 10), Position::new(20, 20));

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
    fn test_area_size() {
        let r = Area::new(Position::new(0, 0), Position::new(24, 80));
        let size = r.size();
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }
}
