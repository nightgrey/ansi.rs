use std::marker::Destruct;
use crate::{Bounds, Column, Index, Position, Row};

mod step;
mod into_position;
mod range_bounds;
mod iterator;

pub use step::*;
pub use into_position::*;
pub use range_bounds::*;
pub use iterator::*;

pub const trait Location {}
impl const Location for Position {}
impl const Location for Row {}
impl const Location for Column {}
impl const Location for Index {}
impl const Location for usize {}
impl const Location for Bounds {}
