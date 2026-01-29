use compact_str::CompactString;
use std::convert::AsRef;
use unicode_width::UnicodeWidthStr;
use ansi::{Escape, Flags, Style};
use ansi::io::Write;
use crate::runes::DisplayWidth;

/// Represents a single terminal grid cell with display attributes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    /// Empty string represents an empty cell.
    grapheme: CompactString,
    width: usize,
    pub style: Style,
}

impl Cell {
    #[cfg(not(test))]
    pub const SPACE: &'static str = " ";

    #[cfg(test)]
    pub const SPACE: &'static str = "▒";

    pub const EMPTY: Cell = Cell {
        grapheme: CompactString::const_new(Self::SPACE),
        width: 1,
        style: Style::EMPTY
    };

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub fn new<S: AsRef<str>>(grapheme: S) -> Self {
        let grapheme = CompactString::new(grapheme);
        let width = grapheme.cluster_display_width();
        Self {
            grapheme,
            width,
            ..Self::EMPTY
        }
    }

    pub fn new_const(grapheme: &'static str) -> Self {
        Self {
            grapheme: CompactString::const_new(grapheme),
            ..Self::EMPTY
        }
    }

    pub fn grapheme(&self) -> &CompactString {
        &self.grapheme
    }

    pub fn set(&mut self, grapheme: impl AsRef<str>) {
        self.grapheme.clear();
        self.grapheme.insert_str(0, grapheme.as_ref());
        self.width = self.grapheme.cluster_display_width();
    }

    pub fn set_char(&mut self, char: char) {
        self.grapheme.clear();
        self.grapheme.insert(0, char);
        self.width = self.grapheme.cluster_display_width();
    }
    
    pub fn set_space(&mut self) {
        self.grapheme.clear();
        self.grapheme.insert_str(0, Self::SPACE);
        self.width = self.grapheme.cluster_display_width();
    }

    pub fn set_style(&mut self, style: &Style) {
        self.style = style.clone();
    }

    pub fn is_empty(&self) -> bool {
        self.grapheme == Self::SPACE
    }

    pub fn is_unstyled(&self) -> bool {
        self.style.is_empty()
    }

    pub fn as_style(&self) -> &Style {
        &self.style
    }

    pub fn as_str(&self) -> &str {
        self.grapheme.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.grapheme.as_bytes()
    }

    pub fn clear(&mut self) -> &mut Self {
        self.set_space();
        self.style.clear();
        self
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
        w.write_escape(&self.style)?;
        w.write(self.as_bytes())?;
        Ok(())
    }
}
