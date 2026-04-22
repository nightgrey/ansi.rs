#![feature(bound_copied)]
#![feature(const_range)]
#![feature(slice_range)]
#![feature(extend_one)]

extern crate core;

#[macro_use]
#[macro_export]
mod slot;
pub use slot::*;

#[macro_export]
#[macro_use]
mod separate_by;
pub use separate_by::*;

mod nested_vec;
pub use nested_vec::*;
mod small_byte_string;
pub use small_byte_string::*;
