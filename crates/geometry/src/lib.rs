#![feature(more_float_constants)]
#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_cmp)]
#![feature(derive_const)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(range_into_bounds)]
#![feature(const_destruct)]
#![feature(const_option_ops)]
#![feature(new_range_api)]

mod traits;
pub mod prelude;
mod index;
mod geometry;
mod macros;

pub use geometry::*;
pub use traits::*;
pub use index::*;
pub use macros::*;
use number::{Zero, One, Min, Max};

/// # Geometric primitives
///
/// Provides common operations and properties that are useful for working with geometric shapes.

/// ## Base
/// - [`Rect`] - Minimum and maximum points forming a rectangular area.
/// - [`Point`] - `x` (horizontal) and `y` (vertical) screen-space coordinates.
/// - [`Size`] - Width and height.
///
/// ## Additional
/// - [`Edges`] - For paddings, margins, etc.
/// - [`Sides`] - For axis-specific dimensions.
///
/// ## Index
/// - [`Row`] - A vertical position.
/// - [`Column`] - A horizontal position.
/// - [`Position`] - `row` (vertical) and `col` (horizontal) index coordinates.
///
pub trait Geometric: Sized + Copy + Zero + One + Min + Max {}

impl Geometric for Rect {}
impl Geometric for Size {}
impl Geometric for Edges {}
impl Geometric for Sides {}

impl Geometric for Point {}
impl Geometric for Row {}
impl Geometric for Column {}

pub const trait Coordinate: Locatable {
    fn new(x: u16, y: u16) -> Self;
}

impl Coordinate for Point {
    fn new(x: u16, y: u16) -> Self {
        Point::new(x, y)
    }
}

impl Coordinate for PointLike {
    fn new(x: u16, y: u16) -> Self {
        (x, y)
    }
}

impl Coordinate for Position {
    fn new(x: u16, y: u16) -> Self {
        Position::new(y as usize, x as usize)
    }
}

impl Coordinate for PositionLike {
    fn new(x: u16, y: u16) -> Self {
        (y as usize, x as usize)
    }
}

