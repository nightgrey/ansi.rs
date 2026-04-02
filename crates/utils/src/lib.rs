#![feature(bound_copied)]
#![feature(const_range)]
#![feature(slice_range)]
extern crate core;

#[macro_export]
#[macro_use]
mod separate_by;
mod resolve;
pub use separate_by::*;
pub use resolve::*;
