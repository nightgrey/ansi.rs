use geometry::Size;
use crate::{Position, Area, Row, Column, Index, Spatial};

/// Tests if a geometry is completely contained within another geometry.
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<C: Spatial, Rhs: Spatial>  Contains<Rhs> for C {
    fn contains(&self, other: &Rhs) -> bool {
        self.min().row <= other.min().row && self.max().row >= other.max().row &&
            self.min().col <= other.min().col && self.max().col >= other.max().col
    }
}

impl Contains<Position> for Area {
    fn contains(&self, rhs: &Position) -> bool {
        self.min.row <= rhs.row && self.max.row > rhs.row &&
            self.min.col <= rhs.col && self.max.col > rhs.col
    }
}

impl Contains<Row> for Area {
    fn contains(&self, rhs: &Row) -> bool {
        self.min.row <= rhs.value() && self.max.row > rhs.value()
    }
}


impl Contains<Column> for Area {
    fn contains(&self, other: &Column) -> bool {
        self.min.col <= other.value() && self.max.col > other.value()
    }
}

impl Contains<Index> for Area {
    fn contains(&self, other: &Index) -> bool {
        self.len() <= other.value()
    }
}


impl Contains for Position {
    fn contains(&self, rhs: &Position) -> bool {
        self.col == rhs.col && self.row == rhs.row
    }
}

impl Contains<Position> for Size {
    fn contains(&self, rhs: &Position) -> bool {
        0 <= rhs.col && rhs.col < self.width && 0 <= rhs.row && rhs.row < self.height
    }
}

/// Tests if a geometry is completely within another geometry.
pub trait Within<Rhs = Self> {
    fn is_within(&self, rhs: &Rhs) -> bool;
}

impl<A, B> Within<B> for A
where
    B: Contains<A>,
{
    fn is_within(&self, b: &B) -> bool {
        b.contains(self)
    }
}
