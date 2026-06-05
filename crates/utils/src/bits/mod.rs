// The `bits!` macro is exported crate-wide via `#[macro_export]`, so this
// module only needs to be visited for that side effect — no glob re-export.
#[macro_use]
pub mod macros;

mod bits;

pub use bits::*;