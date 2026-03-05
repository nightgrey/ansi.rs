use std::marker::Destruct;
use crate::{Bounds, Column, Position, Row, Index};

mod step;
mod into_location;
mod span;
mod iterator;
mod located;
mod into_slice_index;

pub use step::*;
pub use into_location::*;
pub use span::*;
pub use iterator::*;
pub use located::*;
pub use into_slice_index::*;

/// Marker trait for types that represent a spatial location.
///
/// Types implementing `Location` can describe their "natural" position
/// without any external context.
pub const trait Location: Copy {
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

    #[inline]
    fn area(&self) -> usize { self.width() * self.height() }

    #[inline]
    fn is_empty(&self) -> bool { self.area() == 0 }
    
    #[inline]
    fn bounds(&self) -> Bounds { Bounds::new(self.min(), self.max()) }
}

impl const Context for Bounds {
    fn min(&self) -> Position { self.min }
    fn max(&self) -> Position { self.max }

    fn x(&self) -> usize { self.min.col }
    fn y(&self) -> usize { self.min.row }
    
    fn bounds(&self) -> Bounds { *self }
}
