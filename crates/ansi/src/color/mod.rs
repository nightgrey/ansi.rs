mod space;
mod escape;
mod experimental;

pub use escape::*;
pub use space::*;


use crate::{Escape};
use std::io::Write;
use std::marker::Destruct;


#[derive(Clone, Copy, Eq, PartialEq, Default, Hash, Debug)]
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
    Default,

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

    /// Returns whether the color is [`Color::Default`].
    #[inline]
    pub const fn is_default(&self) -> bool {
        match self {
            Color::Default => true,
            _ => false,
        }
    }

    /// Converts color to [`Color::Default`].
    pub fn default(mut self) -> Self {
        Color::Default
    }

    /// Converts color to [`Color::None`].
    pub fn none(mut self) -> Self {
        Color::None
    }

    /// Returns `Some(self)` if the color is not [`Color::None`], otherwise returns `None`.
    pub const fn as_option(&self) -> Option<&Color> {
        match self {
            Color::None => None,
            color => Some(color),
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
            _ => Some(f(self)),
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
            x @ _ => x,
            Color::None => other,
        }
    }

    /// Returns the color if it is not [`Color::None`], otherwise returns [`Color::Default`].
    #[inline]
    pub const fn or_default(self) -> Color {
        match self {
            x @ _ => x,
            Color::None => Color::Default,
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
            x @ _ => x,
            Color::None => Color::None,
        }
    }

    /// Returns the color if it is not [`Color::None`], otherwise calls `f` and
    /// returns the result.
    #[inline]
    pub const fn or_else<F: [const] FnOnce() -> Color + [const] Destruct>(self, f: F) -> Color {
        match self {
            Color::None => f(),
            x @ _ => x,
        }
    }

    /// Returns color if exactly one of `self`, `other` is not [`Color::None`], otherwise returns [`Color::None`].
    #[inline]
    pub const fn xor(self, other: Color) -> Color {
        match (self, other) {
            (a @ _, Color::None) => a,
            (Color::None, b @ _) => b,
            _ => Color::None,
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
            Color::Default | Color::None => None,
            Color::Index(_) => Some(ColorSpace::Ansi),
            Color::Rgb(_, _, _) => Some(ColorSpace::Rgb),
            _ => None,
        }
    }
}
