use std::ops::*;

pub trait CheckedOps<Rhs = Self>: CheckedAdd<Rhs> + CheckedSub<Rhs> + CheckedMul<Rhs> + CheckedDiv<Rhs> + CheckedNeg<Rhs> { }

pub trait CheckedAdd<Rhs = Self>: Sized + Add<Rhs, Output = Self> {
    /// Checked integer addition. Computes `self + rhs`, returning `None`
    /// if overflow occurred.
    fn checked_add(self, rhs: Rhs) -> Option<Self>;
}

/// Performs subtraction that returns `None` instead of wrapping around on underflow.
pub trait CheckedSub<Rhs = Self>: Sized + Sub<Rhs, Output = Self> {
    /// Subtracts two numbers, checking for underflow. If underflow happens,
    /// `None` is returned.
    fn checked_sub(self, rhs: Rhs) -> Option<Self>;
}

/// Performs multiplication that returns `None` instead of wrapping around on underflow or
/// overflow.
pub trait CheckedMul<Rhs = Self>: Sized + Mul<Rhs, Output = Self> {
    /// Multiplies two numbers, checking for underflow or overflow. If underflow
    /// or overflow happens, `None` is returned.
    fn checked_mul(self, rhs: Rhs) -> Option<Self>;
}

/// Performs division that returns `None` instead of panicking on division by zero and instead of
/// wrapping around on underflow and overflow.
pub trait CheckedDiv<Rhs = Self>: Sized + Div<Rhs, Output = Self> {
    /// Divides two numbers, checking for underflow, overflow and division by
    /// zero. If any of that happens, `None` is returned.
    fn checked_div(self, rhs: Rhs) -> Option<Self>;
}

/// Performs an integral remainder that returns `None` instead of panicking on division by zero and
/// instead of wrapping around on underflow and overflow.
pub trait CheckedRem<Rhs = Self>: Sized + Rem<Rhs, Output = Self> {
    /// Finds the remainder of dividing two numbers, checking for underflow, overflow and division
    /// by zero. If any of that happens, `None` is returned.
    fn checked_rem(self, rhs: Rhs) -> Option<Self>;
}

/// Performs negation that returns `None` if the result can't be represented.
pub trait CheckedNeg<Rhs = Self>: Sized {
    /// Negates a number, returning `None` for results that can't be represented, like signed `MIN`
    /// values that can't be positive, or non-zero unsigned values that can't be negative.
    fn checked_neg(self) -> Option<Self>;
}

/// Performs a left shift that returns `None` on shifts larger than
/// or equal to the type width.
pub trait CheckedShl<Rhs = u32>: Sized + Shl<Rhs, Output = Self> {
    /// Checked shift left. Computes `self << rhs`, returning `None`
    /// if `rhs` is larger than or equal to the number of bits in `self`.
    fn checked_shl(self, rhs: Rhs) -> Option<Self>;
}

/// Performs a right shift that returns `None` on shifts larger than
/// or equal to the type width.
pub trait CheckedShr<Rhs = u32>: Sized + Shr<Rhs, Output = Self> {
    /// Checked shift right. Computes `self >> rhs`, returning `None`
    /// if `rhs` is larger than or equal to the number of bits in `self`.
    fn checked_shr(self, rhs: Rhs) -> Option<Self>;
}

macro_rules! checked_impl {
    ($T:ty) => {
        impl CheckedOps for $T {}
        impl CheckedAdd for $T {
            fn checked_add(self, rhs: Self) -> Option<Self> {
                Self::checked_add(self, rhs)
            }
        }

        impl CheckedSub for $T {
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                Self::checked_sub(self, rhs)
            }
        }

        impl CheckedMul for $T {
            fn checked_mul(self, rhs: Self) -> Option<Self> {
                Self::checked_mul(self, rhs)
            }
        }


        impl CheckedDiv for $T {
            fn checked_div(self, rhs: Self) -> Option<Self> {
                Self::checked_div(self, rhs)
            }
        }

        impl CheckedRem for $T {
            fn checked_rem(self, rhs: Self) -> Option<Self> {
                Self::checked_rem(self, rhs)
            }
        }

        impl CheckedNeg for $T {
            fn checked_neg(self) -> Option<Self> {
                Self::checked_neg(self)
            }
        }

        impl CheckedShl for $T {
            fn checked_shl(self, rhs: u32) -> Option<Self> {
                Self::checked_shl(self, rhs)
            }
        }

        impl CheckedShr for $T {
            fn checked_shr(self, rhs: u32) -> Option<Self> {
                Self::checked_shr(self, rhs)
            }
        }
    }
}

checked_impl!(u8);
checked_impl!(u16);
checked_impl!(u32);
checked_impl!(u64);
checked_impl!(usize);
checked_impl!(u128);
checked_impl!(i8);
checked_impl!(i16);
checked_impl!(i32);
checked_impl!(i64);
checked_impl!(isize);
checked_impl!(i128);
