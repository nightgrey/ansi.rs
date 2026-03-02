use std::ops::{self, Deref, RangeInclusive};
use std::range::RangeBounds;
use derive_more::{Deref, DerefMut, Index, IntoIterator};
use geometry::{Bounds, Position, PositionLike, Row};
use utils::into_range_unchecked;
use crate::{Buffer, Cell, GraphemeArena};
// ── Dirty column range ──────────────────────────────────────────

#[derive(Debug, Clone, Deref, DerefMut, IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
struct DirtyColumns(Vec<Mark>);

impl DirtyColumns {
    const EMPTY: Self = Self(Vec::new());

    pub fn new(capacity: usize) -> Self {
        Self(vec![Mark::CLEAN; capacity])
    }
}

/// Mark (inclusive)
///
/// Represents a range of columns that have been modified.
///
/// Note: `min > max` means clean.
#[derive(Debug, Clone, Copy)]
struct Mark {
    min: usize,
    max: usize,
}

impl Mark {
    const CLEAN: Self = Self { min: usize::MAX, max: 0 };

    #[inline]
    fn is_clean(self) -> bool {
        self.min > self.max
    }

    #[inline]
    fn set(&mut self, col: usize) {
        let col = col;
        self.min = self.min.min(col);
        self.max = self.max.max(col);
    }

    #[inline]
    fn set_range(&mut self, range: impl RangeBounds<usize>) {
        let range = into_range_unchecked(range, usize::MAX);

        if range.start < range.end {
            self.min = self.min.min(range.start);
            self.max = self.max.max(range.end - 1);
        }
    }

    #[inline]
    fn set_width(&mut self, width: usize) {
        self.min = 0;
        self.max = width.saturating_sub(1);
    }

    #[inline]
    fn into_range(self) -> RangeInclusive<usize> {
        self.min..=self.max
    }
}

impl Into<RangeInclusive<usize>> for Mark {
    fn into(self) -> RangeInclusive<usize> {
        self.into_range()
    }
}

// ── Row bitset ──────────────────────────────────────────────────

/// Compact bitset for tracking which rows are dirty.
#[derive(Debug, Clone, Deref, DerefMut, IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
struct DirtyRows(Vec<u64>);

impl DirtyRows {
    const EMPTY: Self = Self(Vec::new());

    fn new(rows: usize) -> Self {
        Self(vec![0u64; (rows + 63) / 64])
    }

    #[inline]
    fn mark(&mut self, row: usize) {
        self[row / 64] |= 1u64 << (row % 64);
    }

    #[inline]
    fn mark_all(&mut self) {
        self.fill(!0u64);
    }

    #[inline]
    fn is_marked(&self, row: usize) -> bool {
        self[row / 64] & (1u64 << (row % 64)) != 0
    }

    #[inline]
    fn clear(&mut self) {
        self.fill(0);
    }

    #[inline]
    fn any(&self) -> bool {
        self.0.iter().any(|&w| w != 0)
    }

    /// Iterate over set bit indices.
    fn iter(&self) -> DirtyRowsIter<'_> {
        DirtyRowsIter { words: &self, word_idx: 0, remaining: 0 }
    }
}

/// Iterator over set bits in a `RowBitset`.
struct DirtyRowsIter<'a> {
    words: &'a [u64],
    word_idx: usize,
    remaining: u64,
}

impl Iterator for DirtyRowsIter<'_> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        while self.remaining == 0 {
            if self.word_idx >= self.words.len() {
                return None;
            }
            self.remaining = self.words[self.word_idx];
            if self.remaining == 0 {
                self.word_idx += 1;
            }
        }
        let bit = self.remaining.trailing_zeros() as usize;
        self.remaining &= self.remaining - 1; // clear lowest set bit
        let index = self.word_idx * 64 + bit;
        if self.remaining == 0 {
            self.word_idx += 1;
        }
        Some(index)
    }
}

// ── TrackingBuffer ──────────────────────────────────────────────

/// A buffer that tracks which cells have been modified.
///
/// Wraps a [`Buffer`] and maintains a row-level dirty bitset plus
/// per-row dirty column ranges. This allows the rasterizer to:
/// 1. Skip entirely clean rows (bitset check).
/// 2. Narrow the diff window within dirty rows (column range).
///
/// Read access is transparent via `Deref<Target = Buffer>`.
/// Write access goes through tracked methods or `IndexMut` impls
/// that automatically record dirty regions.
#[derive(Debug, Clone, Deref, IntoIterator, Index)]
pub struct TrackingBuffer {
    #[deref]
    #[into_iterator(owned, ref, ref_mut)]
    #[index]
    inner: Buffer,
    rows: DirtyRows,

    cols: DirtyColumns,
}

impl TrackingBuffer {
    pub const EMPTY: Self = Self {
        inner: Buffer::EMPTY,
        rows: DirtyRows::EMPTY,
        cols: DirtyColumns::EMPTY,
    };

