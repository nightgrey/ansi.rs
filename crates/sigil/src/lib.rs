#![feature(slice_index_methods)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_range)]
#![feature(option_reference_flattening)]

mod buffer;
mod ui;
mod text;

pub use buffer::*;
pub use ui::*;
pub use text::*;