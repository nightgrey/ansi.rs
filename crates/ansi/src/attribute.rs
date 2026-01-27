use derive_more::{Deref, DerefMut, IsVariant};

pub use ::bitflags::{Bits, Flags, bitflags, bitflags_match};
pub use ::std::ops::{BitAnd, BitOr, BitXor, Not};
use std::io::Write;
use crate::Escape;

bitflags! {
    /// Attributes
    ///
    /// Defines a bitset of [`Style`] attributes.
    ///
    /// # Remarks
    ///
    /// - Each bit is `1 << [bit] + 1` to differentiate reset vs. empty.
    #[derive(Copy, Clone, Debug, Deref, DerefMut, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Attribute: u32 {
        /// Resets all the attributes.
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

        /// Sets the underline style to "none".
        const UnderlineStyleNone = 1 << 10;
        /// Sets the underline style to "single".
        const UnderlineStyleSingle = 1 << 11;
        /// Sets the underline style to "double".
        const UnderlineStyleDouble = 1 << 12;
        /// Sets the underline style to "curly".
        const UnderlineStyleCurly = 1 << 13;
        /// Sets the underline style to "dotted".
        const UnderlineStyleDotted = 1 << 14;
        /// Sets the underline style to "dashed".
        const UnderlineStyleDashed = 1 << 15;

        /// Turns off the `Bold` attribute.
        const NoBold = 1 << 16;
        /// Turns off the `Italic` and `Bold` attributes.
        const NormalIntensity = 1 << 17;
        /// Turns off the `Italic` attribute.
        const NoItalic = 1 << 18;
        /// Turns off the `Underline` attribute.
        const NoUnderline = 1 << 19;
        /// Turns off the text blinking.
        const NoBlink = 1 << 20;
        /// Turns off the `Reverse` and `Conceal` attributes.
        const NoReverse = 1 << 21;
        /// Turns off the `Conceal` attribute.
        const NoConceal = 1 << 22;
        /// Turns off the `Strikethrough` attribute.
        const NoStrikethrough = 1 << 23;

        /// Frames the text.
        const Frame = 1 << 24;
        /// Encircles the text.
        const Encircle = 1 << 25;
        /// Draws a line at the top of the text.
        const Overline = 1 << 26;
        /// Turns off the `Frame` and `Encircle` attributes.
        const NoFrameOrEncircle = 1 << 27;
        /// Turns off the `Overline` attribute.
        const NoOverline = 1 << 28;
    }
}

impl Attribute {
    pub const COUNT: usize = 29;
    pub const EMPTY: Attribute = Attribute::new(0);

    /// [`Attribute`] with *all* bits set, no exclusions.
    pub const ALL: Attribute = Attribute::new(
        Attribute::Reset.bits()
            | Attribute::Bold.bits()
            | Attribute::Faint.bits()
            | Attribute::Italic.bits()
            | Attribute::Underline.bits()
            | Attribute::UnderlineStyleNone.bits()
            | Attribute::UnderlineStyleSingle.bits()
            | Attribute::UnderlineStyleDouble.bits()
            | Attribute::UnderlineStyleCurly.bits()
            | Attribute::UnderlineStyleDotted.bits()
            | Attribute::UnderlineStyleDashed.bits()
            | Attribute::Blink.bits()
            | Attribute::RapidBlink.bits()
            | Attribute::Reverse.bits()
            | Attribute::Conceal.bits()
            | Attribute::Strikethrough.bits()
            | Attribute::NoBold.bits()
            | Attribute::NormalIntensity.bits()
            | Attribute::NoItalic.bits()
            | Attribute::NoUnderline.bits()
            | Attribute::NoBlink.bits()
            | Attribute::NoReverse.bits()
            | Attribute::NoConceal.bits()
            | Attribute::NoStrikethrough.bits()
            | Attribute::Frame.bits()
            | Attribute::Encircle.bits()
            | Attribute::Overline.bits()
            | Attribute::NoFrameOrEncircle.bits()
            | Attribute::NoOverline.bits(),
    );

    pub const fn new(bits: u32) -> Self {
        Self::from_bits_truncate(bits)
    }

