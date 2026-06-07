use crate::{Arena, Cell};
use derive_more::{AsRef, Deref, DerefMut, From, Index, IndexMut, IntoIterator};
use ansi::Style;

/// A slice of cells
///
/// Provides common methods for running over, walking over and accessing cells - for rendering,
/// writing, diffing and similar purposes.
#[derive(Debug, Deref, DerefMut, AsRef, Clone, From, IntoIterator, Index)]
#[repr(transparent)]
pub struct Cells<'a>(pub &'a [Cell]);

impl<'a> Cells<'a> {
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
    pub fn run(&self) -> CellsIter<'a> {
        CellsIter {
            cells: self,
            idx: 0,
            col: 0,
        }
    }

    /// Iterator over the column advances of each cell in this row.
    ///
    /// See [`Cell::advance`].
    #[inline]
    pub fn map_advance(&self) -> impl Iterator<Item=usize> + '_ {
        self.iter().map(Cell::advance)
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
    pub fn last(&self) -> Option<usize> {
        self.iter().rposition(Cell::is_empty)
    }
}


/// A mutable of cells
///
/// Provides common methods for writing - for rendering, diffing and similar purposes.
#[derive(Debug, Deref, DerefMut, AsRef, From, IntoIterator, Index, IndexMut)]
#[repr(transparent)]
pub struct CellsMut<'a>(pub &'a mut [Cell]);

impl<'a> CellsMut<'a> {
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
    pub fn write(&mut self, grapheme: &str, width: usize, arena: &mut Arena) -> usize {
        let span = width.max(1);
        if let Some((base, rest)) = self.0.split_first_mut() {
            base.set_str_measured(grapheme, width, arena);
            for cell in rest.iter_mut().take(span - 1) {
                *cell = Cell::CONTINUATION;
            }
        }
        span
    }


    /// Writes a measured grapheme.
    pub fn write_styled(&mut self, grapheme: &str, width: usize, style: Style, arena: &mut Arena) -> usize {
        let span = width.max(1);
        if let Some((base, rest)) = self.0.split_first_mut() {
             base.set_str_measured(grapheme, width, arena).set_style(style);
            for cell in rest.iter_mut().take(span - 1) {
                *cell = Cell::CONTINUATION;
            }
        }
        span
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
