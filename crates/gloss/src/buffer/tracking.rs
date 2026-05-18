//! Dirty-row tracking layered on top of [`Buffer`].
//!
//! [`TrackingBuffer`] wraps a [`Buffer`] and records which rows have been touched
//! since the last [`clear_dirty`](TrackingBuffer::clear_dirty). [`DirtyDiff`] uses
//! that bitmap to skip untouched rows in O(1), instead of falling back to a
//! per-row slice comparison.

use crate::{Arena, Buffer, BufferDiff, BufferIndex, ByDirty, Cell, Change};
use derive_more::{AsRef, Deref};
use geometry::{Bound, Point, Rect};
use std::iter::FusedIterator;
use std::ops::Range;

/// A [`Buffer`] with per-row dirty tracking.
///
/// Read-only access mirrors [`Buffer`] via [`Deref`]. Mutations go through
/// dedicated methods that mark the touched rows dirty, so a subsequent
/// [`diff_dirty`](Self::diff_dirty) can fast-skip rows that were not touched
/// since the last [`clear_dirty`](Self::clear_dirty).
///
/// # Escape hatches
///
/// - [`buffer_mut`](Self::buffer_mut) returns a `&mut Buffer` and
///   conservatively marks every row dirty.
/// - [`row_mut`](Self::row_mut) / [`cell_mut`](Self::cell_mut) give granular
///   mutable access and mark only the affected row.
///
/// # Lifecycle
///
/// 1. New `Tracking` starts with every row dirty so an initial
///    [`diff_dirty`](Self::diff_dirty) sees the full content.
/// 2. Mutations mark rows dirty.
/// 3. Call [`diff_dirty`](Self::diff_dirty) and apply the changes.
/// 4. Call [`clear_dirty`](Self::clear_dirty) before the next frame.
#[derive(Clone, Debug, Deref, AsRef)]
pub struct TrackingBuffer {
    #[deref]
    #[as_ref]
    inner: Buffer,
    /// Per-row dirty flags. Invariant: `dirty.len() == inner.height`.
    dirty: Vec<bool>,
}

impl TrackingBuffer {
    /// Empty tracking buffer.
    pub const EMPTY: Self = Self {
        inner: Buffer::EMPTY,
        dirty: Vec::new(),
    };

