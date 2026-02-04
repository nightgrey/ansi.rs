#![feature(slice_index_methods)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_range)]
#![feature(option_reference_flattening)]
extern crate core;

mod buffer;
mod text;
pub mod ui;

pub use buffer::*;
pub use text::*;
pub use ui::*;
