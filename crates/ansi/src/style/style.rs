use crate::{Attribute, Color, Escape};
use bitflags::Flags;
use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};
use maybe::Maybe;
use std::cmp::PartialEq;
use std::fmt::{Debug, from_fn};
use std::ops::{BitAnd, BitOr, BitXor, BitXorAssign, Not, Sub, SubAssign};
use utils::separate_by;

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    BitOr,
    BitOrAssign,
    BitAnd,
    BitAndAssign,
    Sub,
    SubAssign,
)]
pub struct Style {
    pub attributes: Attribute,
    pub foreground: Color,
    pub background: Color,
}
#[allow(non_upper_case_globals)]
impl Style {
    pub const None: Style = Self {
        attributes: Attribute::None,
        foreground: Color::None,
        background: Color::None,
    };

    pub const Bold: Self = Style {
        attributes: Attribute::Bold,
        ..Self::None
    };

    pub const Faint: Self = Style {
        attributes: Attribute::Faint,
        ..Self::None
    };

    pub const Italic: Self = Style {
        attributes: Attribute::Italic,
        ..Self::None
    };

    pub const Underline: Self = Style {
        attributes: Attribute::Underline,
        ..Self::None
    };

    pub const Blink: Self = Style {
        attributes: Attribute::Blink,
        ..Self::None
    };

    pub const RapidBlink: Self = Style {
        attributes: Attribute::RapidBlink,
        ..Self::None
    };

    pub const Reverse: Self = Style {
        attributes: Attribute::Reverse,
        ..Self::None
    };

    pub const Conceal: Self = Style {
        attributes: Attribute::Conceal,
        ..Self::None
    };

    pub const Strikethrough: Self = Style {
        attributes: Attribute::Strikethrough,
        ..Self::None
    };

    pub const Frame: Self = Style {
        attributes: Attribute::Frame,
        ..Self::None
    };

    pub const Encircle: Self = Style {
        attributes: Attribute::Encircle,
        ..Self::None
    };

    pub const Overline: Self = Style {
        attributes: Attribute::Overline,
        ..Self::None
    };

    pub const fn from_attribute(attribute: Attribute) -> Self {
        Self {
            attributes: attribute,
            foreground: Color::None,
            background: Color::None,
        }
    }

    /// Returns `true` if the given attribute flag is currently set.
    ///
    /// # Example
    /// ```
    /// use ansi::{Attribute, Style};
    /// let s = Style::default().bold();
    /// assert!(s.contains(Attribute::Bold));
    /// ```
    #[inline]
    pub fn contains(&self, attribute: Attribute) -> bool {
        self.attributes.contains(attribute)
    }

    /// Insert attribute flags.
    pub fn insert(&mut self, attributes: Attribute) {
        self.attributes.insert(attributes);
    }

    /// Set attribute flags.
    pub fn set(&mut self, attributes: Attribute, value: bool) {
        self.attributes.set(attributes, value);
    }

    /// Remove attribute flags.
    #[inline]
    pub fn remove(&mut self, attributes: Attribute) {
        self.attributes.remove(attributes);
    }

    /// Set additional attribute flags.
    pub fn with(mut self, attributes: Attribute) -> Self {
        self.attributes.insert(attributes);
        self
    }

    /// Set the background color.
    #[inline]
    pub const fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub const fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    /// Sets [`Attribute::Bold`].
    #[inline]
    pub fn bold(mut self) -> Self {
        self.insert(Attribute::Bold);
        self
    }

    /// Sets [`Attribute::Faint`].
    #[inline]
    pub fn faint(mut self) -> Self {
        self.insert(Attribute::Faint);
        self
    }

    /// Sets [`Attribute::Italic`].
    #[inline]
    pub fn italic(mut self) -> Self {
        self.insert(Attribute::Italic);
        self
    }

    /// Sets [`Attribute::Underline`].
    #[inline]
    pub fn underline(mut self) -> Self {
        self.insert(Attribute::Underline);
        self
    }

    /// Sets [`Attribute::Blink`].
    #[inline]
    pub fn blink(mut self) -> Self {
        self.insert(Attribute::Blink);
        self
    }

    /// Sets [`Attribute::RapidBlink`].
    #[inline]
    pub fn rapid_blink(mut self) -> Self {
        self.insert(Attribute::RapidBlink);
        self
    }

    /// Sets [`Attribute::Reverse`].
    #[inline]
    pub fn reverse(mut self) -> Self {
        self.insert(Attribute::Reverse);
        self
    }

    /// Sets [`Attribute::Conceal`].
    #[inline]
    pub fn conceal(mut self) -> Self {
        self.insert(Attribute::Conceal);
        self
    }

    /// Sets [`Attribute::Strikethrough`].
    #[inline]
    pub fn strikethrough(mut self) -> Self {
        self.insert(Attribute::Strikethrough);
        self
    }

    /// Sets [`Attribute::Frame`].
    #[inline]
    pub fn frame(mut self) -> Self {
        self.insert(Attribute::Frame);
        self
    }

    /// Sets [`Attribute::Encircle`].
    #[inline]
    pub fn encircle(mut self) -> Self {
        self.insert(Attribute::Encircle);
        self
    }

