#![feature(more_float_constants)]
#![feature(const_trait_impl)]

pub mod consts;
pub mod float;
pub mod integer;
pub mod number;
pub mod ops;

pub use consts::*;
pub use float::*;
pub use integer::*;
pub use number::*;
pub use ops::*;
