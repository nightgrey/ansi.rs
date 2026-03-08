#![feature(slice_index_methods)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_range)]
#![feature(option_reference_flattening)]
#![feature(const_index)]
#![feature(slice_get_slice)]
#![feature(slice_get_slice_impls)]
#![feature(slice_index_with_ops_bound_pair)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(trusted_len)]
#![feature(try_trait_v2)]
#![feature(ub_checks)]
#![feature(core_intrinsics)]
#![feature(try_blocks)]
#![feature(iter_advance_by)]
#![feature(fmt_arguments_from_str)]
#![feature(panic_internals)]
#![feature(new_range_api)]
#![feature(step_trait)]
#![feature(const_convert)]
#![feature(const_default)]
#![feature(range_into_bounds)]
#![feature(const_destruct)]
#![feature(trivial_bounds)]
#![feature(range_bounds_is_empty)]
#![feature(bstr)]
#![feature(iter_intersperse)]
extern crate core;

pub mod buffer;
pub mod layout;
pub mod rasterizer;
pub mod painter;
pub mod engine;

pub use buffer::*;
pub use rasterizer::*;
pub use layout::*;
pub use engine::*;