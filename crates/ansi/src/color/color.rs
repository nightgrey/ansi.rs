use std::fmt::Debug;
use crate::{ColorSpace, Escape};
use std::io::Write;
use std::marker::Destruct;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign};
use bilge::prelude::*;
use etwa::Maybe;

#[derive_const(Default, Clone, Eq, PartialEq)]
#[derive(Copy, Maybe)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    Index(u8),
    Rgb(u8, u8, u8),
    #[default]
    None,
}
impl Color {
    /// Returns color if exactly one of `self`, `other` is not [`Color::None`], otherwise returns [`Color::None`].
    #[inline]
    pub const fn xor(self, rhs: Color) -> Color {
        self ^ rhs
    }

    // pub fn convert(self, target: ColorSpace) -> Color {
    //     match target {
    //         ColorSpace::Ansi => Basic::from(self).into(),
    //         ColorSpace::Index => Indexed::from(self).into(),
    //         ColorSpace::Rgb => Rgb::from(self).into(),
    //     }
    // }
    //
    // pub fn clamp(self, max: ColorSpace) -> Color {
    //     if let Some(source) = self.color_space() {
    //         return match max {
    //             ColorSpace::Ansi => match source {
    //                 ColorSpace::Rgb => self.convert(ColorSpace::Ansi),
    //                 ColorSpace::Index => self.convert(ColorSpace::Ansi),
    //                 _ => self,
    //             },
    //             ColorSpace::Index => match source {
    //                 ColorSpace::Rgb => self.convert(ColorSpace::Index),
    //                 _ => self,
    //             },
    //             ColorSpace::Rgb => self,
    //         };
    //     }
    //
    //     self
    // }

    pub fn color_space(&self) -> Option<ColorSpace> {
        match self {
            Color::None => None,
            Color::Index(_) => Some(ColorSpace::Ansi),
            Color::Rgb(_, _, _) => Some(ColorSpace::Rgb),
            _ => None,
        }
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::None => f.write_str("Color::None"),
            Color::Black => f.write_str("Color::Black"),
            Color::Red => f.write_str("Color::Red"),
            Color::Green => f.write_str("Color::Green"),
            Color::Yellow => f.write_str("Color::Yellow"),
            Color::Blue => f.write_str("Color::Blue"),
            Color::Magenta => f.write_str("Color::Magenta"),
            Color::Cyan => f.write_str("Color::Cyan"),
            Color::White => f.write_str("Color::White"),
            Color::BrightBlack => f.write_str("Color::BrightBlack"),
            Color::BrightRed => f.write_str("Color::BrightRed"),
            Color::BrightGreen => f.write_str("Color::BrightGreen"),
            Color::BrightYellow => f.write_str("Color::BrightYellow"),
            Color::BrightBlue => f.write_str("Color::BrightBlue"),
            Color::BrightMagenta => f.write_str("Color::BrightMagenta"),
            Color::BrightCyan => f.write_str("Color::BrightCyan"),
            Color::BrightWhite => f.write_str("Color::BrightWhite"),
            Color::Index(i) => f.debug_tuple("Color::Index").field(i).finish(),
            Color::Rgb(r, g, b) => f.debug_tuple("Color::Rgb").field(r).field(g).field(b).finish(),
        }
    }
}

impl const BitAnd for Color {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Color::None, _) | (_, Color::None) => Color::None,
            (a, b) if a == b => a,
            _ => Color::None,
        }
    }
}

impl const BitAndAssign for Color {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl const BitOr for Color {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Color::None, x) | (x, Color::None) => x,
            (x, _) => x,
        }
    }
}

impl const BitOrAssign for Color {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl const BitXor for Color {
    type Output = Self;

    /// Returns the color if exactly one is non-`None`, otherwise returns `None`.
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Color::None, x) | (x, Color::None) => x,
            _ => Color::None,
        }
    }
}
impl const BitXorAssign for Color {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl const Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self &! rhs
    }
}

impl const SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl const Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        Color::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitand() {
        for (lhs, rhs, expected) in  [
            (Color::None, Color::Black, Color::None),
            (Color::None, Color::None, Color::None),

            (Color::Black, Color::None, Color::None),
            (Color::Black, Color::Black, Color::Black),

            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::None),
        ] {
            assert_eq!(lhs.bitand(rhs), expected, "{:?}.bitand({:?})", lhs, rhs);
        }
    }

    #[test]
    fn test_bitor() {
        for (lhs, rhs, expected) in  [
            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::Black),

            (Color::Black, Color::None, Color::Black),
            (Color::Black, Color::Black, Color::Black),

            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::Black),
        ] {
            assert_eq!(lhs.bitor(rhs), expected, "{:?}.bitand({:?})", lhs, rhs);
        }
    }

    #[test]
    fn test_bitxor() {
        for (lhs, rhs, expected) in  [
            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::Black),

            (Color::Black, Color::None, Color::Black),
            (Color::Black, Color::Black, Color::None),

            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::Black),
        ] {
            assert_eq!(lhs.bitxor(rhs), expected, "{:?}.bitxor({:?})", lhs, rhs);
        }
    }

    #[test]
    fn test_not() {
        for (value, expected) in [
            (Color::None, Color::None),
            (Color::None, Color::None),
            (Color::Black, Color::None),
        ] {
            assert_eq!(value.not(), expected, "{:?}", value);
        }
    }

}