    pub fn insert_only(&mut self, other: Self) {
        self.0.insert(other.0);
    }
    /// Inserts the given attributes, and removes the inverse attributes.
    #[inline]
    pub fn insert_inclusive(&mut self, other: Self) {
        self.insert_only(other);
        self.remove(other.inverse());
    }
    /// Returns the inverse of the given attributes.
    ///
    /// See [`Attribute::invert`].
    pub fn inverse(self) -> Self {
        const INVERSE: &'static [(Attribute, Attribute)] = &[
            (
                Attribute::Reset,
                Attribute::ALL.difference(Attribute::Reset),
            ),
            (
                Attribute::Bold,
                Attribute::new(
                    Attribute::NoBold.bits() | Attribute::NormalIntensity.bits(),
                ),
            ),
            (Attribute::Faint, Attribute::NormalIntensity),
            (
                Attribute::Italic,
                Attribute::new(
                    Attribute::NoItalic.bits() | Attribute::NormalIntensity.bits(),
                ),
            ),
            (
                Attribute::Underline,
                Attribute::new(
                    Attribute::NoUnderline.bits() | Attribute::UnderlineStyleNone.bits(),
                ),
            ),
            (
                Attribute::Blink,
                Attribute::new(
                    Attribute::NoBlink.bits() | Attribute::RapidBlink.bits(),
                ),
            ),
            (
                Attribute::RapidBlink,
                Attribute::new(Attribute::NoBlink.bits() | Attribute::Blink.bits()),
            ),
            (Attribute::Reverse, Attribute::NoReverse),
            (Attribute::Conceal, Attribute::NoConceal),
            (Attribute::Strikethrough, Attribute::NoStrikethrough),
            // Underline styles: each inverts to all others + NoUnderline
            (
                Attribute::UnderlineStyleNone,
                Attribute::new(
                    Attribute::UnderlineStyleSingle.bits()
                        | Attribute::UnderlineStyleDouble.bits()
                        | Attribute::UnderlineStyleCurly.bits()
                        | Attribute::UnderlineStyleDotted.bits()
                        | Attribute::UnderlineStyleDashed.bits(),
                ),
            ),
            (
                Attribute::UnderlineStyleSingle,
                Attribute::new(
                    Attribute::NoUnderline.bits()
                        | Attribute::UnderlineStyleNone.bits()
                        | Attribute::UnderlineStyleDouble.bits()
                        | Attribute::UnderlineStyleCurly.bits()
                        | Attribute::UnderlineStyleDotted.bits()
                        | Attribute::UnderlineStyleDashed.bits(),
                ),
            ),
            (
                Attribute::UnderlineStyleDouble,
                Attribute::new(
                    Attribute::NoUnderline.bits()
                        | Attribute::UnderlineStyleNone.bits()
                        | Attribute::UnderlineStyleSingle.bits()
                        | Attribute::UnderlineStyleCurly.bits()
                        | Attribute::UnderlineStyleDotted.bits()
                        | Attribute::UnderlineStyleDashed.bits(),
                ),
            ),
            (
                Attribute::UnderlineStyleCurly,
                Attribute::new(
                    Attribute::NoUnderline.bits()
                        | Attribute::UnderlineStyleNone.bits()
                        | Attribute::UnderlineStyleSingle.bits()
                        | Attribute::UnderlineStyleDouble.bits()
                        | Attribute::UnderlineStyleDotted.bits()
                        | Attribute::UnderlineStyleDashed.bits(),
                ),
            ),
            (
                Attribute::UnderlineStyleDotted,
                Attribute::new(
                    Attribute::NoUnderline.bits()
                        | Attribute::UnderlineStyleNone.bits()
                        | Attribute::UnderlineStyleSingle.bits()
                        | Attribute::UnderlineStyleDouble.bits()
                        | Attribute::UnderlineStyleCurly.bits()
                        | Attribute::UnderlineStyleDashed.bits(),
                ),
            ),
            (
                Attribute::UnderlineStyleDashed,
                Attribute::new(
                    Attribute::NoUnderline.bits()
                        | Attribute::UnderlineStyleNone.bits()
                        | Attribute::UnderlineStyleSingle.bits()
                        | Attribute::UnderlineStyleDouble.bits()
                        | Attribute::UnderlineStyleCurly.bits()
                        | Attribute::UnderlineStyleDotted.bits(),
                ),
            ),
            // Negatives invert to positives
            (Attribute::NoBold, Attribute::Bold),
            (
                Attribute::NormalIntensity,
                Attribute::new(
                    Attribute::Faint.bits() | Attribute::Italic.bits() | Attribute::Bold.bits(),
                ),
            ),
            (
                Attribute::NoItalic,
                Attribute::new(Attribute::Italic.bits() | Attribute::Bold.bits()),
            ),
            (
                Attribute::NoUnderline,
                Attribute::new(
                    Attribute::Underline.bits()
                        | Attribute::UnderlineStyleNone.bits()
                        | Attribute::UnderlineStyleSingle.bits()
                        | Attribute::UnderlineStyleDouble.bits()
                        | Attribute::UnderlineStyleCurly.bits()
                        | Attribute::UnderlineStyleDotted.bits()
                        | Attribute::UnderlineStyleDashed.bits(),
                ),
            ),
            (
                Attribute::NoBlink,
                Attribute::new(Attribute::Blink.bits() | Attribute::RapidBlink.bits()),
            ),
            (Attribute::NoReverse, Attribute::Reverse),
            (Attribute::NoConceal, Attribute::Conceal),
            (Attribute::NoStrikethrough, Attribute::Strikethrough),
            (
                Attribute::Frame,
                Attribute::new(
                    Attribute::NoFrameOrEncircle.bits() | Attribute::Encircle.bits(),
                ),
            ),
            (
                Attribute::Encircle,
                Attribute::new(
                    Attribute::NoFrameOrEncircle.bits() | Attribute::Frame.bits(),
                ),
            ),
            (Attribute::Overline, Attribute::NoOverline),
            (
                Attribute::NoFrameOrEncircle,
                Attribute::new(Attribute::Frame.bits() | Attribute::Encircle.bits()),
            ),
            (Attribute::NoOverline, Attribute::Overline),
        ];

