use crate::Escape;
use derive_more::{Deref, DerefMut, From, Into};
use std::io::Write;
use std::marker::Destruct;

#[derive(Clone, Copy, Eq, PartialEq, Default, Hash, Debug)]
pub enum Color {
    None,
    #[default]
    Default,
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
    Mono(bool),
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
    //         ColorSpace::Mono => Mono::from(self).into(),
    //         ColorSpace::Rgb => Rgb::from(self).into(),
    //     }
    // }
    //
    // pub fn clamp(self, max: ColorSpace) -> Color {
    //     if let Some(source) = self.color_space() {
    //         return match max {
    //             ColorSpace::Mono => match source {
    //                 ColorSpace::Mono => self,
    //                 _ => self.convert(ColorSpace::Mono),
    //             },
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
            Color::Index(_) => Some(ColorSpace::Index),
            Color::Rgb(_, _, _) => Some(ColorSpace::Rgb),
            Color::Mono(_) => Some(ColorSpace::Mono),
            _ => Some(ColorSpace::Ansi),
        }
    }
}

/// The color space.
///
/// Defines the color space of a color.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColorSpace {
    Mono,
    Ansi,
    Index,
    Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Into)]
#[repr(transparent)]
pub struct Background<'a>(pub &'a Color);

impl Background<'_> {
    pub fn as_foreground(&self) -> Foreground<'_> {
        Foreground(self.0)
    }

    pub fn as_underline(&self) -> Underline<'_> {
        Underline(self.0)
    }
}

impl AsRef<Color> for Background<'_> {
    fn as_ref(&self) -> &Color {
        &self.0
    }
}

impl Into<Color> for Background<'_> {
    fn into(self) -> Color {
        *self.0
    }
}

impl Escape for Background<'_> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use Color::*;

        match self.as_ref() {
            None => Ok(()),
            Default => w.write_all(b"49"),
            Black => w.write_all(b"40"),
            Red => w.write_all(b"41"),
            Green => w.write_all(b"42"),
            Yellow => w.write_all(b"43"),
            Blue => w.write_all(b"44"),
            Magenta => w.write_all(b"45"),
            Cyan => w.write_all(b"46"),
            White => w.write_all(b"47"),
            BrightBlack => w.write_all(b"90"),
            BrightRed => w.write_all(b"91"),
            BrightGreen => w.write_all(b"92"),
            BrightYellow => w.write_all(b"93"),
            BrightBlue => w.write_all(b"94"),
            BrightMagenta => w.write_all(b"95"),
            BrightCyan => w.write_all(b"96"),
            BrightWhite => w.write_all(b"97"),
            Mono(mono) => match mono {
                true => w.write_all(b"37"),
                false => w.write_all(b"40"),
            },
            Index(index) => {
                write!(w, "38;5;{}", index)
            }
            Rgb(r, g, b) => {
                write!(w, "38;2;{};{};{}", r, g, b)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Into)]
#[repr(transparent)]
pub struct Foreground<'a>(pub &'a Color);

impl Foreground<'_> {
    pub fn as_background(&self) -> Background<'_> {
        Background(self.0)
    }

    pub fn as_underline(&self) -> Underline<'_> {
        Underline(self.0)
    }
}
impl AsRef<Color> for Foreground<'_> {
    fn as_ref(&self) -> &Color {
        &self.0
    }
}

impl Into<Color> for Foreground<'_> {
    fn into(self) -> Color {
        *self.0
    }
}

impl Escape for Foreground<'_> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use Color::*;

        match self.0 {
            None => Ok(()),
            Default => w.write_all(b"39"),
            Mono(mono) => match mono {
                true => w.write_all(b"37"),
                false => w.write_all(b"90"),
            },
            Black => w.write_all(b"30"),
            Red => w.write_all(b"31"),
            Green => w.write_all(b"32"),
            Yellow => w.write_all(b"33"),
            Blue => w.write_all(b"34"),
            Magenta => w.write_all(b"35"),
            Cyan => w.write_all(b"36"),
            White => w.write_all(b"37"),
            BrightBlack => w.write_all(b"90"),
            BrightRed => w.write_all(b"91"),
            BrightGreen => w.write_all(b"92"),
            BrightYellow => w.write_all(b"93"),
            BrightBlue => w.write_all(b"94"),
            BrightMagenta => w.write_all(b"95"),
            BrightCyan => w.write_all(b"96"),
            BrightWhite => w.write_all(b"97"),
            Index(i) => {
                write!(w, "38;5;{}", i)
            }
            Rgb(r, g, b) => {
                write!(w, "38;2;{};{};{}", r, g, b)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Into)]
#[repr(transparent)]
pub struct Underline<'a>(pub &'a Color);

impl Underline<'_> {
    pub fn as_background(&self) -> Background<'_> {
        Background(self.0)
    }

    pub fn as_foreground(&self) -> Foreground<'_> {
        Foreground(self.0)
    }
}

impl Escape for Underline<'_> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use Color::*;
        match self.0 {
            None => Ok(()),
            Default => w.write_all(b"59"),
            Mono(is_mono) => w.write_all(if *is_mono { b"58;5;15" } else { b"58;5;0" }),
            Black => w.write_all(b"58;5;0"),
            Red => w.write_all(b"58;5;1"),
            Green => w.write_all(b"58;5;2"),
            Yellow => w.write_all(b"58;5;3"),
            Blue => w.write_all(b"58;5;4"),
            Magenta => w.write_all(b"58;5;5"),
            Cyan => w.write_all(b"58;5;6"),
            White => w.write_all(b"58;5;7"),
            BrightBlack => w.write_all(b"58;5;8"),
            BrightRed => w.write_all(b"58;5;9"),
            BrightGreen => w.write_all(b"58;5;10"),
            BrightYellow => w.write_all(b"58;5;11"),
            BrightBlue => w.write_all(b"58;5;12"),
            BrightMagenta => w.write_all(b"58;5;13"),
            BrightCyan => w.write_all(b"58;5;14"),
            BrightWhite => w.write_all(b"58;5;15"),
            Index(i) => {
                write!(w, "58;5;{}", i)
            }
            Rgb(r, g, b) => {
                write!(w, "58;2;{};{};{}", r, g, b)
            }
        }
    }
}

impl AsRef<Color> for Underline<'_> {
    fn as_ref(&self) -> &Color {
        &self.0
    }
}

impl Into<Color> for Underline<'_> {
    fn into(self) -> Color {
        *self.0
    }
}

impl Color {
    pub fn as_background(&self) -> Background<'_> {
        Background(self)
    }

    pub fn as_foreground(&self) -> Foreground<'_> {
        Foreground(self)
    }

    pub fn as_underline(&self) -> Underline<'_> {
        Underline(self)
    }
}
