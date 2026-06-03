mod checked;
mod saturating;
mod wrapping;

pub use checked::*;
pub use saturating::*;
pub use wrapping::*;

use std::ops::*;

pub const trait Ops<Rhs = Self, Output = Self>:
    Add<Rhs, Output = Output>
    + Sub<Rhs, Output = Output>
    + Mul<Rhs, Output = Output>
    + Div<Rhs, Output = Output>
    + Rem<Rhs, Output = Output>
{
}

pub const trait AssignOps<Rhs = Self>:
    AddAssign<Rhs> + SubAssign<Rhs> + MulAssign<Rhs> + DivAssign<Rhs> + RemAssign<Rhs>
{
}

pub const trait ConditionalOps<Rhs = Self>:
    CheckedOps<Rhs> + SaturatingOps<Rhs> + WrappingOps<Rhs>
{
}

pub const trait BitOps<Rhs = Self>:
    BitAnd<Rhs, Output = Self>
    + BitAndAssign<Rhs>
    + BitOr<Rhs, Output = Self>
    + BitOrAssign<Rhs>
    + BitXor<Rhs, Output = Self>
    + BitXorAssign<Rhs>
    + Not<Output = Self>
{
}

impl<T, Rhs, Output> const Ops<Rhs, Output> for T where
    T: Add<Rhs, Output = Output>
        + Sub<Rhs, Output = Output>
        + Mul<Rhs, Output = Output>
        + Div<Rhs, Output = Output>
        + Rem<Rhs, Output = Output>
{
}

impl<T, Rhs> const AssignOps<Rhs> for T where
    T: AddAssign<Rhs> + SubAssign<Rhs> + MulAssign<Rhs> + DivAssign<Rhs> + RemAssign<Rhs>
{
}

impl<T, Rhs> const ConditionalOps<Rhs> for T where
    T: CheckedOps<Rhs> + SaturatingOps<Rhs> + WrappingOps<Rhs>
{
}


impl<T, Rhs> const BitOps<Rhs> for T where
    T: BitAnd<Rhs, Output = Self> + BitAndAssign<Rhs> + BitOr<Rhs, Output = Self> + BitOrAssign<Rhs> + BitXor<Rhs, Output = Self> + BitXorAssign<Rhs> + Not<Output = Self>
{
}