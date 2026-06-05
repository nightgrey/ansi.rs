use std::fmt::Debug;
use std::marker::Destruct;
use std::ops::{BitAnd, BitOr, BitXor, Not};
use crate::Bit;

pub const trait Base: Sized
{
    /// The unsigned integer that stores the bits.
    type Repr: Sized
    + [ const ] Destruct
    + Copy
    + [ const ] Default
    + [ const ] PartialEq
    + [ const ] Eq
    + [ const ] PartialOrd
    + Debug
    + [ const ] Ord
    + [ const ] BitAnd<Self::Repr, Output=Self::Repr>
    + [ const ] BitOr<Self::Repr, Output=Self::Repr>
    + [ const ] BitXor<Self::Repr, Output=Self::Repr>
    + [ const ] Not<Output=Self::Repr>;

    fn from_repr(repr: Self::Repr) -> Self;
    fn into_repr(self) -> Self::Repr;

    fn bits(self) -> Self::Repr {
        self.into_repr()
    }
}
