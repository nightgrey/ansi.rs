use crate::{Bounds, Column, SpatialIndex, Position, Row, Index};

mod step;
mod into_position;
mod span;
mod iterator;
mod located;

pub use step::*;
pub use into_position::*;
pub use span::*;
pub use iterator::*;
pub use located::*;

/// Marker trait for types that represent a spatial location.
///
/// Types implementing `Location` can describe their "natural" position
/// without any external context. For fully self-describing types like
/// `Position` and `Bounds`, this is exact. For partial types like `Row`
/// or `Column`, the missing axis defaults to 0.
pub const trait Location {
    /// The natural position of this location, without external context.
    ///
    /// - `Position` → itself
    /// - `Bounds` → its min corner
    /// - `Row(r)` → `Position::new(r, 0)`
    /// - `Column(c)` → `Position::new(0, c)`
    /// - `usize` / `Index` → `Position::ZERO` (no width to resolve)
    fn position(&self) -> Position;
}

impl const Location for Position {
    fn position(&self) -> Position { *self }
}

impl const Location for Row {
    fn position(&self) -> Position { Position::new(self.0, 0) }
}

impl const Location for Column {
    fn position(&self) -> Position { Position::new(0, self.0) }
}

impl const Location for Index {
    fn position(&self) -> Position { Position::ZERO }
}

impl const Location for usize {
    fn position(&self) -> Position { Position::ZERO }
}

impl const Location for Bounds {
    fn position(&self) -> Position { self.min }
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