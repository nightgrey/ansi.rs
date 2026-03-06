use crate::{Bounds, Column, Index, Position, Row};

/// Marker trait for types that represent a spatial location.
pub const trait Location: Copy { }

impl const Location for Index { }

impl const Location for Position { }

impl const Location for Row { }

impl const Location for Column { }

impl const Location for Bounds { }
