use crate::Escape;
use maybe::Maybe;
use std::borrow::Cow;
use utils::{const_bitflags, ParseFlagsError};

const_bitflags! {
    pub struct Attribute(u16);
    pub struct AttributeIter;

    Bold = 0,
    Faint = 1,
    Italic = 2,
    Underline = 3,
    Blink = 4,
    RapidBlink = 5,
    Inverse = 6,
    Invisible = 7,
    Strikethrough = 8,
    UnderlineDouble = 9,
    UnderlineCurly = 10,
    Frame = 11,
    Encircle = 12,
    Overline = 13,
}

macro_rules! sgr {
    (
        $(
            $variant:ident => ($set:expr, $reset:expr),
        )+
        $(,)?
    ) => {
        pub const SGR: &'static [&'static str] = &[
            $(
                $set,
            )+
        ];
        pub const RESET: &'static [&'static str] = &[
            $(
                $reset,
            )+
        ];
    };
}

impl Attribute {
    sgr! {
        Bold => ("1", "22"),
        Faint => ("2", "22"),
        Italic => ("3", "23"),
        Underline => ("4", "24"),
        Blink => ("5", "25"),
        RapidBlink => ("6", "25"),
        Inverse => ("7", "27"),
        Invisible => ("8", "28"),
        Strikethrough => ("9", "29"),
        UnderlineDouble => ("21", "24"),
        UnderlineCurly => ("23", "24"),
        Frame => ("51", "54"),
        Encircle => ("52", "54"),
        Overline => ("53", "55"),
    }

    /// Returns an iterator over the SGR parameters for each attribute.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub const fn iter_sgr(self) -> impl Iterator<Item = &'static str> {
        self.iter().map(|i| Attribute::SGR[i])
    }

    /// Returns an iterator over the SGR reset parameters for each attribute.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub const fn iter_reset(self) -> impl Iterator<Item = &'static str> {
        self.iter().map(|i| Attribute::RESET[i])
    }

    /// Returns the semicolon-separated SGR parameters to set attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    /// assert_eq!(attrs.to_sgr(), "1;3");
    /// ```
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn to_sgr(self) -> Cow<'static, str> {
        self.iter_sgr().intersperse(";").collect()
    }

    /// Returns the semicolon-separated SGR parameters to reset attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    ///
    /// assert_eq!(attrs.to_reset(), "22;23");
    /// ```
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn to_reset(self) -> Cow<'static, str> {
        self.iter_reset().intersperse(";").collect()
    }
}

impl Escape for Attribute {
    fn escape(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        w.write_all(self.to_sgr().as_bytes())
    }
}

const impl Maybe for Attribute {
    #[allow(non_upper_case_globals)]
    const None: Self = Attribute::from_bits_retained(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_attributes() {
        let attrs = Attribute::None;
        assert_eq!(attrs.into_inner(), 0);
        assert!(attrs.is_empty());
        assert!(!attrs.is_some());
        assert_eq!(attrs.iter_sgr().count(), 0);
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
    fn clear() {
        let mut attrs = Attribute::Bold | Attribute::Italic;
        attrs.clear();
        assert!(attrs.is_none());
    }

    mod sgr {
        use super::*;

        #[test]
        fn sgr_single() {
            let bold = Attribute::Bold;
            let sgr: Vec<&'static str> = bold.iter_sgr().collect();
            assert_eq!(sgr, vec!["1"]);
        }

        #[test]
        fn sgr_multiple() {
            let styled = Attribute::Bold | Attribute::Italic | Attribute::Underline;
            let sgr: Vec<&'static str> = styled.iter_sgr().collect();
            assert!(sgr.contains(&"1")); // Bold
            assert!(sgr.contains(&"3")); // Italic
            assert!(sgr.contains(&"4")); // Underline
            assert_eq!(sgr.len(), 3);
        }

        #[test]
        fn sgr_frame_encircle_overline() {
            let attrs = Attribute::Frame | Attribute::Encircle | Attribute::Overline;
            let sgr: Vec<&'static str> = attrs.iter_sgr().collect();
            dbg!(&sgr);
            assert!(sgr.contains(&"51")); // Frame
            assert!(sgr.contains(&"52")); // Encircle
            assert!(sgr.contains(&"53")); // Overline
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
        dbg!(&attrs.is_empty());
        let debug = format!("{:?}", attrs);
        dbg!(&debug);
        assert!(debug.contains("Bold"));
        assert!(debug.contains("Italic"));
        assert!(debug.contains(" | "));
    }

    #[test]
    fn constants_all() {
        let all = Attribute::All;
        assert!(all.contains(Attribute::Bold));
        assert!(all.contains(Attribute::Italic));
        assert!(all.contains(Attribute::Overline));

        assert!(all.is_all());
    }
}
