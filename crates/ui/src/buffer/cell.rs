use super::{Grapheme, Graphemes};
use crate::{IntoGrapheme, IntoGraphemeWidth};
use ansi::{Attribute, Color, Style};
use maybe::Maybe;
use std::fmt::{Debug, from_fn};

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
/// │ grapheme   │ width │ attributes  │ colors (fg|bg)        │
/// │ 4 bytes    │ 1 B   │ 2 bytes + 1 │ 8 bytes               │
/// └────────────┴───────┴─────────────┴───────────────────────┘
/// = 16 bytes with repr(C) and careful alignment
/// ```
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

impl Cell {
    /// An empty cell — drawn as a plain space with default style.
    ///
    /// The cell has width `1` (so it advances the cursor) and an empty grapheme,
    /// which renders as a space character.
    pub const EMPTY: Self = Self {
        grapheme: Grapheme::EMPTY,
        width: 1,
        style: Style::None,
    };

    /// A continuation cell — the trailing slot of a wide grapheme.
    ///
    /// Width `0`, empty grapheme. Continuation cells are skipped by
    /// [`CellsIter`] and should never be independently styled or written.
    pub const CONTINUATION: Self = Self {
        grapheme: Grapheme::EMPTY,
        width: 0,
        style: Style::None,
    };

    /// Alias for [`EMPTY`](Self::EMPTY).
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// Create a new cell from a grapheme source with measured width.
    pub fn new(grapheme: impl IntoGrapheme + IntoGraphemeWidth) -> Self {
        let width = grapheme.width() as u8;
        Self {
            grapheme: Grapheme::new(grapheme),
            width,
            style: Style::None,
        }
    }

    /// Create a cell with a pre-measured width (in columns).
    ///
    /// Unlike [`new`](Self::new), this does not call [`IntoGraphemeWidth`];
    /// the caller supplies the column width directly. Useful when the width
    /// has already been computed or the grapheme source doesn't implement
    /// `IntoGraphemeWidth`.
    pub const fn new_measured(grapheme: impl [const] IntoGrapheme, width: usize) -> Self {
        Self {
            grapheme: Grapheme::new(grapheme),
            width: width as u8,
            style: Style::None,
        }
    }

    /// The cell's grapheme cluster handle.
    ///
    /// See [`Grapheme`] for the inline/extended encoding details. Use
    /// [`as_str`](Self::as_str) to resolve to a `&str`.
    #[inline]
    pub const fn grapheme(&self) -> Grapheme {
        self.grapheme
    }

    /// The display width of this cell's grapheme in terminal columns.
    ///
    /// - `1` for ASCII and most single-width characters
    /// - `2` for CJK ideographs, fullwidth forms, and most emoji
    /// - `0` for continuation cells
    #[inline]
    pub const fn width(&self) -> u8 {
        self.width
    }

    /// The cell's visual style (attributes, foreground, background).
    #[inline]
    pub const fn style(&self) -> Style {
        self.style
    }

    /// Returns `true` if this is a continuation cell (width 0, empty grapheme).
    ///
    /// Continuation cells are the trailing positions of a wide character.
    /// They should be skipped by iterators and never independently styled.
    #[inline]
    pub const fn is_continuation(&self) -> bool {
        self.grapheme == Grapheme::EMPTY && self.width == 0
    }

    /// Returns `true` if this cell's grapheme is empty (and would be rendered as a space).
    #[inline]
    pub const fn is_blank(&self) -> bool {
        self.grapheme == Grapheme::EMPTY && self.width == 1
    }

