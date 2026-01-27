use compact_str::CompactString;
use std::convert::AsRef;
use unicode_width::UnicodeWidthStr;
use ansi::{Attribute, Color, Escape, Flags, Style};
use utils::separator;

/// Represents a single terminal grid cell with display attributes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    /// Empty string represents an empty cell.
    grapheme: CompactString,

    /// Text styling attributes (bold, italic, etc.)
    pub attributes: Attribute,
    pub fg: Color,
    pub bg: Color,
    pub ul: Color,

    skipped: bool,
}

impl Cell {
    #[cfg(not(test))]
    pub const SPACE: &'static str = " ";

    #[cfg(test)]
    pub const SPACE: &'static str = "▒";

    pub const EMPTY: Cell = Cell {
        grapheme: CompactString::const_new(Self::SPACE),
        attributes: Attribute::EMPTY,
        fg: Color::None,
        bg: Color::None,
        ul: Color::None,
        skipped: false,
    };

    pub fn new<S: AsRef<str>>(grapheme: S) -> Self {
        Self {
            grapheme: CompactString::new(grapheme),
            ..Self::EMPTY
        }
    }

    pub fn new_const(grapheme: &'static str) -> Self {
        Self {
            grapheme: CompactString::const_new(grapheme),
            ..Self::EMPTY
        }
    }

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub fn grapheme(&self) -> &CompactString {
        &self.grapheme
    }

    pub fn set_grapheme(&mut self, grapheme: impl AsRef<str>) {
        self.grapheme.clear();
        self.grapheme.push_str(grapheme.as_ref());
    }

    pub fn set_char(&mut self, char: char) {
        self.grapheme.clear();
        self.grapheme.push(char);
    }

    pub fn skip(&mut self, value: bool) -> &mut Self {
        self.skipped = value;
        self
    }

    pub fn set_style(&mut self, style: &Style) {
        self.attributes = style.attributes;
        self.fg = style.fg;
        self.bg = style.bg;
        self.ul = style.ul;
    }

    pub fn is_empty(&self) -> bool {
        self.grapheme == Self::SPACE
    }

    pub fn is_unstyled(&self) -> bool {
        self.attributes.is_empty()
            && self.fg.is_none()
            && self.bg.is_none()
            && self.ul.is_none()
    }
    
    pub fn style(&self) -> Style {
        Style {
            attributes: self.attributes,
            fg: self.fg,
            bg: self.bg,
            ul: self.ul,
        }
    }

    pub fn as_str(&self) -> &str {
        self.grapheme.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.grapheme.as_bytes()
    }

    pub fn clear(&mut self) -> &mut Self {
        self.set_grapheme(Self::SPACE);
        self.attributes.clear();
        self.bg = Color::None;
        self.fg = Color::None;
        self.ul = Color::None;
        self
    }
}

impl PartialEq<Style> for Cell {
    fn eq(&self, other: &Style) -> bool {
        self.attributes == other.attributes && self.fg == other.fg && self.bg == other.bg && self.ul == other.ul
    }
}

impl Into<Style> for Cell {
    fn into(self) -> Style {
        Style {
            attributes: self.attributes,
            fg: self.fg,
            bg: self.bg,
            ul: self.ul,
        }
    }
}
impl Default for Cell {
    fn default() -> Self {
        Cell::empty()
    }
}

impl AsRef<str> for Cell {
    fn as_ref(&self) -> &str {
        self.grapheme.as_ref()
    }
}

impl UnicodeWidthStr for Cell {
    fn width(&self) -> usize {
        self.grapheme.width()
    }

    fn width_cjk(&self) -> usize {
        self.grapheme.width_cjk()
    }
}

impl FromIterator<Cell> for String {
    fn from_iter<I: IntoIterator<Item = Cell>>(iter: I) -> Self {
        iter.into_iter()
            .map(|cell| cell.as_str().to_string())
            .collect::<String>()
    }
}

impl<'a> FromIterator<&'a Cell> for String {
    fn from_iter<I: IntoIterator<Item = &'a Cell>>(iter: I) -> Self {
        iter.into_iter()
            .map(|cell| cell.as_str().to_string())
            .collect::<String>()
    }
}


impl Escape for Cell {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use ansi::io::Write;
        
        if self.is_unstyled() {
            w.write(self.as_bytes())?;
            return Ok(());
        }

        w.write(b"\x1b[")?;

        separator!(w.write(b";"));

        if self.bg.is_some() {
            separate!(w.write_escape(&self.bg.as_background())?);
        }

        if self.fg.is_some() {
            separate!(w.write_escape(&self.fg.as_foreground())?);
        }

        if self.ul.is_some() {
            separate!(w.write_escape(&self.ul.as_underline())?);
        }

        separate!(w.write_escape(&self.attributes)?);

        w.write(b"m")?;

        w.write(self.as_bytes())?;

        Ok(())
    }
}
