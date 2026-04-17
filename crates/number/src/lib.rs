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


pub mod consts;
pub mod ops;
pub mod integer;
pub mod float;
pub mod number;

pub use consts::*;
pub use ops::*;
pub use integer::*;
pub use float::*;
pub use number::*;
