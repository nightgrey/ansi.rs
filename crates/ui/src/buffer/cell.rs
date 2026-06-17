use super::{Arena, Grapheme};
use crate::{Encodeable, Offsetted};
use ansi::{Attribute, Color, Style};
use maybe::Maybe;
use std::fmt::{Debug, from_fn};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// Compile-time size check
const _: () = assert!(core::mem::size_of::<Cell>() == 16);

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
#[derive(Copy)]
#[derive_const(Clone, Eq, PartialEq)]
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

impl const Cell {
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
    pub fn is_continuation(&self) -> bool {
        self.grapheme == Grapheme::CONTINUATION
    }

    /// Returns the number of grid columns the cursor advances.
    ///
    /// Used for diffing, run iteration and presentation.
    #[inline]
    pub fn advance(&self) -> usize {
        // This is the cell's [`width`](Self::width) clamped to at least `1`:
        // cleared cells are zero-width but still occupy a single column, so they
        // must advance the cursor. This is the single source of truth for the
        // "a base cell occupies `max(width, 1)` columns" rule shared by diffing,
        // run iteration, and presentation. Continuation cells are not base cells
        // and should be skipped via [`is_continuation`](Self::is_continuation)
        // rather than advanced over with this.
        (self.width as usize).max(1)
    }

    /// Returns `true` if this cell's grapheme is empty (and would be rendered as a space).
    #[inline]
    pub fn is_space(&self) -> bool {
        self.grapheme == Grapheme::EMPTY
    }

    /// Returns `true` if this cell has nothing to draw: no glyph *and* no style.
    ///
    /// An empty cell that carries a style (e.g. a background colour) must still
    /// be painted as a styled space.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.is_space() && self.style.is_empty()
    }

    /// Check if this content is the default value.
    ///
    /// This is equivalent to `is_empty()` and primarily exists for readability in tests.
    #[inline]
    pub fn is_default(self) -> bool {
        self.is_space()
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_foreground(mut self, color: Color) -> Self {
        self.style.foreground = color;
        self
    }

    pub fn with_maybe_foreground(self, color: Option<Color>) -> Self {
        match color {
            Some(color) => self.with_foreground(color),
            None => self,
        }
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.style.background = color;
        self
    }

    pub fn with_maybe_background(self, color: Option<Color>) -> Self {
        match color {
            Some(color) => self.with_background(color),
            None => self,
        }
    }

    pub fn with_attributes(mut self, attribute: Attribute) -> Self {
        self.style.attributes = attribute;
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

    /// Reset this cell to default (empty space).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release) first if needed.
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }
    pub fn eq_bitwise(&self, other: &Self) -> bool {
        (self.grapheme == other.grapheme)
            & (self.style.foreground == other.style.foreground)
            & (self.style.background == other.style.background)
            & (self.style.attributes == other.style.attributes)
    }
}

impl Cell {
    /// Create a cell from a character with the given style.
    ///
    /// The character is always stored inline (every Unicode scalar value fits
    /// in 4 UTF-8 bytes).
    pub fn inline_measured(encode: impl Encodeable, width: usize) -> Self {
        Self {
            grapheme: Grapheme::inline(encode),
            width: width as u8,
            style: Style::None,
        }
    }

    /// Reset this cell to default (empty space).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release) first if needed.
    pub fn clear_and_release(&mut self, arena: &mut Arena) {
        self.release(arena);
        *self = Self::EMPTY;
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

    pub fn char(char: char) -> Self {
        Self::inline(char)
    }

    pub fn char_measured(char: char, width: usize) -> Self {
        Self::inline_measured(char, width)
    }

    /// Create a cell from a character with the given style.
    ///
    /// The character is always stored inline (every Unicode scalar value fits
    /// in 4 UTF-8 bytes).
    pub fn inline(encode: impl Encodeable) -> Self {
        let width = encode.width();
        Self::inline_measured(encode, width)
    }

    pub fn extended(encode: impl Encodeable, arena: &mut Arena) -> Self {
        let width = encode.width();
        Self::extended_measured(encode, width, arena)
    }

    pub fn extended_measured(encode: impl Encodeable, width: usize, arena: &mut Arena) -> Self {
        Self {
            grapheme: Grapheme::extended(encode, arena),
            width: width as u8,
            style: Style::None,
        }
    }

    pub fn set_char(&mut self, char: char, arena: &mut Arena) -> &mut Self {
        *self = self.with_char(char, arena);
        self
    }

    pub fn set_str(&mut self, str: &str, arena: &mut Arena) -> &mut Self {
        *self = self.with_str(str, arena);
        self
    }

    pub fn with_char(self, char: char, arena: &mut Arena) -> Self {
        self.with_char_measured(char, char.width().unwrap_or(0), arena)
    }

    pub fn with_str(self, str: &str, arena: &mut Arena) -> Self {
        self.with_str_measured(str, str.width(), arena)
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

impl const Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_space() && self.style.is_empty() {
            return f.write_str("Cell::Empty");
        }

        let bg = self.style.background.maybe();
        let fg = self.style.foreground.maybe();
        let attr = self.style.attributes.maybe();
        let grapheme = (!self.grapheme.is_empty()).then_some(self.grapheme);

        let g = from_fn(|f| match grapheme.or(Some(Grapheme::EMPTY)) {
            Some(grapheme) if grapheme.is_inline() => f.write_str(grapheme.as_inline_str()),
            Some(grapheme) if grapheme.is_extended() => {
                write!(f, "Offset")
            }
            Some(grapheme) if grapheme.is_continuation() => f.write_str(".."),
            _ => unreachable!(),
        });

        match (bg, fg, attr) {
            (Some(bg), Some(fg), attr) => {
                let mut debug = f.debug_struct("Cell");

                if !self.grapheme.is_empty() {
                    debug.field("grapheme", &g);
                }
                debug.field("background", &bg);
                debug.field("foreground", &fg);

                if attr.is_some() {
                    debug.field("attributes", &attr.unwrap());
                }

                debug.finish()
            }
            (bg, fg, attr) => {
                let mut debug = f.debug_tuple(if bg.is_some() {
                    "Cell::Background"
                } else if fg.is_some() {
                    "Cell::Foreground"
                } else if attr.is_some() {
                    "Cell::Attributes"
                } else {
                    "Cell"
                });

                if !self.grapheme.is_empty() {
                    debug.field(&g);
                }

                if bg.is_some() {
                    debug.field(&bg.unwrap());
                }

                if fg.is_some() {
                    debug.field(&fg.unwrap());
                }
                if attr.is_some() {
                    debug.field(&attr.unwrap());
                }

                debug.finish()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Attribute, Color};

    use super::*;

    #[test]
    fn empty_cell() {
        let cell = Cell::EMPTY;
        assert!(cell.is_space());
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
        let cell = Cell::inline('A')
            .with_attributes(Attribute::Bold)
            .with_foreground(Color::Rgb(255, 0, 0));
        assert!(!cell.is_space());
        assert_eq!(cell.width(), 1);
        assert_eq!(cell.style().foreground, Color::Rgb(255, 0, 0));
        assert!(cell.style().attributes.contains(Attribute::Bold));

        let arena = Arena::new();
        assert_eq!(cell.as_str(&arena), "A");
    }

    #[test]
    fn cell_with_wide_char() {
        let cell = Cell::inline('中');
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
        let cell_before = Cell::inline('Z').with_foreground(Color::Index(1));
        let mut cell = cell_before;
        cell.clear();
        assert!(cell.is_space());
        assert_eq!(cell, Cell::EMPTY);
    }
}
