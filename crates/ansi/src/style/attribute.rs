use std::borrow::Cow;
use std::fmt::from_fn;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use bitflags::{bitflags, bitflags_match, iter::{Iter, IterNames, IterDefinedNames}, Bits, Flag, Flags};
use crate::Escape;

bitflags! {
    /// Attribute
    ///
    /// Defines a compact representation of ANSI SGR attributes.

    #[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Attribute: u16 {
        /// No attributes.
        const None = 0;
        /// Resets all attributes.
        const Reset = 1 << 0;
        /// Increases the text intensity.
        const Bold = 1 << 1;
        /// Decreases the text intensity.
        const Faint = 1 << 2;
        /// Emphasises the text.
        const Italic = 1 << 3;
        /// Underlines the text.
        const Underline = 1 << 4;
        /// Makes the text blink.
        const Blink = 1 << 5;
        /// Makes the text blink rapidly.
        const RapidBlink = 1 << 6;
        /// Swaps the foreground and background colors.
        const Reverse = 1 << 7;
        /// Hides the text.
        const Conceal = 1 << 8;
        /// Crosses the text out.
        const Strikethrough = 1 << 9;

        /// Frames the text.
        const Frame = 1 << 10;
        /// Encircles the text.
        const Encircle = 1 << 11;
        /// Draws a line at the top of the text.
        const Overline = 1 << 12;
    }
}

impl Attribute {
    pub const COUNT: usize = <Self as Flags>::FLAGS.len();

    /// All defined attributes combined.
    pub const MAX: Self = Self::new(<Self as Flags>::Bits::ALL);

    pub const fn new(bits: u16) -> Self {
        Self::from_bits_truncate(bits)
    }

    pub  fn is_none(&self) -> bool {
        self == &Attribute::None
    }

    pub  fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub  fn is_reset(&self) -> bool {
        self == &Attribute::Reset
    }

    /// Returns an iterator over the SGR parameter strings for each attribute.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn sgr(self) -> impl Iterator<Item = &'static str> {
        static SGR: &[(Attribute, &str)] = &[
            (Attribute::Reset, "0"),
            (Attribute::Bold, "1"),
            (Attribute::Faint, "2"),
            (Attribute::Italic, "3"),
            (Attribute::Underline, "4"),
            (Attribute::Blink, "5"),
            (Attribute::RapidBlink, "6"),
            (Attribute::Reverse, "7"),
            (Attribute::Conceal, "8"),
            (Attribute::Strikethrough, "9"),
            (Attribute::Frame, "51"),
            (Attribute::Encircle, "52"),
            (Attribute::Overline, "53"),
        ];

        SGR.iter().filter_map(move |&(attr, sgr)| {
            self.contains(attr).then_some(sgr)
        })
    }

    /// Returns an iterator over the names of the attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    ///
    /// assert_eq!(attrs.names().map(|(name, _)| name).collect(), vec!["Bold", "Italic"]);
    /// ```
    #[inline]
    pub fn names(self) -> AttributeNames {
        self.iter_names()
    }

    /// Returns an iterator over all defined attribute variants.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// assert!(Attribute::variants().any(|(name, _)| name == "Bold"));
    /// assert!(Attribute::variants().any(|(name, _)| name == "Italic"));
    /// assert_eq!(Attribute::variants().count(), Attribute::COUNT);
    /// ```
    #[inline]
    pub fn variants() -> AttributeVariants {
        Attribute::iter_defined_names()
    }

    /// Returns a string representation of the attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    ///
    /// assert_eq!(attrs.as_string(), "Bold | Italic");
    /// ```
    pub fn as_string(&self) -> Cow<str> {
        self.names().map(|(str, attr)| str).intersperse(" | ").collect()
    }
}


impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("Attribute::None");
        }

        if self.is_reset() {
            return f.write_str("Attribute::Reset");
        }

        f.debug_tuple("Attribute").field(&from_fn(|f| {
            for (i, (name, _)) in self.names().enumerate() {
                if i > 0 {
                    f.write_str(" | ")?;
                }
                f.write_str(name)?;
            }

            Ok(())
        })).finish()
    }
}

impl Escape for Attribute {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        for (i, attr) in self.sgr().enumerate() {
            if i > 0 {
                w.write(b";")?;
            }
            w.write(attr.as_bytes())?;
        }
        Ok(())
    }
}


pub type AttributeIter = Iter<Attribute>;
pub type AttributeNames = IterNames<Attribute>;
pub type AttributeVariants = IterDefinedNames<Attribute>;

#[cfg(test)]
mod tests {
    use bitflags::Flags;
    use super::*;

    #[test]
    fn empty_attributes() {
        let attrs = Attribute::None;
        assert_eq!(attrs.bits(), 0);
        assert!(attrs.is_empty());
        assert!(!attrs.is_some());
        assert_eq!(attrs.sgr().count(), 0);
    }

