#![feature(ascii_char)]
#![feature(bstr)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]

mod attribute;
mod color;
mod escape;
mod style;
mod bit_color;

pub use attribute::*;
pub use color::*;
pub use escape::*;
pub use style::*;
