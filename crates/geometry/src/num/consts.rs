
macro_rules! impl_zero {
    ($($ty:ty = $val:expr),*) => {
        $( impl Zero for $ty { const ZERO: Self = $val; } )*
    };
}

macro_rules! impl_one {
    ($($ty:ty = $val:expr),*) => {
        $( impl One for $ty { const ONE: Self = $val; } )*
    };
}

pub const trait Zero {
    const ZERO: Self;
}

pub const trait One {
    const ONE: Self;
}

impl_zero!(u8 = 0, u16 = 0, u32 = 0, u64 = 0, u128 = 0, usize = 0, i8 = 0, i16 = 0, i32 = 0, i64 = 0, i128 = 0, isize = 0, f32 = 0.0, f64 = 0.0);
impl_one!(u8 = 1, u16 = 1, u32 = 1, u64 = 1, u128 = 1, usize = 1, i8 = 1, i16 = 1, i32 = 1, i64 = 1, i128 = 1, isize = 1, f32 = 1.0, f64 = 1.0);