    /// Create a new tracking buffer of the given size with every row dirty.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: Buffer::new(width, height),
            dirty: vec![true; height],
        }
    }

    /// Wrap an existing buffer with every row marked dirty.
    pub fn from_buffer(inner: Buffer) -> Self {
        let dirty = vec![true; inner.height];
        Self { inner, dirty }
    }

    // --------------------------------------------------------------
    // Dirty bitmap
    // --------------------------------------------------------------

    /// Per-row dirty flags. `result[y]` is `true` if row `y` has been
    /// touched since the last [`clear_dirty`](Self::clear_dirty).
    #[inline]
    pub fn dirty_rows(&self) -> &[bool] {
        &self.dirty
    }

    /// Whether row `y` is marked dirty. Out-of-bounds rows return `false`.
    #[inline]
    pub fn is_row_dirty(&self, y: usize) -> bool {
        self.dirty.get(y).copied().unwrap_or(false)
    }

    /// Number of rows currently marked dirty.
    pub fn dirty_row_count(&self) -> usize {
        self.dirty.iter().filter(|&&d| d).count()
    }

    /// `true` if no rows are marked dirty.
    pub fn is_clean(&self) -> bool {
        self.dirty.iter().all(|d| !d)
    }

    /// Iterator over the indices of rows currently marked dirty.
    pub fn dirty_row_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.dirty
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| d.then_some(i))
    }

    /// Mark a single row dirty. Out-of-bounds rows are ignored.
    #[inline]
    pub fn mark_row_dirty(&mut self, y: usize) {
        if let Some(slot) = self.dirty.get_mut(y) {
            *slot = true;
        }
    }

    /// Mark a half-open range of rows dirty. Bounds are clamped.
    pub fn mark_rows_dirty(&mut self, rows: Range<usize>) {
        let end = rows.end.min(self.dirty.len());
        let start = rows.start.min(end);
        for slot in &mut self.dirty[start..end] {
            *slot = true;
        }
    }

    /// Mark every row dirty.
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty.fill(true);
    }

    /// Reset all dirty flags. Call this once the previous frame has been
    /// applied so the next [`diff_dirty`](Self::diff_dirty) can skip
    /// untouched rows.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty.fill(false);
    }

    // --------------------------------------------------------------
    // Escape hatches
    // --------------------------------------------------------------

    /// Borrow the inner buffer.
    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.inner
    }

    /// Mutable access to the inner buffer. Marks every row dirty, since the
    /// wrapper cannot see what is mutated through the returned reference.
    /// Prefer [`row_mut`](Self::row_mut) / [`cell_mut`](Self::cell_mut) for
    /// precise tracking.
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.mark_dirty();
        &mut self.inner
    }

    /// Consume `self` and return the inner buffer, dropping dirty state.
    #[inline]
    pub fn into_buffer(self) -> Buffer {
        self.inner
    }

    // --------------------------------------------------------------
    // Granular mutators
    // --------------------------------------------------------------

    /// Mutable access to row `y`, marking it dirty.
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [Cell]> {
        if y >= self.inner.height {
            return None;
        }
        self.mark_row_dirty(y);
        let width = self.inner.width;
        Some(&mut self.inner[y * width..(y + 1) * width])
    }

    /// Mutable access to a single cell, marking its row dirty.
    pub fn cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        self.mark_row_dirty(point.y as usize);
        self.inner.get_mut(point)
    }

    // --------------------------------------------------------------
    // Wrapped mutation methods
    // --------------------------------------------------------------

    /// Clear the buffer to default cells.
    pub fn clear(&mut self) {
        self.mark_dirty();
        self.inner.clear();
    }

    /// Resize the buffer. Marks every row dirty since width changes shift
    /// existing cells.
    pub fn resize(&mut self, width: usize, height: usize) {
        if self.inner.width == width && self.inner.height == height {
            return;
        }
        self.inner.resize(width, height);
        self.dirty.resize(height, true);
        self.dirty.fill(true);
    }

    /// Write a string starting at `start`, marking the affected row dirty.
    pub fn set_line(
        &mut self,
        start: Point,
        string: impl AsRef<str>,
        arena: &mut Arena,
    ) -> Option<usize> {
        self.mark_row_dirty(start.y as usize);
        self.inner.set_line(start, string, arena)
    }

    /// Write a string at `index`.
    ///
    /// The generic index does not give us a cheap way to compute the
    /// affected rows, so this conservatively marks every row dirty. Use
    /// [`set_line`](Self::set_line) for single-row writes or
    /// [`row_mut`](Self::row_mut) to keep dirty tracking precise.
    pub fn set_string<I>(
        &mut self,
        index: I,
        string: impl AsRef<str>,
        arena: &mut Arena,
    ) -> Option<usize>
    where
        I: BufferIndex<Output = [Cell]>,
    {
        self.mark_dirty();
        self.inner.set_string(index, string, arena)
    }

    /// Insert `n` lines at row `y`, shifting following rows down.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.mark_rows_dirty(y..self.inner.height);
        self.inner.insert_line(y, n, cell);
    }

    /// Delete `n` lines at row `y`, shifting following rows up.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.mark_rows_dirty(y..self.inner.height);
        self.inner.delete_line(y, n, cell);
    }

    /// Insert `n` cells at `(row, col)`, shifting cells right on that row.
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.mark_row_dirty(row);
        self.inner.insert_cell(row, col, n, cell);
    }

    /// Delete `n` cells at `(row, col)`, shifting cells left on that row.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.mark_row_dirty(row);
        self.inner.delete_cell(row, col, n, cell);
    }

    /// Bounded variant of [`insert_line`](Self::insert_line).
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.mark_rows_dirty(bounds.min.y as usize..bounds.max.y as usize);
        self.inner.insert_line_area(y, n, cell, bounds);
    }

    /// Bounded variant of [`delete_line`](Self::delete_line).
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.mark_rows_dirty(bounds.min.y as usize..bounds.max.y as usize);
        self.inner.delete_line_area(y, n, cell, bounds);
    }

    /// Bounded variant of [`insert_cell`](Self::insert_cell).
    pub fn insert_cell_area(
        &mut self,
        row: usize,
        col: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) {
        self.mark_row_dirty(row);
        self.inner.insert_cell_area(row, col, n, cell, bounds);
    }

    /// Bounded variant of [`delete_cell`](Self::delete_cell).
    pub fn delete_cell_area(
        &mut self,
        row: usize,
        col: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) {
        self.mark_row_dirty(row);
        self.inner.delete_cell_area(row, col, n, cell, bounds);
    }

    /// Append a row.
    pub fn push_row(&mut self, row: impl IntoIterator<Item = Cell>) {
        self.inner.push_row(row);
        self.dirty.push(true);
    }

    /// Pop the bottom row.
    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        let row = self.inner.pop_row()?;
        self.dirty.pop();
        Some(row)
    }

    /// Remove row `idx`. Following rows shift up and are marked dirty.
    pub fn remove_row(&mut self, idx: usize) -> Option<Vec<Cell>> {
        let row = self.inner.remove_row(idx)?;
        if idx < self.dirty.len() {
            self.dirty.remove(idx);
        }
        if let Some(rest) = self.dirty.get_mut(idx..) {
            for slot in rest {
                *slot = true;
            }
        }
        Some(row)
    }

    /// Insert a row at `idx`. Following rows shift down and are marked dirty.
    pub fn insert_row(&mut self, idx: usize, row: impl IntoIterator<Item = Cell>) {
        self.inner.insert_row(idx, row);
        let dirty_idx = idx.min(self.dirty.len());
        self.dirty.insert(dirty_idx, true);
        if let Some(rest) = self.dirty.get_mut(dirty_idx + 1..) {
            for slot in rest {
                *slot = true;
            }
        }
    }

    // --------------------------------------------------------------
    // Diff integration
    // --------------------------------------------------------------

    /// Diff `prev` against `self`, skipping rows that are not marked dirty.
    ///
    /// Yields the same [`Change`]s as [`Buffer::diff_cells`] would, provided
    /// every cell mutation since the last [`clear_dirty`](Self::clear_dirty)
    /// marked its row dirty. The win is an O(1) per-row early-exit on clean
    /// rows instead of a slice equality.
    pub fn diff_dirty<'next, 'prev>(&'next self, prev: &'prev Buffer) -> BufferDiff<'prev, 'next, ByDirty> {
        BufferDiff::dirty(prev, self)
    }
}

