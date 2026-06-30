mod checked;
mod saturating;
mod wrapping;

pub use checked::*;
pub use saturating::*;
pub use wrapping::*;

use std::ops::*;

pub const trait Ops<Rhs = Self, Output = Self>:
    [const] Add<Rhs, Output = Output>
    + [const] Sub<Rhs, Output = Output>
    + [const] Mul<Rhs, Output = Output>
    + [const] Div<Rhs, Output = Output>
    + [const] Rem<Rhs, Output = Output>
{
}

pub const trait AssignOps<Rhs = Self>:
    [const] AddAssign<Rhs>
    + [const] SubAssign<Rhs>
    + [const] MulAssign<Rhs>
    + [const] DivAssign<Rhs>
    + [const] RemAssign<Rhs>
{
}

pub const trait ConditionalOps<Rhs = Self>:
    [const] CheckedOps<Rhs> + [const] SaturatingOps<Rhs> + [const] WrappingOps<Rhs>
{
}

const impl<T, Rhs, Output> Ops<Rhs, Output> for T where
    T: [const] Add<Rhs, Output = Output>
        + [const] Sub<Rhs, Output = Output>
        + [const] Mul<Rhs, Output = Output>
        + [const] Div<Rhs, Output = Output>
        + [const] Rem<Rhs, Output = Output>
{
}

const impl<T, Rhs> AssignOps<Rhs> for T where
    T: [const] AddAssign<Rhs>
        + [const] SubAssign<Rhs>
        + [const] MulAssign<Rhs>
        + [const] DivAssign<Rhs>
        + [const] RemAssign<Rhs>
{
}

const impl<T, Rhs> ConditionalOps<Rhs> for T where
    T: [const] CheckedOps<Rhs> + [const] SaturatingOps<Rhs> + [const] WrappingOps<Rhs>
{
}

pub const trait BitOps<Rhs = Self>:
    [const] BitAnd<Rhs, Output = Self>
    + [const] BitAndAssign<Rhs>
    + [const] BitOr<Rhs, Output = Self>
    + [const] BitOrAssign<Rhs>
    + [const] BitXor<Rhs, Output = Self>
    + [const] BitXorAssign<Rhs>
    + [const] Not<Output = Self>
{
}

const impl<T, Rhs> BitOps<Rhs> for T where
    T: [const] BitAnd<Rhs, Output = Self>
        + [const] BitAndAssign<Rhs>
        + [const] BitOr<Rhs, Output = Self>
        + [const] BitOrAssign<Rhs>
        + [const] BitXor<Rhs, Output = Self>
        + [const] BitXorAssign<Rhs>
        + [const] Not<Output = Self>
{
}
