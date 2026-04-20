#![feature(derive_const)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_convert)]
#![feature(const_option_ops)]
#![feature(slice_index_methods)]
#![feature(iter_intersperse)]
#![feature(const_index)]
#![feature(const_result_trait_fn)]
#![feature(const_try)]
#![feature(once_cell_get_mut)]
extern crate core;

pub mod drawing;
pub use drawing::*;

pub mod layout;
pub use layout::*;

pub mod buffer;
pub use buffer::*;

pub mod raster;
pub use raster::*;

pub mod elements;
pub use elements::*;

mod engine;
pub use engine::*;

mod document;
pub mod mock;
pub mod presenter;

pub use document::*;