impl Default for TrackingBuffer {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl PartialEq for TrackingBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<Buffer> for TrackingBuffer {
    fn from(buffer: Buffer) -> Self {
        Self::from_buffer(buffer)
    }
}

impl From<TrackingBuffer> for Buffer {
    fn from(tracking: TrackingBuffer) -> Self {
        tracking.inner
    }
}

impl Bound for TrackingBuffer {
    type Point = Point;

    fn min_x(&self) -> u16 {
        self.inner.min_x()
    }
    fn min_y(&self) -> u16 {
        self.inner.min_y()
    }
    fn max_x(&self) -> u16 {
        self.inner.max_x()
    }
    fn max_y(&self) -> u16 {
        self.inner.max_y()
    }
    fn min(&self) -> Self::Point {
        self.inner.min()
    }
    fn max(&self) -> Self::Point {
        self.inner.max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arena;

    #[test]
    fn new_starts_with_all_rows_dirty() {
        let t = TrackingBuffer::new(5, 3);
        assert_eq!(t.dirty_row_count(), 3);
        assert!(!t.is_clean());
        for y in 0..3 {
            assert!(t.is_row_dirty(y));
        }
    }

    #[test]
    fn clear_dirty_marks_all_clean() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        assert!(t.is_clean());
        assert_eq!(t.dirty_row_count(), 0);
    }

    #[test]
    fn out_of_bounds_marks_are_ignored() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        t.mark_row_dirty(99);
        assert!(t.is_clean());
        t.mark_rows_dirty(10..20);
        assert!(t.is_clean());
    }

