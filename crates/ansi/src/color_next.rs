use crate::Color;
use bilge::prelude::*;
use utils::{Unpack, Pack};

#[bitsize(2)]
#[derive(Default, FromBits)]
enum ColorSpace {
    #[default]
    None = 0,
    Default = 1,
    Ansi = 2,
    Rgb = 3,
}

#[bitsize(32)]
#[derive(DefaultBits, FromBits)]
struct ColorBits {
    space: ColorSpace,
    value: u30,
}

impl Unpack<ColorBits> for Color {
    #[inline]
    fn unpack(packed: ColorBits) -> Self {
        match packed.space() {
            ColorSpace::None => Color::None,
            ColorSpace::Default => Color::Default,
            ColorSpace::Ansi => Color::Index(packed.value().as_u8()),
            ColorSpace::Rgb => {
                let value = packed.value.as_u8();
                Color::Rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
            }
        }
    }
}

impl Pack<ColorBits> for Color {
    #[inline]
    fn pack(self) -> ColorBits {
        match self {
            Color::None => ColorBits::new(ColorSpace::None, u30::ZERO),
            Color::Default => ColorBits::new(ColorSpace::Default, u30::ZERO),
            Color::Black => ColorBits::new(ColorSpace::Ansi, u30::new(0)),
            Color::Red => ColorBits::new(ColorSpace::Ansi, u30::new(1)),
            Color::Green => ColorBits::new(ColorSpace::Ansi, u30::new(2)),
            Color::Yellow => ColorBits::new(ColorSpace::Ansi, u30::new(3)),
            Color::Blue => ColorBits::new(ColorSpace::Ansi, u30::new(4)),
            Color::Magenta => ColorBits::new(ColorSpace::Ansi, u30::new(5)),
            Color::Cyan => ColorBits::new(ColorSpace::Ansi, u30::new(6)),
            Color::White => ColorBits::new(ColorSpace::Ansi, u30::new(7)),
            Color::BrightBlack => ColorBits::new(ColorSpace::Ansi, u30::new(8)),
            Color::BrightRed => ColorBits::new(ColorSpace::Ansi, u30::new(9)),
            Color::BrightGreen => ColorBits::new(ColorSpace::Ansi, u30::new(10)),
            Color::BrightYellow => ColorBits::new(ColorSpace::Ansi, u30::new(11)),
            Color::BrightBlue => ColorBits::new(ColorSpace::Ansi, u30::new(12)),
            Color::BrightMagenta => ColorBits::new(ColorSpace::Ansi, u30::new(13)),
            Color::BrightCyan => ColorBits::new(ColorSpace::Ansi, u30::new(14)),
            Color::BrightWhite => ColorBits::new(ColorSpace::Ansi, u30::new(15)),
            Color::Index(index) => ColorBits::new(ColorSpace::Ansi, u30::from(index)),
            Color::Rgb(r, g, b) => ColorBits::new(
                ColorSpace::Rgb,
                u30::new(((r as u32) << 16) | ((g as u32) << 8) | (b as u32)),
            ),
        }

    }
}
