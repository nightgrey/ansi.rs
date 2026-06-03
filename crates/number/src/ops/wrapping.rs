use std::num::Wrapping;
use std::ops::{Add, Div, Mul, Rem, Sub};

pub const trait WrappingOps<Rhs = Self>:
    [const] WrappingAdd<Rhs> + [const] WrappingSub<Rhs> + [const] WrappingMul<Rhs> + [const] WrappingDiv<Rhs> + [const] WrappingRem<Rhs>
{
}

pub const trait WrappingAdd<Rhs = Self>: Sized + Add<Rhs, Output = Self> {
    fn wrapping_add(self, other: Rhs) -> Self;
}

pub const trait WrappingSub<Rhs = Self>: Sized + Sub<Self, Output = Self> {
    fn wrapping_sub(self, other: Rhs) -> Self;
}

pub const trait WrappingMul<Rhs = Self>: Sized + Mul<Self, Output = Self> {
    fn wrapping_mul(self, other: Rhs) -> Self;
}

pub const trait WrappingDiv<Rhs = Self>: Sized + Div<Self, Output = Self> {
    fn wrapping_div(self, other: Rhs) -> Self;
}

pub const trait WrappingRem<Rhs = Self>: Sized + Rem<Self, Output = Self> {
    fn wrapping_rem(self, other: Rhs) -> Self;
}

macro_rules! wrapping_impl {
    ( $ T: ty) => {
        impl const WrappingOps for $T {}

        impl const WrappingAdd for $T {
            fn wrapping_add(self, rhs: Self) -> Self {
                (Wrapping(self) + Wrapping(rhs)).0
            }
        }

        impl const WrappingSub for $T {
            fn wrapping_sub(self, rhs: Self) -> Self {
                (Wrapping(self) - Wrapping(rhs)).0
            }
        }

        impl const WrappingMul for $T {
            fn wrapping_mul(self, rhs: Self) -> Self {
                (Wrapping(self) * Wrapping(rhs)).0
            }
        }

        impl const WrappingRem for $T {
            fn wrapping_rem(self, rhs: Self) -> Self {
                (Wrapping(self) % Wrapping(rhs)).0
            }
        }

        impl const WrappingDiv for $T {
            fn wrapping_div(self, rhs: Self) -> Self {
                (Wrapping(self) / Wrapping(rhs)).0
            }
        }
    };
}

wrapping_impl!(u8);
wrapping_impl!(u16);
wrapping_impl!(u32);
wrapping_impl!(u64);
wrapping_impl!(usize);
wrapping_impl!(u128);
wrapping_impl!(i8);
wrapping_impl!(i16);
wrapping_impl!(i32);
wrapping_impl!(i64);
wrapping_impl!(isize);
wrapping_impl!(i128);
