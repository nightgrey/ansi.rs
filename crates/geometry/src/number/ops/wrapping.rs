use std::ops::{Add, Div, Mul, Rem, Sub};

pub trait WrappingOps<Rhs = Self>: WrappingAdd<Rhs> + WrappingSub<Rhs> + WrappingMul<Rhs> + WrappingDiv<Rhs> + WrappingRem<Rhs>  { }

pub trait WrappingAdd<Rhs = Self>: Sized + Add<Rhs, Output=Self> {
    fn wrapping_add(self, other: Rhs) -> Self;
}

pub trait WrappingSub<Rhs = Self>: Sized + Sub<Self, Output=Self> {
    fn wrapping_sub(self, other: Rhs) -> Self;
}

pub trait WrappingMul<Rhs = Self>: Sized + Mul<Self, Output=Self> {
    fn wrapping_mul(self, other: Rhs) -> Self;
}

pub trait WrappingDiv <Rhs = Self>: Sized + Div<Self, Output=Self> {
    fn wrapping_div(self, other: Rhs) -> Self;
}

pub trait WrappingRem<Rhs = Self>: Sized + Rem<Self, Output=Self> {
    fn wrapping_rem(self, other: Rhs) -> Self;
}

macro_rules! wrapping_impl {
    ($T:ty) => {
        impl WrappingOps for $T {}
        
        impl WrappingAdd for $T {
            fn wrapping_add(self, rhs: Self) -> Self {
                Self::wrapping_add(self, rhs)
            }
        }

        impl WrappingSub for $T {
            fn wrapping_sub(self, rhs: Self) -> Self {
                Self::wrapping_sub(self, rhs)
            }
        }

        impl WrappingMul for $T {
            fn wrapping_mul(self, rhs: Self) -> Self {
                Self::wrapping_mul(self, rhs)
            }
        }

        impl WrappingRem for $T {
            fn wrapping_rem(self, rhs: Self) -> Self {
                Self::wrapping_rem(self, rhs)
            }
        }

        impl WrappingDiv for $T {
            fn wrapping_div(self, rhs: Self) -> Self {
                Self::wrapping_div(self, rhs)
            }
        }
        
    }
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