        let mut out = Self::empty();
        for (attr, inv) in INVERSE {
            if self.contains(*attr) {
                out.insert_only(*inv);
            }
        }
        out

    }

    pub fn diff(self, other: Self) -> Self {
        let mut result = Self::empty();
        result.insert_inclusive(other.symmetric_difference(self));
        result
    }

    /// Returns an iterator of the SGR parameter strings for each attribute.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn sgr(&self) -> impl Iterator<Item = &'static str> {
        const SGR_STR: &'static [(Attribute, &'static str)] = &[
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
            (Attribute::UnderlineStyleNone, "4:0"),
            (Attribute::UnderlineStyleSingle, "4:1"),
            (Attribute::UnderlineStyleDouble, "4:2"),
            (Attribute::UnderlineStyleCurly, "4:3"),
            (Attribute::UnderlineStyleDotted, "4:4"),
            (Attribute::UnderlineStyleDashed, "4:5"),
            (Attribute::NoBold, "21"),
            (Attribute::NormalIntensity, "22"),
            (Attribute::NoItalic, "23"),
            (Attribute::NoUnderline, "24"),
            (Attribute::NoBlink, "25"),
            (Attribute::NoReverse, "27"),
            (Attribute::NoConceal, "28"),
            (Attribute::NoStrikethrough, "29"),
            (Attribute::Frame, "51"),
            (Attribute::Encircle, "52"),
            (Attribute::Overline, "53"),
            (Attribute::NoFrameOrEncircle, "54"),
            (Attribute::NoOverline, "55"),
        ];

        SGR_STR
            .iter()
            .filter(|(attr, _)| self.contains(*attr))
            .map(|(_, sgr)| *sgr)
    }

}

