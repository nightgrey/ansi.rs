use crate::{Bounds, Column, SpatialIndex, Position, Row, Index};

mod step;
mod into_position;
mod span;
mod iterator;
mod located;
mod indexable;

pub use step::*;
pub use into_position::*;
pub use span::*;
pub use iterator::*;
pub use located::*;
pub use indexable::*;
/// Marker trait for types that represent a spatial location.
///
/// Types implementing `Location` can describe their "natural" position
/// without any external context.
pub const trait Location {
}

impl const Location for Position {
}

impl const Location for Row {
}

impl const Location for Column {

}

impl const Location for Bounds {
}


/// Marker trait for types that represent an external spatial context.
pub const trait Context {
    fn min(&self) -> Position;
    fn max(&self) -> Position;

    fn x(&self) -> usize { self.min().col }
    fn y(&self) -> usize { self.min().row }

    fn width(&self) -> usize { self.max().col.saturating_sub(self.min().col) }
    fn height(&self) -> usize { self.max().row.saturating_sub(self.min().row) }

    fn area(&self) -> usize { self.width() * self.height() }
}

impl const Context for Bounds {
    fn min(&self) -> Position { self.min }
    fn max(&self) -> Position { self.max }
}