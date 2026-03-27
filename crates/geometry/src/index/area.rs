use crate::{Edges, Point, Position, Rect, Steps};
use std::iter::FusedIterator;
use std::ops::{Deref, IntoBounds, RangeBounds};

/// An axis-aligned rectangle for buffer-space coordinates.
///
/// Areas are represented as half-open ranges: `[min, max)`.
/// The `min` position is inclusive, the `max` position is exclusive.
pub type Area = Rect<Position>;
impl Area {
    pub const fn shrink(self, edges: Edges) -> Self {
        let min_row = self.min.row.saturating_add(edges.top);
        let min_col = self.min.col.saturating_add(edges.left);
        let max_row = self.max.row.saturating_sub(edges.bottom);
        let max_col = self.max.col.saturating_sub(edges.right);

        Self {
            min: Position {
                row: min_row.min(max_row),
                col: min_col.min(max_col),
            },
            max: Position {
                row: max_row,
                col: max_col,
            },
        }
    }
}

impl From<Rect> for Area {
    fn from(value: Rect) -> Self {
        Area::new(Position::from(value.min), Position::from(value.max))
    }
}

impl From<Area> for Rect {
    fn from(value: Area) -> Self {
        Rect::new(Point::from(value.min), Point::from(value.max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounded, Contains};

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
