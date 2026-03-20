mod space;
mod escape;
mod experimental;

use std::fmt::Debug;
pub use escape::*;
pub use space::*;


use crate::{Escape};
use std::io::Write;
use std::marker::Destruct;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign};
use bilge::prelude::*;

#[derive_const(Clone, Eq, PartialEq, Default)]
#[derive(Copy)]
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
    Reset,
    None,
}

impl Color {
    /// Returns whether the color is not [`Color::None`].
    #[inline]
    pub const fn is_some(&self) -> bool {
        match self {
            Color::None => false,
            _ => true,
        }
    }

    /// Returns whether the color is [`Color::None`].
    #[inline]
    pub const fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns whether the color is [`Color::Reset`].
    #[inline]
    pub const fn is_reset(&self) -> bool {
        match self {
            Color::Reset => true,
            _ => false,
        }
    }

    /// Maps an `Option<T>` to `Option<U>` by applying a function to a contained value (if `Some`) or returns `None` (if `None`).
    ///
    /// # Examples
    ///
    /// Calculates the length of an <code>Option<[String]></code> as an
    /// <code>Option<[usize]></code>, consuming the original:
    ///
    /// [String]: ../../std/string/struct.String.html "String"
    /// ```
    /// let maybe_some_string = Some(String::from("Hello, World!"));
    /// // `Option::map` takes self *by value*, consuming `maybe_some_string`
    /// let maybe_some_len = maybe_some_string.map(|s| s.len());
    /// assert_eq!(maybe_some_len, Some(13));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map(|s| s.len()), None);
    /// ```
    #[inline]
    pub const fn map<U, F>(self, f: F) -> Option<U>
    where
        F: [const] FnOnce(Color) -> U + [const] Destruct,
    {
        match self {
            Color::None => None,
            x => Some(f(x)),
        }
    }

    /// Returns [`Color::None`] if the color is [`Color::None`], otherwise returns `other`.
    ///
    /// Arguments passed to `and` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`and_then`], which is
    /// lazily evaluated.
    #[inline]
    pub const fn and(self, other: Color) -> Color {
       match self {
            Color::None => Color::None,
            _ => other,
        }
    }

    /// Returns [`Color::None`] if the color is [`Color::None`], otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// Some languages call this operation flatmap.
    #[inline]
    pub const fn and_then<U, F: [const] FnOnce(Color) -> U + [const] Destruct>(
        self,
        f: F,
    ) -> Option<U> {
        match self {
            Color::None => None,
            some => Some(f(some)),
        }
    }

    /// Returns the color if it is not [`Color::None`], otherwise returns `other`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`Color::or_else`], which is
    /// lazily evaluated.
    #[inline]
    pub const fn or(self, other: Color) -> Color {
        match self {
            Color::None => other,
            some => some,
        }
    }

    /// Returns the color if it is not [`Color::None`], otherwise calls `f` and
    /// returns the result.
    #[inline]
    pub const fn or_else<F: [const] FnOnce() -> Color + [const] Destruct>(self, f: F) -> Color {
        match self {
            Color::None => f(),
            some => some,
        }
    }

    /// Returns the color if it is not [`Color::None`], otherwise returns [`Color::Reset`].
    #[inline]
    pub const fn or_reset(self) -> Color {
        match self {
            Color::None => Color::Reset,
            some => some,
        }
    }

    /// Returns the color if it is not [`Color::None`], otherwise it's `None`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`Color::or_else`], which is
    /// lazily evaluated.
    #[inline]
    pub const fn or_none(self) -> Color {
        match self {
            Color::None => Color::None,
            some => some,
        }
    }

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
            Color::Reset | Color::None => None,
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
            Color::Reset => f.write_str("Color::Reset"),
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
            (Color::Reset, Color::Reset) => Color::Reset,
            (Color::Reset, _) | (_, Color::Reset) => Color::None,
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
            (Color::Reset, x) | (x, Color::Reset) => x,
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
            (Color::Reset, Color::Reset) => Color::None,
            (Color::Reset, x) | (x, Color::Reset) => x,
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
        match self {
            Color::None => Color::Reset,
            Color::Reset => Color::None,
            _ => Color::Reset,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Attribute;
    use super::*;

        #[test]
        fn test_bitand() {
            for (lhs, rhs, expected) in  [
                (Color::None, Color::None, Color::None),
                (Color::None, Color::Black, Color::None),
                (Color::None, Color::Reset, Color::None),

                (Color::Black, Color::None, Color::None),
                (Color::Black, Color::Black, Color::Black),
                (Color::Black, Color::Reset, Color::None),

                (Color::Reset, Color::None, Color::None),
                (Color::Reset, Color::Black, Color::None),
                (Color::Reset, Color::Reset, Color::Reset),
            ] {
                assert_eq!(lhs.bitand(rhs), expected, "{:?}.bitand({:?})", lhs, rhs);
            }
        }

        #[test]
        fn test_bitor() {
            let a = Attribute::Bold | Attribute::Bold;
            dbg!(a);
            for (lhs, rhs, expected) in  [
                (Color::None, Color::None, Color::None),
                (Color::None, Color::Black, Color::Black),
                (Color::None, Color::Reset, Color::Reset),

                (Color::Black, Color::None, Color::Black),
                (Color::Black, Color::Black, Color::Black),
                (Color::Black, Color::Reset, Color::Black),

                (Color::Reset, Color::None, Color::Reset),
                (Color::Reset, Color::Black, Color::Black),
                (Color::Reset, Color::Reset, Color::Reset),
            ] {
                assert_eq!(lhs.bitor(rhs), expected, "{:?}.bitand({:?})", lhs, rhs);
            }
        }

        #[test]
        fn test_bitxor() {
            for (lhs, rhs, expected) in  [
                (Color::None, Color::None, Color::None),
                (Color::None, Color::Black, Color::Black),
                (Color::None, Color::Reset, Color::Reset),

                (Color::Black, Color::None, Color::Black),
                (Color::Black, Color::Black, Color::None),
                (Color::Black, Color::Reset, Color::Black),

                (Color::Reset, Color::None, Color::Reset),
                (Color::Reset, Color::Black, Color::Black),
                (Color::Reset, Color::Reset, Color::None),
            ] {
                assert_eq!(lhs.bitxor(rhs), expected, "{:?}.bitxor({:?})", lhs, rhs);
            }
        }

        #[test]
        fn test_not() {
            for (value, expected) in [
                (Color::None, Color::Reset),
                (Color::Reset, Color::None),
                (Color::Black, Color::Reset),
            ] {
                assert_eq!(value.not(), expected, "{:?}", value);
            }
        }
}