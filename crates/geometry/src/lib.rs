#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_cmp)]
#![feature(derive_const)]
#![feature(const_trait_impl)]
#![feature(range_into_bounds)]
#![feature(const_convert)]
#![feature(const_destruct)]
#![feature(const_range)]
#![feature(exact_size_is_empty)]
#![feature(bound_copied)]

mod edges;
mod index;
mod point;
mod rect;
mod size;
mod region;

pub use edges::*;
pub use index::*;
pub use point::*;
pub use rect::*;
pub use size::*;
pub use region::*;