    /// Creates a new tracking buffer with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: Buffer::new(width, height),
            rows: DirtyRows::new(height),
            cols: DirtyColumns::new(height),
        }
    }

    /// Creates a tracking buffer with an existing grapheme arena.
    pub fn with_arena(width: usize, height: usize, arena: GraphemeArena) -> Self {
        Self {
            inner: Buffer::with_arena(width, height, arena),
            rows: DirtyRows::new(height),
            cols: DirtyColumns::new(height),
        }
    }

    /// Returns `true` if any cell has been modified since the last reset.
    #[inline]
    pub fn any(&self) -> bool {
        self.rows.any()
    }

    /// Returns `true` if the given row has been modified.
    #[inline]
    pub fn is_marked(&self, row: usize) -> bool {
        self.rows.is_marked(row)
    }

    /// Returns the dirty column range for a row (min..=max inclusive).
    /// Returns `None` if the row is clean.
    #[inline]
    pub fn get_marks(&self, row: usize) -> Option<RangeInclusive<usize>> {
        let range = self.cols[row];
        if range.is_clean() {
            None
        } else {
            Some(range.into())
        }
    }

    /// Iterate over indices of dirty rows.
    pub fn marked_rows(&self) -> impl Iterator<Item = usize> + '_ {
        let height = self.inner.height;
        self.rows.iter().take_while(move |&r| r < height)
    }

    // ── Marking ─────────────────────────────────────────────────

    /// Mark a row as dirty across its full width.
    #[inline]
    pub fn mark(&mut self, row: usize) {
        self.rows.mark(row);
        self.cols[row].set_width(self.inner.width);
    }


    /// Mark a row as dirty across its full width.
    #[inline]
    pub fn mark_rows(&mut self, row: usize, n: usize) {
        for r in row..(row + n) {
            self.rows.mark(r);
            self.cols[r].set_width(self.inner.width);
        }
    }


    /// Mark all rows as dirty.
    pub fn mark_all(&mut self) {
        self.rows.mark_all();
        let width = self.inner.width;
        for range in &mut self.cols {
            range.set_width(width);
        }
    }

    /// Clear all dirty tracking state.
    pub fn reset(&mut self) {
        self.rows.clear();
        self.cols.fill(Mark::CLEAN);
    }

    /// Fill a region with a cell value.
    pub fn fill_region(&mut self, bounds: Bounds, cell: Cell) {
        let bounds = self.clip(bounds);
        for row in bounds.min.row..bounds.max.row {
            self.rows.mark(row);
            self.cols[row].set_range(bounds.min.col..bounds.max.col);
        }
        self.inner.fill_area(bounds, cell);
    }

    /// Clear the buffer contents and mark everything dirty.
    pub fn clear(&mut self) {
        self.mark_all();
        self.inner.clear();
    }

    /// Resize the buffer. Marks everything dirty and resets tracking state.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height);
        self.rows = DirtyRows::new(height);
        self.cols = DirtyColumns::new(height);
        self.mark_all();
    }

    // ── Tracked line/cell operations ────────────────────────────

    /// Insert `n` lines at row `y`, shifting remaining lines down.
    /// Marks all rows from `y` downward as dirty.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        for row in y..self.inner.height {
            self.mark(row);
        }
        self.inner.insert_line(y, n, cell);
    }

    /// Delete `n` lines at row `y`, shifting remaining lines up.
    /// Marks all rows from `y` downward as dirty.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        for row in y..self.inner.height {
            self.mark(row);
        }
        self.inner.delete_line(y, n, cell);
    }

    /// Insert `n` cells at `(x, y)`, shifting cells right.
    /// Marks the entire row as dirty.
    pub fn insert_cell(&mut self, x: usize, y: usize, n: usize, cell: Cell) {
        self.mark(y);
        self.inner.insert_cell(x, y, n, cell);
    }

    /// Delete `n` cells at `(x, y)`, shifting cells left.
    /// Marks the entire row as dirty.
    pub fn delete_cell(&mut self, x: usize, y: usize, n: usize, cell: Cell) {
        self.mark(y);
        self.inner.delete_cell(x, y, n, cell);
    }

    pub fn insert_cell_area(&mut self, x: usize, y: usize, n: usize, cell: Cell, bounds: Bounds) {
        self.inner.insert_cell_area(x, y, n, cell, bounds);
        self.mark(y);
    }

    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Bounds) {
        self.inner.insert_line_area(y, n, cell, bounds);
        for row in y..(y + n) {
            self.mark(row);
        }
    }

    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Bounds) {
        self.inner.delete_line_area(y, n, cell, bounds);
        for row in y..(y + n) {
            self.mark(row);
        }
    }
}

// ── Tracked IndexMut implementations ────────────────────────────
impl ops::IndexMut<Position> for TrackingBuffer {
    #[inline]
    fn index_mut(&mut self, pos: Position) -> &mut Cell {
        self.rows.mark(pos.row);
        self.cols[pos.row].set(pos.col);
        &mut self.inner[pos]
    }
}

impl ops::IndexMut<PositionLike> for TrackingBuffer {
    #[inline]
    fn index_mut(&mut self, pos: PositionLike) -> &mut Cell {
        self.rows.mark(pos.0);
        self.cols[pos.0].set(pos.1);
        &mut self.inner[pos]
    }
}

