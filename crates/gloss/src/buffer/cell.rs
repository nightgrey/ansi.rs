use super::{Grapheme, Arena};
use crate::Offset;
use ansi::Style;
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
#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// - `0` is treated as `1` by [`columns()`](Self::width)
    ///
    /// Wide characters (width 2) occupy this cell and the next cell to the
    /// right, which should be a "continuation" cell with an empty grapheme.
    width: u8,

    /// Visual style: text attributes, foreground and background colors.
    pub style: Style,
}

impl Cell {
    /// An empty cell with no grapheme and default style.
    pub const EMPTY: Self = Self {
        grapheme: Grapheme::SPACE,
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

    /// The grapheme handle for this cell.
    #[inline]
    pub fn grapheme(&self) -> Grapheme {
        self.grapheme
    }

    /// The raw width value (0 means unset).
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
    pub fn is_wide(&self) -> bool {
        self.width >= 2
    }

    #[inline]
    pub fn is_narrow(&self) -> bool {
        self.width <= 1
    }

    #[inline]
    pub fn is_continuation(&self) -> bool {
        self.width == 0
    }

    /// Returns `true` if this cell has no grapheme (blank).
    pub fn is_blank(&self) -> bool {
        self.grapheme.is_empty()
    }

    /// Returns `true` if this cell has no grapheme (blank) or style (default).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self == &Cell::EMPTY
    }

    /// Returns `true` if this cell has no style (default).
    #[inline]
    pub fn is_unstyled(&self) -> bool {
        self.style.is_none()
    }

    pub fn set_char(&mut self, char: char, arena: &mut Arena) -> &mut Self {
        self.set_char_measured(char, char.width().unwrap_or(0), arena)
    }

    pub fn set_str(&mut self, str: &str, arena: &mut Arena) -> &mut Self {
        self.set_str_measured(str, str.width(), arena)
    }

    pub fn set_char_measured(&mut self, char: char, width: usize, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.release(self.grapheme);
        }

        if char == ' ' {
            return self.set_space(arena);
        }

        self.grapheme = Grapheme::inline(char);
        self.width = width as u8;

        self
    }
    pub fn set_str_measured(&mut self, str: &str, width: usize, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.release(self.grapheme);
        }
        self.grapheme = Grapheme::extended(str, arena);
        self.width = width as u8;
        self
    }

    /// Set this cell to a width-1 space with the given style.
    pub fn set_space(&mut self, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.release(self.grapheme);
        }
        self.grapheme = Grapheme::SPACE;
        self.width = 1;
        self
    }

    /// Set this cell as a width-0 continuation cell with the given style.
    pub fn set_continuation(&mut self, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.release(self.grapheme);
        }
        self.grapheme = Grapheme::SPACE;
        self.width = 0;
        self
    }

    pub fn set_style(&mut self, style: Style)  -> &mut Self {
        self.style = style;
        self
    }

    pub fn replace_grapheme(&mut self, grapheme: Grapheme, width: usize, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.release(self.grapheme);
        }
        self.grapheme = grapheme;
        self.width = width as u8;
        self
    }

    /// Release any arena storage held by this cell's grapheme.
    ///
    /// Call this before the cell is dropped or overwritten if its grapheme
    /// may be arena-stored. No-op for inline and empty graphemes.
    pub fn release_grapheme(&mut self, arena: &mut Arena) -> &mut Self {
        if self.grapheme.is_extended() {
            arena.release(self.grapheme);
        }
        self.grapheme = Grapheme::SPACE;
        self.width = 0;
        self
    }

    /// Reset this cell to empty (no grapheme, default style).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release_grapheme) first if needed.
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
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
        let mut debug = f.debug_tuple("Cell");

        if self.is_continuation() {
            debug.field(&" ");
        } else {
            debug.field(&self.grapheme);
        }

        if !self.is_unstyled() {
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
        assert_eq!(cell.width(), 0);
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
