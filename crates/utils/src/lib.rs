#![feature(bound_copied)]
#![feature(const_range)]
#![feature(slice_range)]
extern crate core;

#[macro_use]
#[macro_export]
mod slot;
pub use slot::*;

#[macro_export]
#[macro_use]
mod separate_by;
pub use separate_by::*;
