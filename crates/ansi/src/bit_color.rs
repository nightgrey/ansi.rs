use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[bits = 2]
enum ColorKindBits {
    None,
    Default,
    Index,
    Rgb,
}

#[bitfield]
#[derive(Specifier)]
#[repr(u32)]
struct RgbBits {
    #[skip]
    _reserved: B8,
    r: B8,
    g: B8,
    b: B8,
}

#[bitfield]
#[derive(Specifier)]
#[repr(u32)]
struct ColorBits {
    #[skip]
    _reserved: B6,
    tag: ColorKindBits,
    value: B24,
}

#[bitfield]
#[derive(Specifier)]
#[repr(u64)]
struct BitColors {
    foreground: ColorBits,
    background: ColorBits,
}
