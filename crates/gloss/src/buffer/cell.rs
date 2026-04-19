use super::{Arena, Grapheme};
use crate::AsOffset;
use ansi::{Attribute, Color, Style};
use maybe::Maybe;
use std::fmt::Debug;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A single terminal cell — the fundamental unit of the framebuffer.
///
/// Each cell holds a grapheme cluster (the visible character), its display
/// width in terminal columns, and visual style (attributes + colors).
///
/// # Size target
///
/// When `Style` is finalized into a packed representation (attributes in a
/// `u16`, foreground + background in a `u64` channel pair), this struct will
/// pack to exactly **16 bytes** for cache-line efficiency:
///
/// ```text
/// ┌────────────┬───────┬─────────────┬───────────────────────┐
/// │ grapheme   │ width │ attributes  │ channels (fg|bg)      │
/// │ 4 bytes    │ 1 B   │ 2 bytes + 1 │ 8 bytes               │
/// └────────────┴───────┴─────────────┴───────────────────────┘
/// = 16 bytes with repr(C) and careful alignment
/// ```
///
/// For now, `Style` is a mock and the struct may be slightly larger.
#[derive(Clone, Copy, Eq)]
#[repr(C)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    ///
    /// 4 bytes: either inline UTF-8 or a arena offset (see [`Grapheme`]).
    grapheme: Grapheme,

    /// Column width of this cell's grapheme.
    ///
    /// - `1` for ASCII and most single-width characters
    /// - `2` for CJK ideographs, fullwidth forms, and most emoji
    /// - `0` for continuation cells (no grapheme)
    ///
    /// Wide characters (width 2) occupy this cell and the next cell to the
    /// right, which should be a "continuation" cell with an empty grapheme.
    width: u8,

    /// Visual style: text attributes, foreground and background colors.
    pub style: Style,
}

impl Cell {
    pub const DEFAULT: Self = Self::EMPTY;

    pub const EMPTY: Self = Self {
        grapheme: Grapheme::EMPTY,
        width: 0,
        style: Style::None,
    };

    pub const CONTINUATION: Self = Self {
        grapheme: Grapheme::CONTINUATION,
        width: 0,
        style: Style::None,
    };

    /// Create a new cell.
    pub fn new(grapheme: Grapheme, width: usize, style: Style) -> Self {
        Self {
            grapheme,
            width: width as u8,
            style,
        }
    }

    /// Create a cell from a character with the given style.
    ///
    /// The character is always stored inline (every Unicode scalar value fits
    /// in 4 UTF-8 bytes).
    pub fn inline(char: char, style: Style) -> Self {
        Self {
            grapheme: Grapheme::inline(char),
            width: char.width().unwrap_or(0) as u8,
            style,
        }
    }

    // ── Accessors ──────────────────────────────────────────────────────

    #[inline]
    pub fn grapheme(&self) -> Grapheme {
        self.grapheme
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.width
    }

    /// The cell's visual style.
    #[inline]
    pub fn style(&self) -> &Style {
        &self.style
    }
    
    #[inline]
    pub const fn is_none(&self) -> bool {
        self.grapheme == Grapheme::EMPTY || self.grapheme == Grapheme::CONTINUATION
    }

    #[inline]
    pub const fn is_continuation(&self) -> bool {
        self.grapheme == Grapheme::CONTINUATION
    }