    #[test]
    fn set_line_marks_only_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        let mut arena = Arena::new();
        t.set_line(Point { x: 0, y: 1 }, "abc", &mut arena);
        assert!(!t.is_row_dirty(0));
        assert!(t.is_row_dirty(1));
        assert!(!t.is_row_dirty(2));
    }

    #[test]
    fn set_string_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        let mut arena = Arena::new();
        t.set_string(
            Point { x: 0, y: 0 }..Point { x: 5, y: 0 },
            "abc",
            &mut arena,
        );
        assert_eq!(t.dirty_row_count(), 3);
    }

    #[test]
    fn cell_mut_marks_just_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        *t.cell_mut(Point { x: 2, y: 2 }).unwrap() = Cell::inline('z');
        assert!(!t.is_row_dirty(0));
        assert!(!t.is_row_dirty(1));
        assert!(t.is_row_dirty(2));
    }

    #[test]
    fn row_mut_marks_just_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        t.row_mut(1).unwrap()[0] = Cell::inline('x');
        assert!(!t.is_row_dirty(0));
        assert!(t.is_row_dirty(1));
        assert!(!t.is_row_dirty(2));
    }

    #[test]
    fn row_mut_out_of_bounds_is_none() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        assert!(t.row_mut(99).is_none());
        assert!(t.is_clean());
    }

    #[test]
    fn buffer_mut_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        let _ = t.buffer_mut();
        assert_eq!(t.dirty_row_count(), 3);
    }

    #[test]
    fn resize_updates_bitmap_and_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        t.resize(7, 5);
        assert_eq!(t.dirty_rows().len(), 5);
        assert!(t.dirty_rows().iter().all(|&d| d));
    }

    #[test]
    fn resize_noop_when_dimensions_match() {
        let mut t = TrackingBuffer::new(5, 3);
        t.clear_dirty();
        t.resize(5, 3);
        assert!(t.is_clean());
    }

    #[test]
    fn push_pop_row_updates_bitmap() {
        let mut t = TrackingBuffer::new(2, 1);
        t.clear_dirty();
        t.push_row([Cell::inline('a'), Cell::inline('b')]);
        assert_eq!(t.dirty_rows().len(), 2);
        assert!(t.is_row_dirty(1));
        let _ = t.pop_row();
        assert_eq!(t.dirty_rows().len(), 1);
    }

    #[test]
    fn insert_line_marks_y_and_below() {
        let mut t = TrackingBuffer::new(3, 4);
        t.clear_dirty();
        t.insert_line(1, 1, Cell::EMPTY);
        assert!(!t.is_row_dirty(0));
        assert!(t.is_row_dirty(1));
        assert!(t.is_row_dirty(2));
        assert!(t.is_row_dirty(3));
    }

    #[test]
    fn insert_cell_marks_only_its_row() {
        let mut t = TrackingBuffer::new(4, 3);
        t.clear_dirty();
        t.insert_cell(1, 0, 1, Cell::EMPTY);
        assert!(!t.is_row_dirty(0));
        assert!(t.is_row_dirty(1));
        assert!(!t.is_row_dirty(2));
    }

    #[test]
    fn dirty_row_indices_enumerates_dirty() {
        let mut t = TrackingBuffer::new(2, 4);
        t.clear_dirty();
        t.mark_row_dirty(0);
        t.mark_row_dirty(2);
        let dirty: Vec<_> = t.dirty_row_indices().collect();
        assert_eq!(dirty, vec![0, 2]);
    }

    #[test]
    fn diff_dirty_skips_clean_rows_and_unchanged_dirty_rows() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["aaa", "bbb", "ccc"], &mut arena);
        let mut next =
            TrackingBuffer::from_buffer(Buffer::from_lines(["aaa", "bbb", "ccc"], &mut arena));
        next.clear_dirty();

        // Re-render row 1 with the same content. Dirty bit set, but slice-eq
        // fast path should still produce no changes.
        next.set_line(Point { x: 0, y: 1 }, "bbb", &mut arena);
        assert!(next.is_row_dirty(1));
        let changes: Vec<_> = next.diff_dirty(&prev).collect();
        assert!(changes.is_empty());

        // Mutate row 2 with different content.
        next.set_line(Point { x: 0, y: 2 }, "cCc", &mut arena);
        let changes: Vec<_> = next.diff_dirty(&prev).collect();
        assert_eq!(changes.len(), 1);
        assert_eq!((changes[0].x, changes[0].y), (1, 2));
    }

    #[test]
    fn diff_dirty_matches_buffer_diff_when_all_dirty() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["hello", "world"], &mut arena);
        let next_buf = Buffer::from_lines(["hallo", "wXrld"], &mut arena);
        let next = TrackingBuffer::from_buffer(next_buf.clone());

        let via_dirty: Vec<_> = next.diff_dirty(&prev).map(|c| (c.x, c.y)).collect();
        let via_buffer: Vec<_> = Buffer::diff_cells(&prev, &next_buf)
            .map(|c| (c.x, c.y))
            .collect();

        assert_eq!(via_dirty, via_buffer);
    }

    #[test]
    fn diff_dirty_size_hint_is_upper_bound() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["aaa"], &mut arena);
        let next = TrackingBuffer::from_buffer(Buffer::from_lines(["bbb"], &mut arena));

        let iter = next.diff_dirty(&prev);
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 0);
        assert_eq!(upper, Some(3));
        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn diff_dirty_exhausted_iterator_stays_none() {
        let prev = Buffer::new(2, 1);
        let next = TrackingBuffer::new(2, 1);
        let mut iter = next.diff_dirty(&prev);
        // Single row, both empty: slice eq skips it immediately.
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    #[should_panic(expected = "buffers must have the same width")]
    fn diff_dirty_mismatched_widths_panic() {
        let prev = Buffer::new(5, 1);
        let next = TrackingBuffer::new(10, 1);
        let _ = next.diff_dirty(&prev);
    }

    #[test]
    fn into_buffer_round_trips() {
        let mut arena = Arena::new();
        let buf = Buffer::from_lines(["abc"], &mut arena);
        let tracking: TrackingBuffer = buf.clone().into();
        assert_eq!(tracking.into_buffer(), buf);
    }

    #[test]
    fn deref_exposes_buffer_api() {
        // Read-only API on Buffer should be reachable via auto-deref.
        let mut arena = Arena::new();
        let t = TrackingBuffer::from_buffer(Buffer::from_lines(["abc"], &mut arena));
        assert_eq!(t.width(), 3);
        assert_eq!(t.height(), 1);
    }
}
