#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(bstr)]
#![feature(ascii_char)]
#![feature(const_range)]
#![feature(const_range_bounds)]
#![feature(const_destruct)]
#![feature(iter_intersperse)]
#![feature(const_ops)]
#![feature(formatting_options)]
#![feature(const_index)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_cmp)]
#![feature(extend_one)]
#![feature(ascii_char_variants)]
#![feature(step_trait)]
#![feature(range_into_bounds)]
#![feature(const_slice_make_iter)]
#![feature(iter_advance_by)]
#![feature(const_iter)]
#![feature(const_result_trait_fn)]
#![feature(const_try)]
#![feature(const_option_ops)]
extern crate core;

#[macro_use]
mod slot;

#[macro_use]
mod separate_by;

pub mod nested;
pub use nested::*;

pub mod byte_string;

pub use byte_string::*;
pub mod ansi;
pub mod counting_writer;

pub use counting_writer::*;

pub use ansi::*;

pub use utils_derive::*;
