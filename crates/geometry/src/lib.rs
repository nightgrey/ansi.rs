#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_cmp)]
#![feature(derive_const)]
#![feature(const_trait_impl)]

mod edges;
mod point;
mod rect;
mod size;
pub mod features;
pub mod prelude;
mod axis;

pub use edges::*;
pub use point::*;
pub use rect::*;
pub use size::*;
pub use features::*;
pub use axis::*;
