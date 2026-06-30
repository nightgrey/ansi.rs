use crate::ColorSpace;
use maybe::{Maybe, MaybeConst};
use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};

#[derive_const(Eq, Clone, PartialEq, MaybeConst)]
#[derive(Copy, Default)]
#[repr(u8)]
pub enum Color {
    #[default]
    None,
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
}
const impl Color {
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        match (self, other) {
            (Color::None, _) | (_, Color::None) => Color::None,
            (a, b) if a == b => a,
            _ => Color::None,
        }
    }

    #[inline]
    #[must_use]
    pub fn difference(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Color::None, _) => Color::None,
            (x, Color::None) => x,
            (_a, _b) => Color::None,
        }
    }
    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (x, Color::None) | (Color::None, x) => x,
            (_, x) => x,
        }
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
const impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Color::Rgb(
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        )
    }
}

const impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Color::Rgb(value.0, value.1, value.2)
    }
}

const impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            8 => Color::BrightBlack,
            9 => Color::BrightRed,
            10 => Color::BrightGreen,
            11 => Color::BrightYellow,
            12 => Color::BrightBlue,
            13 => Color::BrightMagenta,
            14 => Color::BrightCyan,
            15 => Color::BrightWhite,
            16..=255 => Color::Index(value),
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
            Color::Rgb(r, g, b) => f
                .debug_tuple("Color::Rgb")
                .field(r)
                .field(g)
                .field(b)
                .finish(),
        }
    }
}

const impl BitAnd for Color {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

const impl BitAndAssign for Color {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

const impl BitOr for Color {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

const impl BitOrAssign for Color {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

const impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

const impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        for (lhs, rhs, expected) in [
            (Color::None, Color::Black, Color::None),
            (Color::None, Color::None, Color::None),
            (Color::Black, Color::None, Color::None),
            (Color::Black, Color::Black, Color::Black),
        ] {
            assert_eq!(
                lhs.intersection(rhs),
                expected,
                "{:?}.intersection({:?})",
                lhs,
                rhs
            );
        }
    }

    #[test]
    fn test_union() {
        for (lhs, rhs, expected) in [
            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::Black),
            (Color::Black, Color::None, Color::Black),
            (Color::Black, Color::Black, Color::Black),
        ] {
            assert_eq!(lhs.union(rhs), expected, "{:?}.union({:?})", lhs, rhs);
        }
    }

    #[test]
    fn test_difference() {
        for (lhs, rhs, expected) in [
            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::None),
            (Color::Black, Color::None, Color::Black),
            (Color::Black, Color::Blue, Color::None),
        ] {
            assert_eq!(
                lhs.difference(rhs),
                expected,
                "{:?}.difference({:?})",
                lhs,
                rhs
            );
        }
    }
}