    /// Returns `true` if this cell is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.grapheme == Grapheme::EMPTY
    }

    /// Check if this content is the default value.
    ///
    /// This is equivalent to `is_empty()` and primarily exists for readability in tests.
    #[inline]
    pub const fn is_default(self) -> bool {
        self.is_empty()
    }

    pub fn with_char(mut self, char: char, arena: &mut Arena) -> Self {
        self.with_char_measured(char, char.width().unwrap_or(0), arena)
    }

    pub fn with_str(mut self, str: &str, arena: &mut Arena) -> Self {
        self.with_str_measured(str, str.width(), arena)
    }

    pub fn with_char_measured(mut self, char: char, width: usize, arena: &mut Arena) -> Self {
        if self.grapheme.is_extended() {
            arena.remove(self.grapheme);
        }

        self.grapheme = Grapheme::inline(char);
        self.width = width as u8;

        self
    }

    pub fn with_str_measured(mut self, str: &str, width: usize, arena: &mut Arena) -> Self {
        if self.grapheme.is_extended() {
            arena.remove(self.grapheme);
        }
        self.grapheme = Grapheme::extended(str, arena);
        self.width = width as u8;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_foreground(mut self, color: Color) -> Self {
        self.style.foreground = color;
        self
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.style.background = color;
        self
    }

    pub fn with_attributes(mut self, attribute: Attribute) -> Self {
        self.style.attributes = attribute;
        self
    }

    pub fn set_char(&mut self, char: char, arena: &mut Arena) -> &mut Self {
        *self = self.with_char(char, arena);
        self
    }

    pub fn set_str(&mut self, str: &str, arena: &mut Arena) -> &mut Self {
        *self = self.with_str(str, arena);
        self
    }

    pub fn set_char_measured(&mut self, char: char, width: usize, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.remove(self.grapheme);
        }

        self.grapheme = Grapheme::inline(char);
        self.width = width as u8;

        self
    }

    pub fn set_str_measured(&mut self, str: &str, width: usize, arena: &mut Arena) -> &mut Self {
        *self = self.with_str_measured(str, width, arena);
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        *self = self.with_style(style);
        self
    }

    pub fn set_foreground(&mut self, color: Color) -> &mut Self {
        *self = self.with_foreground(color);
        self
    }

    pub fn set_background(&mut self, color: Color) -> &mut Self {
        *self = self.with_background(color);
        self
    }

    pub fn set_attributes(&mut self, attribute: Attribute) -> &mut Self {
        *self = self.with_attributes(attribute);
        self
    }

    /// Release any arena storage held by this cell's grapheme.
    ///
    /// Call this before the cell is dropped or overwritten if its grapheme
    /// may be arena-stored. No-op for inline and empty graphemes.
    pub fn release(&mut self, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.remove(self.grapheme);
        }
        self
    }

    /// Reset this cell to default (empty space).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release) first if needed.
    pub fn clear(&mut self) {
        *self = Self::DEFAULT;
    }

    /// Resolve the grapheme to a readable string.
    ///
    /// Shorthand for `self.grapheme().resolve(arena)`.
    pub fn as_str<'a>(&'a self, arena: &'a Arena) -> &'a str {
        self.grapheme.as_str(arena)
    }

    pub fn as_bytes<'a>(&'a self, arena: &'a Arena) -> &'a [u8] {
        self.grapheme.as_bytes(arena)
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        (self.grapheme == other.grapheme)
            & (self.style.foreground == other.style.foreground)
            & (self.style.background == other.style.background)
            & (self.style.attributes == other.style.attributes)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("Cell::EMPTY");
        }
        if self.is_continuation() {
            return f.write_str("Cell::CONTINUATION");
        }

        let mut debug = f.debug_tuple("Cell");

        debug.field(&self.grapheme);

        if self.style.is_some() {
            debug.field(&self.style);
        }

        debug.finish()
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Attribute, Color};

    use super::*;

    #[test]
    fn empty_cell() {
        let cell = Cell::EMPTY;
        assert!(cell.is_empty());
        assert_eq!(cell.width(), 1);
    }

    #[test]
    fn continuation_cell() {
        let cell = Cell::CONTINUATION;
        assert!(cell.is_continuation());
        assert_eq!(cell.width(), 0);
    }

    #[test]
    fn cell_from_char() {
        let style = Style::default()
            .with(Attribute::Bold)
            .foreground(Color::Rgb(255, 0, 0));

        let cell = Cell::inline('A', style);
        assert!(!cell.is_empty());
        assert_eq!(cell.width(), 1);
        assert_eq!(cell.style().foreground, Color::Rgb(255, 0, 0));
        assert!(cell.style().attributes.contains(Attribute::Bold));

        let arena = Arena::new();
        assert_eq!(cell.as_str(&arena), "A");
    }

    #[test]
    fn cell_with_wide_char() {
        let cell = Cell::inline('中', Style::None);
        assert_eq!(cell.width(), 2);
    }

    #[test]
    fn cell_replace_grapheme() {
        let mut arena = Arena::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";

        let g = Grapheme::extended(family, &mut arena);
        let mut cell = Cell::new(g, 2, Style::None);

        assert!(!arena.is_empty());

        // Replace with an inline grapheme — old one gets released.
        cell.set_char('X', &mut arena);

        assert!(arena.is_empty()); // Pool storage was freed.
    }

    #[test]
    fn cell_clear() {
        let cell_before = Cell::inline('Z', Style::default().foreground(Color::Index(1)));
        let mut cell = cell_before;
        cell.clear();
        assert!(cell.is_empty());
        assert_eq!(cell, Cell::EMPTY);
    }
}
