use super::ops::*;
use crate::{One, Zero};
use std::fmt::Debug;
use std::str::FromStr;

pub const trait Number:
    Sized + Copy + PartialEq + PartialOrd + Zero + One + Ops + AssignOps + FromStr + Debug
{
}

impl const Number for u8 {}
impl const Number for u16 {}
impl const Number for u32 {}
impl const Number for u64 {}
impl const Number for u128 {}
impl const Number for usize {}
impl const Number for i8 {}
impl const Number for i16 {}
impl const Number for i32 {}
impl const Number for i64 {}
impl const Number for i128 {}
impl const Number for isize {}
impl const Number for f32 {}
impl const Number for f64 {}
