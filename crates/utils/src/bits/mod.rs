// The `bits!` macro is exported crate-wide via `#[macro_export]`, so this
// module only needs to be visited for that side effect — no glob re-export.
#[macro_use]
pub mod macros;

mod bit;
pub use bit::*;

pub mod bits;
pub use bits::*;

pub mod iter;
pub use iter::*;

pub mod error;

pub use error::*;


pub mod other;
pub use other::*;