impl ops::IndexMut<Row> for TrackingBuffer {
    #[inline]
    fn index_mut(&mut self, row: Row) -> &mut [Cell] {
        self.mark(row.0);
        &mut self.inner[row]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ansi::Style;

    #[test]
    fn new_buffer_is_clean() {
        let buf = TrackingBuffer::new(80, 24);
        assert!(!buf.any());
        assert!(!buf.is_marked(0));
        assert!(buf.get_marks(0).is_none());
        assert_eq!(buf.marked_rows().count(), 0);
    }

    #[test]
    fn set_marks_dirty() {
        let mut buf = TrackingBuffer::new(80, 24);
        let cell = Cell::from_char('A', Style::EMPTY);
        buf[Position::new(5, 10)] = cell;

        assert!(buf.any());
        assert!(buf.is_marked(5));
        assert!(!buf.is_marked(0));
        assert_eq!(buf.get_marks(5), Some(10..=10));
    }

    #[test]
    fn index_mut_position_marks_dirty() {
        let mut buf = TrackingBuffer::new(80, 24);
        buf[Position::new(3, 7)] = Cell::from_char('B', Style::EMPTY);

        assert!(buf.is_marked(3));
        assert_eq!(buf.get_marks(3), Some(7..=7));
    }

    #[test]
    fn index_mut_row_marks_full_row() {
        let mut buf = TrackingBuffer::new(80, 24);
        let _ = &mut buf[Row(2)]; // take mutable ref to row

        assert!(buf.is_marked(2));
        assert_eq!(buf.get_marks(2), Some(0..=79));
    }

    #[test]
    fn multiple_writes_expand_range() {
        let mut buf = TrackingBuffer::new(80, 24);
        let cell = Cell::from_char('X', Style::EMPTY);
        buf[Position::new(1, 5)] = cell;
        buf[Position::new(1, 20)] = cell;
        buf[Position::new(1, 12)] = cell;

        assert_eq!(buf.get_marks(1), Some(5..=20));
    }

    #[test]
    fn reset_clears_dirty_state() {
        let mut buf = TrackingBuffer::new(80, 24);
        let cell = Cell::from_char('Z', Style::EMPTY);
        buf[Position::new(0, 0)] = cell;
        buf[Position::new(23, 79)] = cell;

        assert!(buf.any());
        buf.reset();

        assert!(!buf.any());
        assert!(!buf.is_marked(0));
        assert!(!buf.is_marked(23));
        assert_eq!(buf.marked_rows().count(), 0);
    }

    #[test]
    fn dirty_rows_iterates_only_dirty() {
        let mut buf = TrackingBuffer::new(80, 24);
        dbg!(&buf.inner);
        let cell = Cell::from_char('A', Style::EMPTY);
        buf[Position::new(2, 0)] = cell;
        buf[Position::new(10, 0)] = cell;
        buf[Position::new(23, 0)] = cell;

        let dirty: Vec<usize> = buf.marked_rows().collect();
        assert_eq!(dirty, vec![2, 10, 23]);
    }

    #[test]
    fn fill_region_marks_affected_rows() {
        let mut buf = TrackingBuffer::new(80, 24);
        let bounds = Bounds::new(Position::new(5, 10), Position::new(8, 30));
        buf.fill_region(bounds, Cell::default());

        assert!(!buf.is_marked(4));
        assert!(buf.is_marked(5));
        assert!(buf.is_marked(6));
        assert!(buf.is_marked(7));
        assert!(!buf.is_marked(8)); // half-open: 8 is excluded
        assert_eq!(buf.get_marks(5), Some(10..=29));
    }

    #[test]
    fn insert_line_marks_from_y_downward() {
        let mut buf = TrackingBuffer::new(80, 5);
        buf.insert_line(2, 1, Cell::EMPTY);

        assert!(!buf.is_marked(0));
        assert!(!buf.is_marked(1));
        assert!(buf.is_marked(2));
        assert!(buf.is_marked(3));
        assert!(buf.is_marked(4));
    }

    #[test]
    fn read_via_deref_does_not_mark_dirty() {
        let buf = TrackingBuffer::new(80, 24);
        let _ = buf[Position::new(0, 0)]; // read via Deref -> Buffer -> Index
        let _ = &buf[Row(5)];             // read row slice

        assert!(!buf.any());
    }

    #[test]
    fn bitset_handles_more_than_64_rows() {
        let mut buf = TrackingBuffer::new(80, 200);
        let cell = Cell::from_char('A', Style::EMPTY);
        buf[Position::new(0, 0)] = cell;
        buf[Position::new(63, 0)] = cell;
        buf[Position::new(64, 0)] = cell;
        buf[Position::new(127, 0)] = cell;
        buf[Position::new(199, 0)] = cell;

        let dirty: Vec<usize> = buf.marked_rows().collect();
        assert_eq!(dirty, vec![0, 63, 64, 127, 199]);
    }
}
