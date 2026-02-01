use crate::text::DisplayWidth;
use ansi::io::Write;
use ansi::{Escape, Flags, Style};
use compact_str::CompactString;
use std::convert::AsRef;
use unicode_width::UnicodeWidthStr;

/// Represents a single terminal grid cell with display attributes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    /// Empty string represents an empty cell.
    content: CompactString,
    width: usize,
    style: Style,
}

impl Cell {
    pub const SPACE: &'static str = &" ";

    pub const EMPTY: Cell = Cell {
        content: CompactString::const_new(Self::SPACE),
        width: 1,
        style: Style::EMPTY,
    };

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub fn new<S: AsRef<str>>(grapheme: S) -> Self {
        let grapheme = CompactString::new(grapheme);
        let width = grapheme.cluster_display_width();
        Self {
            content: grapheme,
            width,
            ..Self::EMPTY
        }
    }

    pub fn new_const(grapheme: &'static str) -> Self {
        Self {
            content: CompactString::const_new(grapheme),
            ..Self::EMPTY
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn style(&self) -> &Style {
        &self.style
    }
    
    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    pub fn set_content(&mut self, grapheme: impl AsRef<str>) {
        self.content.clear();
        self.content.insert_str(0, grapheme.as_ref());
        self.width = self.content.cluster_display_width();
    }

    pub fn set_char(&mut self, char: char) {
        self.content.clear();
        self.content.push(char);
        self.width = self.content.cluster_display_width();
    }

    pub fn set_space(&mut self) {
        self.content.clear();
        self.content.push_str(Self::SPACE);
        self.width = self.content.cluster_display_width();
    }

    pub fn push_str(&mut self, string: impl AsRef<str>) {
        self.content.push_str(string.as_ref());
        self.width = self.content.cluster_display_width();
    }

    pub fn push(&mut self, char: char) {
        self.content.push(char);
        self.width = self.content.cluster_display_width();
    }

    pub fn set_style(&mut self, style: &Style) {
        self.style = style.clone();
    }

    pub fn clear(&mut self) -> &mut Self {
        self.set_space();
        self.style.clear();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.content == Self::SPACE
    }

    pub fn is_unstyled(&self) -> bool {
        self.style.is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.content.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }
}

impl PartialEq<Style> for Cell {
    fn eq(&self, other: &Style) -> bool {
        &self.style == other
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::empty()
    }
}

impl AsRef<str> for Cell {
    fn as_ref(&self) -> &str {
        self.content.as_ref()
    }
}

impl AsRef<Style> for Cell {
    fn as_ref(&self) -> &Style {
        &self.style
    }
}

impl UnicodeWidthStr for Cell {
    fn width(&self) -> usize {
        self.content.width()
    }

    fn width_cjk(&self) -> usize {
        self.content.width_cjk()
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
        w.escape(&self.style)?;
        w.write(self.as_bytes())?;
        Ok(())
    }
}
