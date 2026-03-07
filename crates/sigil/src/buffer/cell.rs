use std::fmt::Debug;
use unicode_width::UnicodeWidthChar;
use ansi::Style;
use crate::Offset;
use super::{Graph, Grapheme, GraphemeArena};

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
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
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
    /// - `0` is treated as `1` by [`columns()`](Self::columns)
    ///
    /// Wide characters (width 2) occupy this cell and the next cell to the
    /// right, which should be a "continuation" cell with an empty grapheme.
    width: u8,

    /// Visual style: text attributes, foreground and background colors.
    style: Style,
}

impl Cell {
    /// An empty cell with no grapheme and default style.
    pub const EMPTY: Self = Self {
        grapheme: Grapheme::EMPTY,
        width: 0,
        style: Style::EMPTY,
    };

    /// Create a new cell.
    pub fn new(grapheme: Grapheme, width: u8, style: Style) -> Self {
        Self {
            grapheme,
            width,
            style,
        }
    }

    /// Create a cell from a character with the given style.
    ///
    /// The character is always stored inline (every Unicode scalar value fits
    /// in 4 UTF-8 bytes).
    pub fn from_char(char: char, style: Style) -> Self {
        Self {
            grapheme: Grapheme::from_char(char),
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

    /// The effective column count: `width`, or 1 if `width` is 0.
    #[inline]
    pub fn columns(&self) -> u8 {
        if self.width == 0 { 1 } else { self.width }
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

    /// Returns `true` if this cell has no grapheme (blank) or style (default).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.grapheme.is_empty() && self.is_unstyled()
    }

    /// Returns `true` if this cell has no style (default).
    #[inline]
    pub fn is_unstyled(&self) -> bool {
        self.style.is_empty()
    }

    /// Replace the grapheme, releasing the old one from the arena if needed.
    pub fn set_grapheme(&mut self, grapheme: Grapheme, arena: &mut GraphemeArena) {
        self.grapheme.release(arena);
        self.grapheme = grapheme;
    }

    /// Set grapheme, width, and style atomically.
    pub fn set(&mut self, grapheme: Grapheme, width: u8, style: Style) {
        self.grapheme = grapheme;
        self.width = width;
        self.style = style;
    }

    /// Set this cell to a width-1 space with the given style.
    pub fn set_space(&mut self, style: Style) {
        self.grapheme = Grapheme::SPACE;
        self.width = 1;
        self.style = style;
    }

    /// Set this cell as a width-0 continuation cell with the given style.
    pub fn set_continuation(&mut self, style: Style) {
        self.grapheme = Grapheme::EMPTY;
        self.width = 0;
        self.style = style;
    }
    
    pub fn set_char(&mut self, char: char) {
        self.grapheme = Grapheme::from_char(char);
    }
    
    /// Release any arena storage held by this cell's grapheme.
    ///
    /// Call this before the cell is dropped or overwritten if its grapheme
    /// may be arena-stored. No-op for inline and empty graphemes.
    pub fn release(&mut self, arena: &mut GraphemeArena) {
        self.grapheme.release(arena);
        self.grapheme = Grapheme::EMPTY;
    }

    /// Set the column width.
    #[inline]
    pub fn set_width(&mut self, width: u8) {
        self.width = width;
    }

    /// Set the visual style.
    #[inline]
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }


    /// Reset this cell to empty (no grapheme, default style).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release) first if needed.
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// Resolve the grapheme to a readable string.
    ///
    /// Shorthand for `self.grapheme().resolve(arena)`.
    pub fn as_str<'a>(&'a self, arena: &'a GraphemeArena) -> &'a str {
        self.grapheme.as_str(arena)
    }

    /// Resolve the grapheme to a [`Graph`].
    pub fn as_graph<'a>(&self, arena: &'a GraphemeArena) -> Graph<'a> {
        self.grapheme.as_graph(arena)
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        if self.is_empty() {
            return f.write_str("Cell::EMPTY")
        }
        let mut debug = f.debug_tuple("Cell");

        if self.is_continuation() {
            debug.field(&"");
        } else {
            debug.field(&self.grapheme);
        }

        if !self.is_unstyled() {
            debug.field(&self.style);
        }

        debug.finish()
    }
}

impl Offset for Cell {
    #[inline]
    fn offset(self) -> usize {
        self.grapheme.offset()
    }
}
impl Offset for &Cell {
    #[inline]
    fn offset(self) -> usize {
        self.grapheme.offset()
    }
}

impl Offset for &mut Cell {
    #[inline]
    fn offset(self) -> usize {
        self.grapheme.offset()
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
        assert_eq!(cell.columns(), 1);
        assert_eq!(cell.width(), 0);
    }

    #[test]
    fn cell_from_char() {
        let style = Style::new()
            .attributes(Attribute::Bold)
            .foreground(Color::Rgb(255, 0, 0));

        let cell = Cell::from_char('A', style);
        assert!(!cell.is_empty());
        assert_eq!(cell.width(), 1);
        assert_eq!(cell.columns(), 1);
        assert_eq!(cell.style().fg, Color::Rgb(255, 0, 0));
        assert!(cell.style().attributes.contains(Attribute::Bold));

        let arena = GraphemeArena::new();
        assert_eq!(cell.as_str(&arena), "A");
    }

    #[test]
    fn cell_with_wide_char() {
        let cell = Cell::from_char('中', Style::EMPTY);
        assert_eq!(cell.columns(), 2);
    }

    #[test]
    fn cell_replace_grapheme() {
        let mut arena = GraphemeArena::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";

        let g = Grapheme::encode(family, &mut arena);
        let mut cell = Cell::new(g, 2, Style::EMPTY);

        assert_eq!(cell.as_graph(&arena), family);
        assert!(!arena.is_empty());

        // Replace with an inline grapheme — old one gets released.
        let g2 = Grapheme::from_char('X');
        cell.set_grapheme(g2, &mut arena);

        assert_eq!(cell.as_graph(&arena), "X");
        assert!(arena.is_empty()); // Pool storage was freed.
    }

    #[test]
    fn cell_clear() {
        let cell_before = Cell::from_char('Z', Style::new().foreground(Color::Index(1)));
        let mut cell = cell_before;
        cell.clear();
        assert!(cell.is_empty());
        assert_eq!(cell, Cell::EMPTY);
    }
}