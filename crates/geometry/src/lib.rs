//! # Geometric primitives
//!
//! Provides common operations and properties that are useful for working with geometric shapes.
//!
//! ## General
//! - [`Rect`] - Minimum and maximum points forming a rectangular area.
//! - [`Point`] - `x` (horizontal) and `y` (vertical) screen-space coordinates.
//! - [`Size`] - Width and height.
//!
//! ## Additional
//! - [`Edges`] - For paddings, margins, etc.
//! - [`Sides`] - For axis-specific dimensions.
//!
//! ## Index
//! - [`Row`] - A vertical position.
//! - [`Column`] - A horizontal position.
//! - [`Position`] - `row` (vertical) and `col` (horizontal) index coordinates.
//!
//! ## Traits
//! - [`Location`] - An (x, y) coordinate with getters, setters, and a `from_xy` constructor.
//! - [`Bound`] - A geometry with a half-open `[min, max)` bounding rectangle.

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