    /// Returns `true` if this cell has nothing to draw: no glyph *and* no style.
    ///
    /// An empty cell that carries a style (e.g. a background colour) must still
    /// be painted as a styled space, so it is **not** empty by this definition.
    /// Use this to find the last drawable cell in a row (trim trailing blanks).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.is_blank() && self.style.is_empty()
    }

    /// Check if this cell has the default value (empty space).
    ///
    /// Equivalent to [`is_space`](Self::is_blank) — exists primarily for
    /// readability in tests.
    #[inline]
    pub const fn is_default(&self) -> bool {
        self.is_blank()
    }

    /// Replace the grapheme and width in-place. Returns `self` for chaining.
    pub fn set(&mut self, grapheme: impl IntoGrapheme, width: usize) -> &mut Self {
        self.grapheme = grapheme.into_grapheme();
        self.width = width as u8;
        self
    }

    /// Create a new cell with the given grapheme and width (builder pattern).
    pub fn with(mut self, grapheme: impl IntoGrapheme, width: usize) -> Self {
        self.grapheme = grapheme.into_grapheme();
        self.width = width as u8;
        self
    }

    /// Replace the cell's full style in-place. Returns `self` for chaining.
    pub const fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    /// Builder: set the full style.
    pub const fn with_style(mut self, style: Style) -> Self {
        self.set_style(style);
        self
    }

    /// Set the foreground colour in-place. Returns `self` for chaining.
    pub const fn set_foreground(&mut self, color: Color) -> &mut Self {
        self.style.foreground = color;
        self
    }

    /// Builder: set the foreground colour.
    pub const fn with_foreground(mut self, color: Color) -> Self {
        self.set_foreground(color);
        self
    }

    /// Set the background colour in-place. Returns `self` for chaining.
    pub const fn set_background(&mut self, color: Color) -> &mut Self {
        self.style.background = color;
        self
    }

    /// Builder: set the background colour.
    pub const fn with_background(mut self, color: Color) -> Self {
        self.set_background(color);
        self
    }

    /// Set text attributes in-place. Returns `self` for chaining.
    pub const fn set_attributes(&mut self, attribute: Attribute) -> &mut Self {
        self.style.attributes = attribute;
        self
    }

    /// Builder: set text attributes.
    pub const fn with_attributes(mut self, attribute: Attribute) -> Self {
        self.set_attributes(attribute);
        self
    }

    /// Reset this cell to default (empty space).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release) first if needed.
    pub const fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// Resolve the cell's grapheme to a `&str`.
    ///
    /// Empty cells yield `" "` (a single space). Inline graphemes read
    /// zero-copy from the cell; extended graphemes borrow from `arena`.
    pub fn as_str<'a>(&'a self, graphemes: &'a Graphemes) -> &'a str {
        self.grapheme.as_str_or(graphemes, " ")
    }

    /// Returns the number of grid columns the cursor advances.
    ///
    /// Used for diffing, run iteration and presentation.
    #[inline]
    pub const fn advance(&self) -> usize {
        // This is the cell's [`width`](Self::width) clamped to at least `1`:
        // cleared cells are zero-width but still occupy a single column, so they
        // must advance the cursor. This is the single source of truth for the
        // "a base cell occupies `max(width, 1)` columns" rule shared by diffing,
        // run iteration, and presentation. Continuation cells are not base cells
        // and should be skipped via [`is_continuation`](Self::is_continuation)
        // rather than advanced over with this.
        (self.width as usize).max(1)
    }

    /// Bitwise equality — compares all fields without relying on `PartialEq`.
    ///
    /// Unlike the derived `PartialEq`, this uses bitwise `&` rather than `&&`
    /// to combine field comparisons, giving branch-free equality checks.
    pub const fn eq_bitwise(&self, other: &Self) -> bool {
        (self.grapheme == other.grapheme)
            & (self.style.foreground == other.style.foreground)
            & (self.style.background == other.style.background)
            & (self.style.attributes == other.style.attributes)
    }
}

impl Cell {}

const impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_blank() && self.style.is_empty() {
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
        assert!(cell.is_blank());
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
        let cell = Cell::new('A')
            .with_attributes(Attribute::Bold)
            .with_foreground(Color::Rgb(255, 0, 0));
        assert!(!cell.is_blank());
        assert_eq!(cell.width(), 1);
        assert_eq!(cell.style().foreground, Color::Rgb(255, 0, 0));
        assert!(cell.style().attributes.contains(Attribute::Bold));

        let arena = Graphemes::new();
        assert_eq!(cell.as_str(&arena), "A");
    }

    #[test]
    fn cell_with_wide_char() {
        let cell = Cell::new('中');
        assert_eq!(cell.width(), 2);
    }

    #[test]
    fn cell_clear() {
        let cell_before = Cell::new('Z').with_foreground(Color::Index(1));
        let mut cell = cell_before;
        cell.clear();
        assert!(cell.is_blank());
        assert_eq!(cell, Cell::EMPTY);
    }
}
