mod checked;
mod wrapping;
mod saturating;

pub use checked::*;
pub use wrapping::*;
pub use saturating::*;

use std::ops::*;

pub trait Ops<Rhs = Self, Output = Self>: Add<Rhs, Output=Output> + Sub<Rhs, Output=Output> + Mul<Rhs, Output=Output> + Div<Rhs, Output=Output> + Rem<Rhs, Output=Output> {}
pub trait AssignOps<Rhs = Self>: AddAssign<Rhs> + SubAssign<Rhs> + MulAssign<Rhs> + DivAssign<Rhs> + RemAssign<Rhs> {}

impl<T, Rhs, Output> Ops<Rhs, Output> for T
where
    T: Add<Rhs, Output=Output>
    + Sub<Rhs, Output=Output>
    + Mul<Rhs, Output=Output>
    + Div<Rhs, Output=Output>
    + Rem<Rhs, Output=Output>,
{}

impl<T, Rhs> AssignOps<Rhs> for T
where
    T: AddAssign<Rhs> + SubAssign<Rhs> + MulAssign<Rhs> + DivAssign<Rhs> + RemAssign<Rhs>,
{}

pub trait SpecialtyOps<Rhs = Self>: CheckedOps<Rhs> + SaturatingOps<Rhs> + WrappingOps<Rhs> {}