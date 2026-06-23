use super::ops::*;
use crate::{One, Zero};
use std::fmt::Debug;
use std::str::FromStr;

pub const trait Number:
    Sized
    + Copy
    + [const] Default
    + [const] PartialEq
    + [const] PartialOrd
    + [const] Zero
    + [const] One
    + [const] Ops
    + [const] AssignOps
    + FromStr
    + Debug
{
}

const impl Number for u8 {}
const impl Number for u16 {}
const impl Number for u32 {}
const impl Number for u64 {}
const impl Number for u128 {}
const impl Number for usize {}
const impl Number for i8 {}
const impl Number for i16 {}
const impl Number for i32 {}
const impl Number for i64 {}
const impl Number for i128 {}
const impl Number for isize {}
const impl Number for f32 {}
const impl Number for f64 {}
