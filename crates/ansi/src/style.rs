use crate::attribute::Attribute;
use crate::color::Color;

use crate::Escape;
use bitflags::Flags;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXorAssign, Sub, SubAssign};
use utils::separator;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Style {
    pub attributes: Attribute,
    pub fg: Color,
    pub bg: Color,
    pub ul: Color,
}

#[allow(non_upper_case_globals)]
impl Style {
    pub const EMPTY: Style = Self {
        attributes: Attribute::EMPTY,
        fg: Color::None,
        bg: Color::None,
        ul: Color::None,
    };

    pub const Reset: Self = Style {
        attributes: Attribute::Reset,
        fg: Color::Default,
        bg: Color::Default,
        ul: Color::Default,
    };

    pub const Bold: Self = Style {
        attributes: Attribute::Bold,
        ..Self::EMPTY
    };

    pub const Faint: Self = Style {
        attributes: Attribute::Faint,
        ..Self::EMPTY
    };

    pub const Italic: Self = Style {
        attributes: Attribute::Italic,
        ..Self::EMPTY
    };

    pub const Underline: Self = Style {
        attributes: Attribute::Underline,
        ..Self::EMPTY
    };

    pub const Blink: Self = Style {
        attributes: Attribute::Blink,
        ..Self::EMPTY
    };

    pub const RapidBlink: Self = Style {
        attributes: Attribute::RapidBlink,
        ..Self::EMPTY
    };

    pub const Reverse: Self = Style {
        attributes: Attribute::Reverse,
        ..Self::EMPTY
    };

    pub const Conceal: Self = Style {
        attributes: Attribute::Conceal,
        ..Self::EMPTY
    };

    pub const Strikethrough: Self = Style {
        attributes: Attribute::Strikethrough,
        ..Self::EMPTY
    };
    pub const UnderlineStyleNone: Self = Style {
        attributes: Attribute::UnderlineStyleNone,
        ..Self::EMPTY
    };

    pub const UnderlineStyleSingle: Self = Style {
        attributes: Attribute::Underline.union(Attribute::UnderlineStyleSingle),
        ..Self::EMPTY
    };

    pub const UnderlineStyleDouble: Self = Style {
        attributes: Attribute::Underline.union(Attribute::UnderlineStyleDouble),
        ..Self::EMPTY
    };

    pub const UnderlineStyleCurly: Self = Style {
        attributes: Attribute::Underline.union(Attribute::UnderlineStyleCurly),
        ..Self::EMPTY
    };

    pub const UnderlineStyleDotted: Self = Style {
        attributes: Attribute::Underline.union(Attribute::UnderlineStyleDotted),
        ..Self::EMPTY
    };

    pub const UnderlineStyleDashed: Self = Style {
        attributes: Attribute::Underline.union(Attribute::UnderlineStyleDashed),
        ..Self::EMPTY
    };

    pub const NoBold: Self = Style {
        attributes: Attribute::NoBold,
        ..Self::EMPTY
    };

    pub const NormalIntensity: Self = Style {
        attributes: Attribute::NormalIntensity,
        ..Self::EMPTY
    };

    pub const NoItalic: Self = Style {
        attributes: Attribute::NoItalic,
        ..Self::EMPTY
    };

    pub const NoUnderline: Self = Style {
        attributes: Attribute::NoUnderline,
        ..Self::EMPTY
    };

    pub const NoBlink: Self = Style {
        attributes: Attribute::NoBlink,
        ..Self::EMPTY
    };

    pub const NoReverse: Self = Style {
        attributes: Attribute::NoReverse,
        ..Self::EMPTY
    };

    pub const NoConceal: Self = Style {
        attributes: Attribute::NoConceal,
        ..Self::EMPTY
    };

    pub const NoStrikethrough: Self = Style {
        attributes: Attribute::NoStrikethrough,
        ..Self::EMPTY
    };

    pub const Frame: Self = Style {
        attributes: Attribute::Frame,
        ..Self::EMPTY
    };

    pub const Encircle: Self = Style {
        attributes: Attribute::Encircle,
        ..Self::EMPTY
    };

    pub const Overline: Self = Style {
        attributes: Attribute::Overline,
        ..Self::EMPTY
    };

    pub const NoFrameOrEncircle: Self = Style {
        attributes: Attribute::NoFrameOrEncircle,
        ..Self::EMPTY
    };

