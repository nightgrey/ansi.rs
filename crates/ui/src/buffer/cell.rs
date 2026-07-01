use super::{Grapheme, Graphemes};
use crate::{Source};
use ansi::{Attribute, Color, Style};
use maybe::Maybe;
use std::fmt::{Debug, from_fn};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// Compile-time size check
const _: () = assert!(size_of::<Cell>() == 16);

/// Cell
///
/// The fundamental frame buffer unit. It holds a grapheme cluster, its width, and a visual style.
#[derive(Copy)]
#[derive_const(Clone, Eq, PartialEq)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    ///
    /// 4 bytes: either inline UTF-8 or a arena offset (see [`Grapheme`]).
    grapheme: Grapheme,

    /// Display width of this cell's grapheme.
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
    pub fn new(grapheme: impl MeasurableSource) -> Self {
        let width = grapheme.width();
        Self {
            grapheme: Grapheme::new(grapheme),
            width,
            style: Style::None,
        }
    }

    pub const fn new_measured(grapheme: impl [const] Source, width: usize) -> Self {
        Self {
            grapheme: Grapheme::new(grapheme),
            width: width as u8,
            style: Style::None,
        }
    }

    pub const fn from_grapheme(grapheme: Grapheme, width: usize) -> Self {
        Self {
            grapheme,
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

    /// The display width of this cell's grapheme.
    ///
    /// Note: For continuation cells, this returns `0`.
    #[inline]
    pub const fn display_width(&self) -> usize {
        self.width as usize
    }

    /// The column width of this cell.
    ///
    /// Note: For continuation cells, this returns `1`.
    #[inline]
    pub const fn column_width(&self) -> usize {
        self.width.max(1) as usize
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

    /// Returns `true` if this cell renders a space.
    ///
    /// Such a cell carries no grapheme, but it may still have a style and be
    /// painted as a space.
    #[inline]
    pub const fn is_blank(&self) -> bool {
        self.grapheme == Grapheme::EMPTY && self.width == 1
    }

    /// Returns `true` if this cell is completely empty — no glyph **and** no style.
    ///
    /// Use this to find the last drawable cell in a row (trim trailing blanks).
    /// An empty cell that carries a style (e.g., a background) must still be
    /// painted, so it is **not** empty by this definition.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.is_blank() && self.style.is_empty()
    }

    /// Replace the grapheme and width in-place.
    pub fn set(&mut self, grapheme: impl Source, width: usize) -> &mut Self {
        self.grapheme = grapheme.into();
        self.width = width as u8;
        self
    }

    /// Create a new cell with the given grapheme and width (builder pattern).
    pub fn with(mut self, grapheme: impl Source, width: usize) -> Self {
        self.set(grapheme, width);
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

    /// Resolve the cell's grapheme to a `&str`.
    ///
    /// Empty cells yield `" "` (a single space). Inline graphemes read
    /// zero-copy from the cell; extended graphemes borrow from `arena`.
    pub fn as_str<'a>(&'a self, graphemes: &'a Graphemes) -> &'a str {
        self.grapheme.as_str_or(graphemes, " ")
    }

    /// Reset this cell to default (empty space).
    ///
    /// Does **not** release arena storage — call
    /// [`release`](Self::release) first if needed.
    pub const fn clear(&mut self) {
        *self = Self::EMPTY;
    }

}

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

pub trait MeasurableSource: Source {
    fn width(&self) -> u8;
}

impl MeasurableSource for char {
    fn width(&self) -> u8 {
        UnicodeWidthChar::width(*self).unwrap_or(1) as u8
    }
}

impl MeasurableSource for &str {
    fn width(&  self) -> u8 {
        UnicodeWidthStr::width(*self) as u8
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
        assert_eq!(cell.display_width(), 1);
    }

    #[test]
    fn continuation_cell() {
        let cell = Cell::CONTINUATION;
        assert!(cell.is_continuation());
        assert_eq!(cell.display_width(), 0);
    }

    #[test]
    fn cell_from_char() {
        let cell = Cell::new('A')
            .with_attributes(Attribute::Bold)
            .with_foreground(Color::Rgb(255, 0, 0));
        assert!(!cell.is_blank());
        assert_eq!(cell.display_width(), 1);
        assert_eq!(cell.style().foreground, Color::Rgb(255, 0, 0));
        assert!(cell.style().attributes.contains(Attribute::Bold));

        let arena = Graphemes::new();
        assert_eq!(cell.as_str(&arena), "A");
    }

    #[test]
    fn cell_with_wide_char() {
        let cell = Cell::new('中');
        assert_eq!(cell.display_width(), 2);
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