pub type AttributesIter = bitflags::iter::Iter<Attribute>;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_attributes() {
        let attrs = Attribute::empty();
        assert_eq!(attrs.bits(), 0);
        assert_eq!(attrs.sgr().count(), 0);
    }

    #[test]
    fn single_attribute() {
        let bold = Attribute::Bold;
        assert!(bold.contains(Attribute::Bold));
        assert!(!bold.contains(Attribute::Italic));
    }

    #[test]
    fn combine_attributes() {
        let styled = Attribute::Bold | Attribute::Italic | Attribute::Underline;
        assert!(styled.contains(Attribute::Bold));
        assert!(styled.contains(Attribute::Italic));
        assert!(styled.contains(Attribute::Underline));
        assert!(!styled.contains(Attribute::Strikethrough));
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
        fn sgr_underline_styles() {
            let single = Attribute::UnderlineStyleSingle;
            assert_eq!(single.sgr().collect::<Vec<_>>(), vec!["4:1"]);

            let double = Attribute::UnderlineStyleDouble;
            assert_eq!(double.sgr().collect::<Vec<_>>(), vec!["4:2"]);

            let curly = Attribute::UnderlineStyleCurly;
            assert_eq!(curly.sgr().collect::<Vec<_>>(), vec!["4:3"]);
        }

        #[test]
        fn sgr_reset() {
            let reset = Attribute::Reset;
            assert_eq!(reset.sgr().collect::<Vec<_>>(), vec!["0"]);
        }
    }

    mod inverse {
        use std::time::Instant;
        use super::*;

        #[test]
        fn inverse_bold() {
            let inverse = Attribute::Bold.inverse();
            assert!(inverse.contains(Attribute::NoBold));
            assert!(inverse.contains(Attribute::NormalIntensity));
        }

        #[test]
        fn inverse_italic() {
            let inverse = Attribute::Italic.inverse();
            assert!(inverse.contains(Attribute::NoItalic));
            assert!(inverse.contains(Attribute::NormalIntensity));
        }

        #[test]
        fn inverse_underline() {
            let inverse = Attribute::Underline.inverse();
            assert!(inverse.contains(Attribute::NoUnderline));
            assert!(inverse.contains(Attribute::UnderlineStyleNone));
        }

        #[test]
        fn inverse_no_bold() {
            let inverse = Attribute::NoBold.inverse();
            assert_eq!(inverse, Attribute::Bold);
        }

        #[test]
        fn inverse_blink() {
            let inverse = Attribute::Blink.inverse();
            assert!(inverse.contains(Attribute::NoBlink));
            assert!(inverse.contains(Attribute::RapidBlink));
        }

        #[test]
        fn inverse_frame() {
            let inverse = Attribute::Frame.inverse();
            assert!(inverse.contains(Attribute::NoFrameOrEncircle));
            assert!(inverse.contains(Attribute::Encircle));
        }

        #[test]
        fn inverse_encircle() {
            let inverse = Attribute::Encircle.inverse();
            assert!(inverse.contains(Attribute::NoFrameOrEncircle));
            assert!(inverse.contains(Attribute::Frame));
        }

        #[test]
        fn inverse_normal_intensity() {
            let inverse = Attribute::NormalIntensity.inverse();
            assert!(inverse.contains(Attribute::Faint));
            assert!(inverse.contains(Attribute::Italic));
            assert!(inverse.contains(Attribute::Bold));
        }

        #[test]
        fn inverse_empty() {
            let inverse = Attribute::empty().inverse();
            assert_eq!(inverse, Attribute::empty());
        }
    }

    #[test]
    fn constants_empty() {
        assert_eq!(Attribute::EMPTY, Attribute::empty());
        assert_eq!(Attribute::EMPTY.bits(), 0);
    }

    #[test]
    fn constants_all() {
        let all = Attribute::ALL;
        assert!(all.contains(Attribute::Reset));
        assert!(all.contains(Attribute::Bold));
        assert!(all.contains(Attribute::NoBold));
        assert!(all.contains(Attribute::Italic));
        assert!(all.contains(Attribute::NoItalic));
    }

    #[test]
    fn remove_using_inverse() {
        let mut attrs = Attribute::Bold | Attribute::NoBold;
        attrs.remove(Attribute::Bold.inverse());
        assert_eq!(attrs, Attribute::Bold);
    }

    #[test]
    fn double_inverse_identity() {
        // Note: This won't be perfect due to complex mappings like NormalIntensity
        let original = Attribute::Reverse;
        let double_inverted = original.inverse().inverse();
        assert_eq!(original, double_inverted);
    }

    #[test]
    fn underline_style_variants() {
        let styles = vec![
            (Attribute::UnderlineStyleNone, "4:0"),
            (Attribute::UnderlineStyleSingle, "4:1"),
            (Attribute::UnderlineStyleDouble, "4:2"),
            (Attribute::UnderlineStyleCurly, "4:3"),
            (Attribute::UnderlineStyleDotted, "4:4"),
            (Attribute::UnderlineStyleDashed, "4:5"),
        ];

        for (attr, expected_sgr) in styles {
            let sgr: Vec<&str> = attr.sgr().collect();
            assert_eq!(sgr, vec![expected_sgr]);
        }
    }
}