    pub const NoOverline: Self = Style {
        attributes: Attribute::NoOverline,
        ..Self::EMPTY
    };

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub const fn new() -> Self {
        Self::EMPTY
    }

    pub const fn from_attribute(attribute: Attribute) -> Self {
        Self {
            attributes: attribute,
            fg: Color::None,
            bg: Color::None,
            ul: Color::None,
        }
    }

    /// Returns `true` if the given attribute flag is currently set.
    ///
    /// # Example
    /// ```
    /// use ansi::{Attribute, Style};
    /// let s = Style::new().bold();
    /// assert!(s.contains(Attribute::Bold));
    /// ```
    #[inline]
    pub fn contains(&self, attribute: Attribute) -> bool {
        self.attributes.contains(attribute)
    }

    /// Set additional attribute flags.
    ///
    /// Inverse flags (e.g. `NoBold`) automatically cancel their positive
    /// counterpart.
    pub fn set(&mut self, attributes: Attribute) {
        self.attributes.insert_inclusive(attributes);
    }

    /// Remove attribute flags.
    #[inline]
    pub fn remove(&mut self, attributes: Attribute) {
        self.attributes.remove(attributes);
    }

    /// Set attribute flags.
    pub fn attributes(mut self, attributes: Attribute) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set the background color.
    #[inline]
    pub const fn background(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    pub const fn foreground(mut self, color: Color) -> Self {
        self.fg = color;
        self
    }

    pub const fn underline_color(mut self, color: Color) -> Self {
        self.ul = color;
        self
    }

    /// Sets [`Attribute::Reset`].
    pub fn reset(mut self) -> Self {
        self.set(Attribute::Reset);
        self
    }

    /// Sets [`Attribute::Bold`].
    #[inline]
    pub fn bold(mut self) -> Self {
        self.set(Attribute::Bold);
        self
    }

    /// Sets [`Attribute::Faint`].
    #[inline]
    pub fn faint(mut self) -> Self {
        self.set(Attribute::Faint);
        self
    }

    /// Sets [`Attribute::Italic`].
    #[inline]
    pub fn italic(mut self) -> Self {
        self.set(Attribute::Italic);
        self
    }

    /// Sets [`Attribute::Underline`].
    #[inline]
    pub fn underline(mut self) -> Self {
        self.set(Attribute::Underline);
        self
    }

    /// Sets the underline style
    #[inline]
    pub fn underline_style(mut self, underline_style: UnderlineStyle) -> Self {
        self.set(underline_style.into());
        self
    }

    /// Returns the current underline style.
    #[inline]
    pub fn get_underline_style(&self) -> UnderlineStyle {
        let attributes = self.attributes;

        if attributes.contains(Attribute::UnderlineStyleNone) {
            UnderlineStyle::None
        } else if attributes.contains(Attribute::UnderlineStyleSingle) {
            UnderlineStyle::Single
        } else if attributes.contains(Attribute::UnderlineStyleDouble) {
            UnderlineStyle::Double
        } else if attributes.contains(Attribute::UnderlineStyleCurly) {
            UnderlineStyle::Curly
        } else if attributes.contains(Attribute::UnderlineStyleDotted) {
            UnderlineStyle::Dotted
        } else if attributes.contains(Attribute::UnderlineStyleDashed) {
            UnderlineStyle::Dashed
        } else if attributes.contains(Attribute::Underline) {
            UnderlineStyle::Single
        } else {
            UnderlineStyle::None
        }
    }

    /// Sets [`Attribute::Blink`].
    #[inline]
    pub fn blink(mut self) -> Self {
        self.set(Attribute::Blink);
        self
    }

    /// Sets [`Attribute::RapidBlink`].
    #[inline]
    pub fn rapid_blink(mut self) -> Self {
        self.set(Attribute::RapidBlink);
        self
    }

    /// Sets [`Attribute::Reverse`].
    #[inline]
    pub fn reverse(mut self) -> Self {
        self.set(Attribute::Reverse);
        self
    }

    /// Sets [`Attribute::Conceal`].
    #[inline]
    pub fn conceal(mut self) -> Self {
        self.set(Attribute::Conceal);
        self
    }

    /// Sets [`Attribute::Strikethrough`].
    #[inline]
    pub fn strikethrough(mut self) -> Self {
        self.set(Attribute::Strikethrough);
        self
    }

    /// Sets [`Attribute::Frame`].
    #[inline]
    pub fn frame(mut self) -> Self {
        self.set(Attribute::Frame);
        self
    }

    /// Sets [`Attribute::Encircle`].
    #[inline]
    pub fn encircle(mut self) -> Self {
        self.set(Attribute::Encircle);
        self
    }

    /// Sets [`Attribute::Overline`].
    #[inline]
    pub fn overline(mut self) -> Self {
        self.set(Attribute::Overline);
        self
    }

    /// Unsets: [`Attribute::Reset`]
    pub fn no_reset(mut self) -> Self {
        self.remove(Attribute::Reset);
        self
    }

    /// Sets [`Attribute::NoBold`].
    ///
    /// Unsets: [`Attribute::Bold`]
    #[inline]
    pub fn no_bold(mut self) -> Self {
        self.remove(Attribute::NoBold);
        self
    }

    /// Sets [`Attribute::NormalIntensity`].
    ///
    /// Unsets: [`Attribute::Bold`]
    #[inline]
    pub fn normal_intensity(mut self) -> Self {
        self.remove(Attribute::NormalIntensity);
        self
    }

    /// Sets [`Attribute::NoItalic`].
    ///
    /// Unsets: [`Attribute::Italic`]
    #[inline]
    pub fn no_italic(mut self) -> Self {
        self.remove(Attribute::NoItalic);
        self
    }

    /// Sets [`Attribute::NoUnderline`].
    ///
    /// Unsets: [`Attributes::UnderlineStyleNone`] [`Attributes::UnderlineStyleSingle`] [`Attributes::UnderlineStyleDouble`] [`Attributes::UnderlineStyleCurly`] [`Attributes::UnderlineStyleDotted`] [`Attributes::UnderlineStyleDashed`]
    #[inline]
    pub fn no_underline(mut self) -> Self {
        self.remove(Attribute::NoUnderline);
        self
    }

    /// Sets [`Attribute::NoBlink`].
    ///
    /// Unsets: [`Attribute::Blink`]
    #[inline]
    pub fn no_blink(mut self) -> Self {
        self.remove(Attribute::NoBlink);
        self
    }

    /// Sets [`Attribute::NoReverse`].
    ///
    /// Unsets: [`Attribute::Reverse`]
    #[inline]
    pub fn no_reverse(mut self) -> Self {
        self.remove(Attribute::NoReverse);
        self
    }

    /// Sets [`Attribute::NoConceal`].
    ///
    /// Unsets: [`Attribute::Conceal`]
    #[inline]
    pub fn no_conceal(mut self) -> Self {
        self.remove(Attribute::NoConceal);
        self
    }

    /// Sets [`Attribute::NoStrikethrough`].
    ///
    /// Unsets: [`Attribute::Strikethrough`]
    #[inline]
    pub fn no_strikethrough(mut self) -> Self {
        self.remove(Attribute::NoStrikethrough);
        self
    }

    /// Sets [`Attribute::NoFrameOrEncircle`].
    ///
    /// Unsets: [`Attribute::Encircle`]
    #[inline]
    pub fn no_frame_or_encircle(mut self) -> Self {
        self.remove(Attribute::NoFrameOrEncircle);
        self
    }

    /// Sets [`Attribute::NoOverline`].
    ///
    /// Unsets: [`Attribute::Overline`]
    #[inline]
    pub fn no_overline(mut self) -> Self {
        self.remove(Attribute::NoOverline);
        self
    }

    #[inline]
    pub fn bitand(self, other: Style) -> Self {
        let mut style = self;

        style.bitand_assign(other);

        style
    }

    #[inline]
    fn bitand_assign(&mut self, other: Style) {
        self.attributes.bitand_assign(other.attributes);

        self.fg = self.fg.and(other.fg);
        self.bg = self.bg.and(other.bg);
        self.ul = self.ul.and(other.ul);
    }

    #[inline]
    fn bitor(self, other: Style) -> Self {
        let mut style = self;

        style.bitor_assign(other);

        style
    }

    #[inline]
    fn bitor_assign(&mut self, other: Style) {
        self.attributes.bitor_assign(other.attributes);

        self.fg = self.fg.or(other.fg);
        self.bg = self.bg.or(other.bg);
        self.ul = self.ul.or(other.ul);
    }

    #[inline]
    fn bitxor(self, other: Style) -> Self {
        let mut style = self;

        style.bitxor_assign(other);

        style
    }

    fn bitxor_assign(&mut self, other: Style) {
        self.attributes.bitxor_assign(other.attributes);

        self.fg = self.fg.xor(other.fg);
        self.bg = self.bg.xor(other.bg);
        self.ul = self.ul.xor(other.ul);
    }

    pub fn diff(self, other: Style) -> Self {
        let mut style = self;

        if other.is_empty() {
            style.clear();
            style.set(Attribute::Reset);
            return style;
        }

        style.bg = if other.bg == style.bg {
            Color::None
        } else {
            other.bg
        };
        style.fg = if other.fg == style.fg {
            Color::None
        } else {
            other.fg
        };
        style.ul = if other.ul == style.ul {
            Color::None
        } else {
            other.ul
        };

        style.set(other.attributes - style.attributes);
        style
    }

    /// Returns `true` if the style is empty.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.fg.is_none() && self.bg.is_none() && self.ul.is_none()
    }

