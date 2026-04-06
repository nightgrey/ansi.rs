use super::{Ops, AssignOps};
use std::str::FromStr;
use crate::{Zero, One};
pub trait Number: Sized + Copy + PartialEq + PartialOrd + Zero + One + Ops + AssignOps + FromStr {}


impl Number for u8 {}
impl Number for u16 {}
impl Number for u32 {}
impl Number for u64 {}
impl Number for u128 {}
impl Number for usize {}
impl Number for i8 {}
impl Number for i16 {}
impl Number for i32 {}
impl Number for i64 {}
impl Number for i128 {}
impl Number for isize {}
impl Number for f32 {}
impl Number for f64 {}
