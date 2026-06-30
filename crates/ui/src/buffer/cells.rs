//! Row-writing helpers and cell iteration.
//!
//! [`Cells`] is a stateless namespace that provides the shared logic for
//! writing graphemes and strings into cell slices. It also houses the
//! canonical cell iterator ([`CellsIter`]) — the single source of truth
//! for walking a row's cells in column order, correctly skipping continuation
//! cells and advancing over cleared slots.
//!
//! # Wide-cell invariant
//!
//! A wide character (e.g. `'中'`, display width 2) occupies a *base cell*
//! followed by one or more *continuation cells*. [`CellsIter`] skips
//! continuations so consumers see each distinct character exactly once.
//! [`write_grapheme`](Cells::write_grapheme) enforces this invariant when
//! building rows.

use crate::{Cell, Grapheme, Graphemes, IntoGraphemeWidth};
use ansi::Style;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A stateless namespace of row-level operations on cell slices.
///
/// All methods accept `&[Cell]` / `&mut [Cell]` and operate on a single
/// buffer row at a time. The caller is responsible for slicing out the
/// desired row before calling these helpers.
#[derive(Debug)]
pub struct Cells;

impl Cells {
    /// Iterates the base cells of a row slice in column order.
    ///
    /// This is the single source of truth for walking a row's cells: continuation
    /// cells (the trailing positions of a wide grapheme) are skipped because they
    /// are implicitly covered by their base cell, and each base cell advances the
    /// column cursor by [`Cell::advance`] so cleared cells still occupy a column.
    ///
    /// Each item is `(column, cell)` where `column` is the cell's starting column
    /// relative to the start of the slice.
    #[inline]
    pub fn run(cells: &[Cell]) -> CellsIter<'_> {
        CellsIter {
            cells,
            idx: 0,
            col: 0,
        }
    }

    /// Iterator over the column advances of each cell in this row.
    ///
    /// See [`Cell::advance`].
    #[inline]
    pub fn advance(cells: &[Cell]) -> impl Iterator<Item = usize> + '_ {
        cells.iter().map(Cell::advance)
    }

    /// Index of the last cell in `row` that must be drawn, or `None` if the row
    /// is entirely blank.
    ///
    /// This is the single source of truth for "where does a row's drawable
    /// content end" — used to trim trailing blanks before an erase-to-end.
    /// A cleared cell that carries a style (e.g. a background colour) is *not*
    /// blank and must still be painted, so this tests [`is_empty`](Self::is_empty)
    /// (glyph *and* style absent), not [`is_space`](Self::is_space).
    #[inline]
    pub fn last(cells: &[Cell]) -> Option<usize> {
        cells.iter().rposition(Cell::is_empty)
    }

    /// Writes a measured grapheme as a cell at `cells[0]` and fills the
    /// following `width - 1` cells with continuations, establishing the
    /// wide-cell invariant in one place. Returns the column span written
    /// (`max(width, 1)`) — how far the writing cursor advances.
    ///
    /// A zero-width grapheme still consumes its base slot (span `1`).
    /// Continuation fill is clamped to `cells`, so a slice shorter than the
    /// grapheme's width (a clip at the row edge) truncates instead of panicking.
    /// The base cell keeps its existing style; callers that style text should
    /// apply it to `cells[0]` afterwards.
    pub fn write_grapheme(
        cells: &mut [Cell],
        grapheme: &str,
        width: usize,
        style: Option<Style>,
        graphemes: &mut Graphemes,
    ) -> usize {
        let span = width.max(1);

        if let Some((cell, rest)) = cells.split_first_mut() {
            cell.set(Grapheme::new((grapheme, graphemes)), width);
            if let Some(style) = style {
                cell.set_style(style);
            }
            for cell in rest.iter_mut().take(span - 1) {
                *cell = Cell::CONTINUATION;
            }
        }
        span
    }

    /// Writes a string left-to-right, delegating each
    /// individual write (and the wide-cell invariant) to [`write_grapheme`].
    /// The cursor advances by each grapheme's measured width; zero-width
    /// graphemes still advance by 1 via `write_grapheme`'s own `span` floor.
    ///
    /// Stops as soon as `cells` is exhausted, so a string wider than the
    /// available space is clipped — the last grapheme that straddles the edge
    /// is written but its continuation cells are truncated by `write_grapheme`.
    /// Returns the total column span actually written (`<= cells.len()`).
    ///
    /// All graphemes share `style`; pass `None` to leave existing styles
    /// untouched, matching `write_grapheme`'s contract.
    pub fn write(
        cells: &mut [Cell],
        str: &str,
        style: Option<Style>,
        graphemes: &mut Graphemes,
    ) -> usize {
        let mut written = 0;
        let mut rest = cells;

        for (grapheme, width) in str
            .graphemes(true)
            .filter(|g| !g.contains(char::is_control))
            .map(|g| (g, g.width()))
            .filter(|&(_, width)| width > 0)
        {
            let span = width.max(1);

            if span > rest.len() {
                break;
            }
            let advance = Cells::write_grapheme(rest, grapheme, width, style, graphemes);
            written += advance;
            rest = &mut rest[advance..];
        }

        written
    }
}

/// Iterator over the cells of a row slice, in left-to-right column order.
#[derive(Clone, Debug)]
pub struct CellsIter<'a> {
    cells: &'a [Cell],
    idx: usize,
    col: u16,
}

impl<'a> CellsIter<'a> {
    #[inline]
    pub fn new(cells: &'a [Cell]) -> Self {
        Self {
            cells,
            idx: 0,
            col: 0,
        }
    }
}

impl<'a> Iterator for CellsIter<'a> {
    type Item = (u16, &'a Cell);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let cell = self.cells.get(self.idx)?;

            // Continuations are absorbed by their base cell. Empty (cleared)
            // cells are *not* continuations and must be yielded.
            if cell.is_continuation() {
                self.idx += 1;
                continue;
            }

            let col = self.col;
            let advance = cell.advance();
            self.col += advance as u16;
            self.idx += advance;
            return Some((col, cell));
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // The remaining slice could be entirely continuations, so the lower
        // bound is 0; each base cell consumes at least one slot.
        (0, Some(self.cells.len().saturating_sub(self.idx)))
    }
}

impl std::iter::FusedIterator for CellsIter<'_> {}