    /// Clears the style.
    #[inline]
    pub fn clear(&mut self) {
        self.attributes.clear();
        self.bg = Color::None;
        self.fg = Color::None;
        self.ul = Color::None;
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::EMPTY
    }
}

impl Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("Style::EMPTY");
        }
        
        let mut debug = f.debug_struct("Style");


        if !self.fg.is_none() {
            debug.field("foreground", &self.fg);
        }

        if !self.bg.is_none() {
            debug.field("background", &self.bg);
        }

        if !self.ul.is_none() {
            debug.field("underline", &self.ul);
        }

        if !self.attributes.is_empty() {
            debug.field("attributes", &self.attributes);
        }

        debug.finish()
    }
}

impl Sub for Style {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.diff(other)
    }
}
impl SubAssign for Style {
    fn sub_assign(&mut self, other: Self) {
        *self = self.diff(other);
    }
}
impl BitAnd for Style {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        self.bitand(other)
    }
}

impl BitAndAssign for Style {
    fn bitand_assign(&mut self, other: Self) {
        self.bitand_assign(other)
    }
}

impl BitOr for Style {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        self.bitor(other)
    }
}

impl BitOrAssign for Style {
    fn bitor_assign(&mut self, other: Self) {
        self.bitor_assign(other)
    }
}

