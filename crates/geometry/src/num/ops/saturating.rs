use std::ops::*;

pub trait SaturatingOps<Rhs = Self>: SaturatingAdd<Rhs> + SaturatingSub<Rhs> + SaturatingMul<Rhs> + SaturatingDiv<Rhs>  { }

pub trait SaturatingAdd<Rhs = Self>: Sized + Add<Rhs, Output=Self> {
    fn saturating_add(self, other: Rhs) -> Self;
}

pub trait SaturatingSub<Rhs = Self>: Sized + Sub<Self, Output=Self> {
    fn saturating_sub(self, other: Rhs) -> Self;
}

pub trait SaturatingDiv <Rhs = Self>: Sized + Div<Self, Output=Self> {
    fn saturating_div(self, other: Rhs) -> Self;
}

pub trait SaturatingMul<Rhs = Self>: Sized + Mul<Self, Output=Self> {
    fn saturating_mul(self, other: Rhs) -> Self;
}

macro_rules! saturating_impl {
    ($T:ty) => {
        impl SaturatingOps for $T {}
        
        impl SaturatingAdd for $T {
            fn saturating_add(self, rhs: Self) -> Self {
                Self::saturating_add(self, rhs)
            }
        }

        impl SaturatingSub for $T {
            fn saturating_sub(self, rhs: Self) -> Self {
                Self::saturating_sub(self, rhs)
            }
        }

        impl SaturatingMul for $T {
            fn saturating_mul(self, rhs: Self) -> Self {
                Self::saturating_mul(self, rhs)
            }
        }


        impl SaturatingDiv for $T {
            fn saturating_div(self, rhs: Self) -> Self {
                Self::saturating_div(self, rhs)
            }
        }
    }
}

saturating_impl!(u8);
saturating_impl!(u16);
saturating_impl!(u32);
saturating_impl!(u64);
saturating_impl!(usize);
saturating_impl!(u128);
saturating_impl!(i8);
saturating_impl!(i16);
saturating_impl!(i32);
saturating_impl!(i64);
saturating_impl!(isize);
saturating_impl!(i128);