    /// Sets [`Attribute::Overline`].
    #[inline]
    pub fn overline(mut self) -> Self {
        self.insert(Attribute::Overline);
        self
    }

    /// Unsets: [`Attribute::Bold`]
    #[inline]
    pub fn no_bold(mut self) -> Self {
        self.remove(Attribute::Bold);
        self
    }

    /// Unsets: [`Attribute::Bold`]
    #[inline]
    pub fn normal_intensity(mut self) -> Self {
        self.remove(Attribute::Bold | Attribute::Italic);
        self
    }

    /// Sets [`Attribute::NoItalic`].
    ///
    /// Unsets: [`Attribute::Italic`]
    #[inline]
    pub fn no_italic(mut self) -> Self {
        self.remove(Attribute::Italic);
        self
    }

    /// Unsets: [`Attribute::Underline`]
    #[inline]
    pub fn no_underline(mut self) -> Self {
        self.remove(Attribute::Underline);
        self
    }

    /// Unsets: [`Attribute::Blink`]
    #[inline]
    pub fn no_blink(mut self) -> Self {
        self.remove(Attribute::Blink);
        self
    }

    /// Unsets: [`Attribute::Reverse`]
    #[inline]
    pub fn no_reverse(mut self) -> Self {
        self.remove(Attribute::Reverse);
        self
    }

    /// Unsets: [`Attribute::Conceal`]
    #[inline]
    pub fn no_conceal(mut self) -> Self {
        self.remove(Attribute::Conceal);
        self
    }

    /// Unsets: [`Attribute::Strikethrough`]
    #[inline]
    pub fn no_strikethrough(mut self) -> Self {
        self.remove(Attribute::Strikethrough);
        self
    }

    /// Unsets: [`Attribute::Frame`] [`Attribute::Encircle`]
    #[inline]
    pub fn no_frame_or_encircle(mut self) -> Self {
        self.remove(Attribute::Frame | Attribute::Encircle);
        self
    }

    /// Unsets: [`Attribute::Overline`]
    #[inline]
    pub fn no_overline(mut self) -> Self {
        self.remove(Attribute::Overline);
        self
    }

    #[inline]
    #[must_use]
    pub fn difference(self, other: Self) -> Self {
        Self {
            attributes: self.attributes.difference(other.attributes),
            foreground: self.foreground.difference(other.foreground),
            background: self.background.difference(other.background),
        }
    }

    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        Self {
            attributes: self.attributes.intersection(other.attributes),
            foreground: self.foreground.intersection(other.foreground),
            background: self.background.intersection(other.background),
        }
    }

    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            attributes: self.attributes.union(other.attributes),
            foreground: self.foreground.union(other.foreground),
            background: self.background.union(other.background),
        }
    }

    pub fn is_colored(&self) -> bool {
        self.foreground.is_some() || self.background.is_some()
    }

    /// Returns `true` if the style is none.
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }

    /// Returns `true` if the style is none.
    pub fn is_empty(&self) -> bool {
        self.is_none()
    }

    /// Clears the style.
    #[inline]
    pub fn clear(&mut self) {
        self.attributes.clear();
        self.background = Color::None;
        self.foreground = Color::None;
    }
}

impl BitXor for Style {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl BitXorAssign for Style {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Not for Style {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            attributes: !self.attributes,
            foreground: self.foreground,
            background: self.background,
        }
    }
}



impl Default for Style {
    fn default() -> Self {
        Style::None
    }
}

impl Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none() {
            return f.write_str("Style::None");
        }

        let mut debug = f.debug_tuple("Style");

        if self.attributes.is_some() {
            debug.field(&from_fn(|f| f.write_str(&self.attributes.to_string())));
        }

        if self.is_colored() {
            debug.field(&from_fn(|f| {
                f.debug_set()
                    .entry(&from_fn(|f| {
                        f.write_str("foreground: ")?;
                        write!(f, "{:?}", self.foreground)
                    }))
                    .entry(&from_fn(|f| {
                        f.write_str("background: ")?;
                        write!(f, "{:?}", self.background)
                    }))
                    .finish()
            }));
        }

        debug.finish()
    }
}

impl Escape for Style {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use crate::io::Write as _;
        use std::io::Write as _;

        if self.is_none() {
            return Ok(());
        }

        w.write_all(b"\x1B[")?;

        separate_by!({ w.write_all(b";") });

        if self.background.is_some() {
            separate!(w.escape(self.background.as_background())?);
        }

        if self.foreground.is_some() {
            separate!(w.escape(self.foreground.as_foreground())?);
        }

        // Attributes (bold, underline, etc.)
        separate!(w.write(self.attributes.sgr().as_bytes())?);

        w.write_all(b"m")
    }
}

#[allow(non_upper_case_globals)]
impl Maybe for Style {
    const None: Self = Self {
        attributes: Attribute::None,
        foreground: Color::None,
        background: Color::None,
    };

    fn is_none(&self) -> bool {
        self == &Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_difference() {
        let s1 = Style::Bold.background(Color::White);
        let s2 = Style::None.background(Color::None);
        let diff = s1.difference(s2);
        assert_eq!(diff, Style::None);
    }
}