impl Escape for Style {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use crate::io::Write;
        use std::io::Write as _;

        if self.is_empty() {
            return Ok(());
        }

        if self.contains(Attribute::Reset) {
            return w.write_all(b"\x1B[0m");
        }

        w.write_all(b"\x1B[")?;

        separator!({ w.write_all(b";") });

        if self.bg.is_some() {
            separate!(w.escape(&self.bg.as_background())?);
        }

        if self.fg.is_some() {
            separate!(w.escape(&self.fg.as_foreground())?);
        }

        if self.ul.is_some() {
            separate!(w.escape(&self.ul.as_underline())?);
        }

        // Attributes (bold, underline, etc.)
        for attr in self.attributes.sgr() {
            separate!(w.write(attr.as_bytes())?);
        }

        w.write_all(b"m")
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum UnderlineStyle {
    None,
    #[default]
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

impl UnderlineStyle {
    pub const MAX: Attribute = Attribute::new(
        Attribute::UnderlineStyleSingle.bits()
            | Attribute::UnderlineStyleDouble.bits()
            | Attribute::UnderlineStyleCurly.bits()
            | Attribute::UnderlineStyleDotted.bits()
            | Attribute::UnderlineStyleDashed.bits(),
    );
}

impl From<UnderlineStyle> for Attribute {
    fn from(value: UnderlineStyle) -> Self {
        match value {
            UnderlineStyle::None => Attribute::UnderlineStyleNone,
            UnderlineStyle::Single => Attribute::UnderlineStyleSingle,
            UnderlineStyle::Double => Attribute::UnderlineStyleDouble,
            UnderlineStyle::Curly => Attribute::UnderlineStyleCurly,
            UnderlineStyle::Dotted => Attribute::UnderlineStyleDotted,
            UnderlineStyle::Dashed => Attribute::UnderlineStyleDashed,
        }
    }
}
