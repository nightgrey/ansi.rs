pub const trait Zero {
    const ZERO: Self;
}

pub const trait One {
    const ONE: Self;
}

pub const trait Min {
    const MIN: Self;
}

pub const trait Max {
    const MAX: Self;
}

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

macro_rules! impl_min {
    ($($ty:ty = $val:expr),*) => {
        $( impl Min for $ty { const MIN: Self = $val; } )*
    };
}

macro_rules! impl_max {
    ($($ty:ty = $val:expr),*) => {
        $( impl Max for $ty { const MAX: Self = $val; } )*
    };
}

impl_zero!(
    u8 = 0,
    u16 = 0,
    u32 = 0,
    u64 = 0,
    u128 = 0,
    usize = 0,
    i8 = 0,
    i16 = 0,
    i32 = 0,
    i64 = 0,
    i128 = 0,
    isize = 0,
    f32 = 0.0,
    f64 = 0.0
);
impl_one!(
    u8 = 1,
    u16 = 1,
    u32 = 1,
    u64 = 1,
    u128 = 1,
    usize = 1,
    i8 = 1,
    i16 = 1,
    i32 = 1,
    i64 = 1,
    i128 = 1,
    isize = 1,
    f32 = 1.0,
    f64 = 1.0
);
impl_min!(
    u8 = u8::MIN,
    u16 = u16::MIN,
    u32 = u32::MIN,
    u64 = u64::MIN,
    u128 = u128::MIN,
    usize = usize::MIN,
    i8 = i8::MIN,
    i16 = i16::MIN,
    i32 = i32::MIN,
    i64 = i64::MIN,
    i128 = i128::MIN,
    isize = isize::MIN,
    f32 = f32::MIN,
    f64 = f64::MIN
);
impl_max!(
    u8 = u8::MAX,
    u16 = u16::MAX,
    u32 = u32::MAX,
    u64 = u64::MAX,
    u128 = u128::MAX,
    usize = usize::MAX,
    i8 = i8::MAX,
    i16 = i16::MAX,
    i32 = i32::MAX,
    i64 = i64::MAX,
    i128 = i128::MAX,
    isize = isize::MAX,
    f32 = f32::MAX,
    f64 = f64::MAX
);