    #[test]
    fn single_attribute() {
        let bold = Attribute::Bold;
        assert!(bold.contains(Attribute::Bold));
        assert!(!bold.contains(Attribute::Italic));
        assert!(bold.is_some());
    }

    #[test]
    fn combine_attributes() {
        let styled = Attribute::Bold | Attribute::Italic | Attribute::Underline;
        assert!(styled.contains(Attribute::Bold));
        assert!(styled.contains(Attribute::Italic));
        assert!(styled.contains(Attribute::Underline));
        assert!(!styled.contains(Attribute::Strikethrough));
    }

    #[test]
    fn union() {
        let attrs = Attribute::Bold.union(Attribute::Italic);
        assert!(attrs.contains(Attribute::Bold));
        assert!(attrs.contains(Attribute::Italic));
    }

    #[test]
    fn intersection() {
        let a = Attribute::Bold | Attribute::Italic;
        let b = Attribute::Italic | Attribute::Underline;
        let result = a.intersection(b);
        assert!(!result.contains(Attribute::Bold));
        assert!(result.contains(Attribute::Italic));
        assert!(!result.contains(Attribute::Underline));
    }

    #[test]
    fn difference() {
        let a = Attribute::Bold | Attribute::Italic;
        let result = a.difference(Attribute::Italic);
        assert!(result.contains(Attribute::Bold));
        assert!(!result.contains(Attribute::Italic));
    }

    #[test]
    fn symmetric_difference() {
        let a = Attribute::Bold | Attribute::Italic;
        let b = Attribute::Italic | Attribute::Underline;
        let result = a.symmetric_difference(b);
        assert!(result.contains(Attribute::Bold));
        assert!(!result.contains(Attribute::Italic));
        assert!(result.contains(Attribute::Underline));
    }

    #[test]
    fn insert_remove() {
        let mut attrs = Attribute::None;
        attrs.insert(Attribute::Bold);
        assert!(attrs.contains(Attribute::Bold));
        attrs.remove(Attribute::Bold);
        assert!(!attrs.contains(Attribute::Bold));
    }

    #[test]
    fn toggle() {
        let mut attrs = Attribute::Bold;
        attrs.toggle(Attribute::Bold);
        assert!(!attrs.contains(Attribute::Bold));
        attrs.toggle(Attribute::Bold);
        assert!(attrs.contains(Attribute::Bold));
    }

    #[test]
    fn set() {
        let mut attrs = Attribute::None;
        attrs.set(Attribute::Bold, true);
        assert!(attrs.contains(Attribute::Bold));
        attrs.set(Attribute::Bold, false);
        assert!(!attrs.contains(Attribute::Bold));
    }

    #[test]
    fn clear() {
        let mut attrs = Attribute::Bold | Attribute::Italic;
        attrs.clear();
        assert!(attrs.is_empty());
    }

    mod sgr {
        use super::*;

        #[test]
        fn sgr_single() {
            let bold = Attribute::Bold;
            let sgr: Vec<&str> = bold.sgr().collect();
            assert_eq!(sgr, vec!["1"]);
        }

        #[test]
        fn sgr_multiple() {
            let styled = Attribute::Bold | Attribute::Italic | Attribute::Underline;
            let sgr: Vec<&str> = styled.sgr().collect();
            assert!(sgr.contains(&"1")); // Bold
            assert!(sgr.contains(&"3")); // Italic
            assert!(sgr.contains(&"4")); // Underline
            assert_eq!(sgr.len(), 3);
        }

        #[test]
        fn sgr_reset() {
            let reset = Attribute::Reset;
            let sgr: Vec<&str> = reset.sgr().collect();
            assert_eq!(sgr, vec!["0"]);
        }

        #[test]
        fn sgr_frame_encircle_overline() {
            let attrs = Attribute::Frame | Attribute::Encircle | Attribute::Overline;
            let sgr: Vec<&str> = attrs.sgr().collect();
            assert!(sgr.contains(&"51"));
            assert!(sgr.contains(&"52"));
            assert!(sgr.contains(&"53"));
        }
    }

    #[test]
    fn debug_empty() {
        let attrs = Attribute::None;
        assert_eq!(format!("{:?}", attrs), "Attribute::None");
    }

    #[test]
    fn debug_single() {
        let attrs = Attribute::Bold;
        assert_eq!(format!("{:?}", attrs), "Attribute(Bold)");
    }

    #[test]
    fn debug_multiple() {
        let attrs = Attribute::Bold | Attribute::Italic;
        let debug = format!("{:?}", attrs);
        assert!(debug.contains("Bold"));
        assert!(debug.contains("Italic"));
        assert!(debug.contains(" | "));
    }

    #[test]
    fn constants_max() {
        let all = Attribute::MAX;
        assert!(all.contains(Attribute::Reset));
        assert!(all.contains(Attribute::Bold));
        assert!(all.contains(Attribute::Italic));
        assert!(all.contains(Attribute::Overline));
    }
}
