use std::marker::Destruct;
use geometry::Size;
use crate::{Area, Column, Index, Position, Row, Steps};

/// Type that represents a spatial context.
pub const trait Context {
    #[inline]
    fn min(&self) -> Position;

    #[inline]
    fn max(&self) -> Position;

    #[inline]
    fn x(&self) -> usize { self.min().col }

    #[inline]
    fn y(&self) -> usize { self.min().row }

    #[inline]
    fn width(&self) -> usize { self.max().col.saturating_sub(self.min().col) }

    #[inline]
    fn height(&self) -> usize { self.max().row.saturating_sub(self.min().row) }

    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    #[inline]
    fn len(&self) -> usize { self.width() * self.height() }

    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }

    #[inline]
    fn area(&self) -> Area { Area::new(self.min(), self.max()) }

    fn positions(&self) -> Steps where Self: Sized {
        Steps::new(self)
    }
}

pub const trait Intersect<Rhs = Self, Output = Self>: Context {
    fn intersect(&self, other: &Rhs) -> Output;

    fn clip(&self, other: &Rhs) -> Output;
}

impl<C: [const] Context + [const] Destruct, Rhs: [const] Context + [const] Destruct> const Intersect<Rhs, Area> for C {
    fn intersect(&self, other: &Rhs) -> Area {
        let min_row = self.min().row.max(other.min().row);
        let min_col = self.min().col.max(other.min().col);
        let max_row = self.max().row.min(other.max().row);
        let max_col = self.max().col.min(other.max().col);

        // Clamp to empty if min overtakes max on either axis.
        let (max_row, max_col) = if min_row > max_row || min_col > max_col {
            (min_row, min_col)
        } else {
            (max_row, max_col)
        };

        Area {
            min: Position::new(min_row, min_col),
            max: Position::new(max_row, max_col),
        }
    }

    fn clip(&self, other: &Rhs) -> Area {
        other.intersect(&self.area())
    }
}

pub const trait Contains<Rhs = Self>: Context {
    fn contains(&self, other: &Rhs) -> bool;
}

impl<C: [const] Context> const Contains<Position> for C {
    fn contains(&self, other: &Position) -> bool {
        self.min().row <= other.row && self.max().row > other.row &&
            self.min().col <= other.col && self.max().col > other.col
    }
}

impl<C: [const] Context> const Contains<Row> for C {
    fn contains(&self, other: &Row) -> bool {
        self.min().row <= other.value() && self.max().row > other.value()
    }
}

impl<C: [const] Context> const Contains<Column> for C {
    fn contains(&self, other: &Column) -> bool {
        self.min().col <= other.value() && self.max().col > other.value()
    }
}

impl<C: [const] Context> const Contains<Index> for C {
    fn contains(&self, other: &Index) -> bool {
        self.len() <= other.value()
    }
}

impl<C: [const] Context, Rhs: [const] Context + [const] Destruct> const Contains<Rhs> for C {
    fn contains(&self, other: &Rhs) -> bool {
        self.min().row <= other.min().row && self.max().row >= other.max().row &&
            self.min().col <= other.min().col && self.max().col >= other.max().col
    }
}
