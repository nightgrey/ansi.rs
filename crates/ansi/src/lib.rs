#![feature(ascii_char)]
#![feature(bstr)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]

mod color;
mod style;
mod escape;

pub use escape::*;
pub use style::*;
pub use color::*;