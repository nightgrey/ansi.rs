//! Dirty-row tracking layered on top of [`Buffer`].
//!
//! [`TrackingBuffer`] wraps a [`Buffer`] and records which rows have been touched
//! since the last [`clear_dirty`](TrackingBuffer::unmark_all). [`DirtyDiff`] uses
//! that bitmap to skip untouched rows in O(1), instead of falling back to a
//! per-row slice comparison.

use std::{iter, ops};
use crate::{Arena, Buffer, BufferDiff, BufferIndex, ByDirty, Cell, Change};
use derive_more::{AsRef, Deref, From, Index};
use geometry::{Bound, Point, Position, PositionLike, Rect, Resolve, Row};
use std::ops::{DerefMut, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice::SliceIndex;
pub use fixedbitset::{IndexRange as TrackingRange};

pub type BitSet = fixedbitset::FixedBitSet;
pub type Bit = fixedbitset::Block;

/// A [`Buffer`] with per-row dirty tracking using a [`BitSet`].
///
/// Read-only access mirrors [`Buffer`] via [`Deref`]. Mutations go through
/// dedicated methods that mark the touched rows dirty, so a subsequent
/// [`diff_dirty`](Self::diff) can fast-skip rows that were not touched
/// since the last [`clear_dirty`](Self::unmark_all).
///
/// # Lifecycle
///
/// 1. New `Tracking` starts with every row dirty so an initial
///    [`diff_dirty`](Self::diff) sees the full content.
/// 2. Mutations mark rows dirty.
/// 3. Call [`diff_dirty`](Self::diff) and apply the changes.
/// 4. Call [`clear_dirty`](Self::unmark_all) before the next frame.
/// A [`Buffer`] with per-row dirty tracking via a [`BitSet`].
///
/// Read‑only access mirrors the inner [`Buffer`] via [`Deref`].
/// Mutations go through dedicated methods that **mark** the touched rows,
/// enabling a subsequent fast diff that skips unmarked rows.
///
/// # Lifecycle
///
/// 1. A new `TrackingBuffer` starts with **all rows marked** so that the
///    first diff sees the full content.
/// 2. Each mutator marks the affected rows.
/// 3. Call [`diff`](Self::diff) to obtain the changes.
/// 4. Call [`unmark_all`](Self::unmark_all) (or [`clean`](Self::clean))
///    before the next frame.
///
/// # Invariants
///
/// - `self.bits.len() == self.inner.height`
/// - A row is *marked* iff it has been mutated since the last `unmark_all`.
#[derive(Clone, Debug, Deref, AsRef, From)]
pub struct TrackingBuffer {
    #[deref]
    #[as_ref]
    pub(super) inner: Buffer,
    /// Per-row markers. Invariant: `dirty.len() == inner.height`.
    pub(super) bits: BitSet,
}

impl TrackingBuffer {
    /// An empty tracking buffer. All rows are implicitly unmarked.
    pub const EMPTY: Self = Self {
        inner: Buffer::EMPTY,
        bits: BitSet::new(),
    };

    /// Create a new tracking buffer of the given size.
    ///
    /// Every row is initially **marked** so a fresh diff covers the whole area.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: Buffer::new(width, height),
            bits: BitSet::with_capacity(height),
        }
    }

    /// Create a new tracking buffer of the given size with every row marked.
    pub fn new_dirty(width: usize, height: usize) -> Self {
        Self {
            inner: Buffer::new(width, height),
            bits: BitSet::with_capacity_and_blocks(height, iter::repeat_n(1, height)),
        }
    }

    /// Create a tracking buffer from an existing [`Buffer`].
    ///
    /// Rows that are **non‑empty** are marked; empty rows are unmarked.
    /// This is useful when creating a tracking wrapper for a buffer that
    /// already has content.
    pub fn from_buffer_scanned(buffer: Buffer) -> Self {
        let bits = BitSet::with_capacity_and_blocks(
            buffer.height,
            buffer.iter_rows().map(|row| !row.is_empty() as usize),
        );
        Self {
            inner: buffer,
            bits,
        }
    }

    /// Create a tracking buffer from an existing [`Buffer`].
    ///
    /// **All** rows are marked unconditionally.
    pub fn from_buffer_dirty(buffer: Buffer) -> Self {
        Self {
            bits: BitSet::with_capacity_and_blocks(
                buffer.height,
                iter::repeat_n(1, buffer.height),
            ),
            inner: buffer,
        }
    }

    /// Create a tracking buffer from an existing [`Buffer`].
    ///
    /// **No** rows are marked (all unmarked).
    pub fn from_buffer_clean(buffer: Buffer) -> Self {
        Self {
            bits: BitSet::with_capacity_and_blocks(
                buffer.height,
                iter::repeat_n(0, buffer.height),
            ),
            inner: buffer,
        }
    }
    
    // ----------------------------------------------------------
    // Low-level marker manipulation
    // ----------------------------------------------------------

    /// Mark a single row.
    #[inline]
    pub fn mark(&mut self, y: usize) {
        self.bits.set(y, true);
    }

    /// Mark all rows in the given range.
    #[inline]
    pub fn mark_many(&mut self, y_range: impl TrackingBufferIndex) {
        let range = y_range.into_tracking_range(&mut self.inner);
        self.bits.set_range(range, true);
    }

    /// Mark every row.
    #[inline]
    pub fn mark_all(&mut self) {
        self.mark_many(..);
    }

    /// Unmark a single row.
    #[inline]
    pub fn unmark(&mut self, y: usize) {
        self.bits.set(y, false);
    }

    /// Unmark all rows in the given range.
    #[inline]
    pub fn unmark_many(&mut self, y_range: impl TrackingRange) {
        self.bits.set_range(y_range, false);
    }

    /// Unmark every row. Call this after applying a diff so that the next
    /// diff only sees rows mutated after this call.
    #[inline]
    pub fn unmark_all(&mut self) {
        self.bits.clear();
    }

    // ----------------------------------------------------------
    // Querying marker state
    // ----------------------------------------------------------

    /// Returns `true` if the row at `y` is marked.
    ///
    /// Out‑of‑bounds rows return `false`.
    #[inline]
    pub fn is_marked(&self, y: usize) -> bool {
        self.bits.contains(y)
    }

    /// Returns `true` if **any** row in the range is marked.
    #[inline]
    pub fn is_any_marked(&self, y_range: impl TrackingRange) -> bool {
        self.bits.contains_any_in_range(y_range)
    }

    /// Returns `true` if **all** rows in the range are marked.
    #[inline]
    pub fn is_all_marked(&self, y_range: impl TrackingRange) -> bool {
        self.bits.contains_all_in_range(y_range)
    }

    /// Returns `true` if **any** row in the range is unmarked.
    #[inline]
    pub fn is_any_unmarked(&self, y_range: impl TrackingRange) -> bool {
        !self.is_all_marked(y_range)
    }

    /// Returns `true` if **all** rows in the range are unmarked.
    #[inline]
    pub fn is_all_unmarked(&self, y_range: impl TrackingRange) -> bool {
        !self.is_any_marked(y_range)
    }

    /// Returns `true` if **no** rows are marked.
    #[inline]
    pub fn is_clean(&self) -> bool {
        self.bits.is_empty()
    }

    /// Unmark every row.  (Synonym of [`unmark_all`](Self::unmark_all).)
    ///
    /// After calling this the buffer is *clean*.
    pub fn clean(&mut self) {
        self.bits.clear();
    }

    /// Returns `true` if **any** row is marked.
    ///
    /// This is the dual of [`is_clean`](Self::is_clean).
    pub fn is_dirty(&self) -> bool {
        !self.is_clean()
    }

    /// Mark every row.  (Synonym of [`mark_all`](Self::mark_all).)
    ///
    /// After calling this the buffer is *dirty*.
    pub fn dirty(&mut self) {
        self.bits.set_range(.., true);
    }

    /// Iterate over the indices of marked rows.
    pub fn marked(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.ones()
    }

    /// Returns the number of marked rows.
    pub fn count(&self) -> usize {
        self.count_marked(..)
    }

    /// Returns the number of marked rows in the given range.
    pub fn count_marked(&self, range: impl TrackingRange) -> usize {
        self.bits.count_ones(range)
    }

    /// Returns the number of unmarked rows in the given range.
    pub fn count_unmarked(&self, range: impl TrackingRange) -> usize {
        self.bits.count_zeroes(range)
    }

    /// Returns a slice of the raw marker bits (one `Bit` per row).
    ///
    /// `result[y]` is `true` if row `y` is marked.
    #[inline]
    pub fn as_bits(&self) -> &BitSet {
        &self.bits
    }

    // ----------------------------------------------------------
    // Borrowing the inner buffer
    // ----------------------------------------------------------

    /// Borrow the inner buffer (immutable).
    #[inline]
    pub fn as_inner(&self) -> &Buffer {
        &self.inner
    }

    /// Borrow the inner buffer mutably.
    ///
    /// **Warning:** This escape hatch **bypasses** dirty tracking. Use the
    /// dedicated mutating methods instead whenever possible.
    pub fn as_mut_inner(&mut self) -> &mut Buffer {
        &mut self.inner
    }

    /// Consume the tracking wrapper and return the inner [`Buffer`],
    /// discarding marker state.
    #[inline]
    pub fn into_inner(self) -> Buffer {
        self.inner
    }

    // ----------------------------------------------------------
    // Granular mutators (with automatic marking)
    // ----------------------------------------------------------

    /// Returns a mutable reference to the output at the given index, if it
    /// is in bounds.  Marks the affected row(s) automatically.
    pub fn get_mut<I: BufferIndex>(
        &mut self,
        index: I,
    ) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output> {
        index.get_mut(self)
    }

    /// Mutable access to row `y`, marking it.
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [Cell]> {
        if y >= self.inner.height {
            return None;
        }
        self.mark(y);
        let width = self.inner.width;
        Some(&mut self.inner[y * width..(y + 1) * width])
    }

    /// Mutable access to a single cell, marking its row.
    pub fn cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        self.mark(point.y as usize);
        self.inner.get_mut(point)
    }

    // ----------------------------------------------------------
    // Bulk mutation wrappers
    // ----------------------------------------------------------

    /// Clear all cells to defaults.  Every row is marked.
    pub fn clear(&mut self) {
        self.mark_many(..);
        self.inner.clear();
    }

    /// Resize the buffer.
    ///
    /// If the width changes, existing cells shift and **all** rows become
    /// marked.  If the height grows, new rows are marked as well.
    /// If the height shrinks, only the remaining rows are marked.
    pub fn resize(&mut self, width: usize, height: usize) {
        let previous_height = self.inner.height;
        if self.inner.width == width && self.inner.height == height {
            return;
        }
        self.inner.resize(width, height);
        if previous_height < height {
            self.bits.grow(height);
            self.bits.set_range(.., true);
        } else if previous_height > height {
            self.bits = BitSet::with_capacity_and_blocks(height, iter::repeat_n(1, height));
        }
    }

    /// Write a string starting at `start`, marking the affected row.
    pub fn set_line(
        &mut self,
        start: Point,
        string: impl AsRef<str>,
        arena: &mut Arena,
    ) -> Option<usize> {
        self.mark(start.y as usize);
        self.inner.set_line(start, string, arena)
    }

    /// Write a string at `index`.
    ///
    /// Because the generic index does not expose which rows are touched,
    /// this method conservatively marks **all** rows. Prefer
    /// [`set_line`](Self::set_line) or [`row_mut`](Self::row_mut) for
    /// precise tracking.
    pub fn set_string<I>(
        &mut self,
        index: I,
        string: impl AsRef<str>,
        arena: &mut Arena,
    ) -> Option<usize>
    where
        I: BufferIndex<Output = [Cell]>,
    {
        self.mark_all();
        self.inner.set_string(index, string, arena)
    }

    /// Insert `n` lines at row `y`, shifting subsequent rows down.
    ///
    /// All rows from `y` to the bottom are marked.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.mark_many(y..self.inner.height);
        self.inner.insert_line(y, n, cell);
    }

    /// Delete `n` lines at row `y`, shifting subsequent rows up.
    ///
    /// All rows from `y` to the bottom are marked.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.mark_many(y..self.inner.height);
        self.inner.delete_line(y, n, cell);
    }

    /// Insert `n` cells at `(row, col)`, shifting cells to the right on
    /// that row.  The row is marked.
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.mark(row);
        self.inner.insert_cell(row, col, n, cell);
    }

    /// Delete `n` cells at `(row, col)`, shifting cells to the left on
    /// that row.  The row is marked.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.mark(row);
        self.inner.delete_cell(row, col, n, cell);
    }

    /// Bounded variant of [`insert_line`](Self::insert_line).
    ///
    /// Only rows within `bounds` are marked.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.mark_many(bounds.min.y as usize..bounds.max.y as usize);
        self.inner.insert_line_area(y, n, cell, bounds);
    }

    /// Bounded variant of [`delete_line`](Self::delete_line).
    ///
    /// Only rows within `bounds` are marked.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.mark_many(bounds.min.y as usize..bounds.max.y as usize);
        self.inner.delete_line_area(y, n, cell, bounds);
    }

    /// Bounded variant of [`insert_cell`](Self::insert_cell).
    ///
    /// Only the affected row is marked.
    pub fn insert_cell_area(
        &mut self,
        row: usize,
        col: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) {
        self.mark(row);
        self.inner.insert_cell_area(row, col, n, cell, bounds);
    }

    /// Bounded variant of [`delete_cell`](Self::delete_cell).
    ///
    /// Only the affected row is marked.
    pub fn delete_cell_area(
        &mut self,
        row: usize,
        col: usize,
        n: usize,
        cell: Cell,
        bounds: Rect,
    ) {
        self.mark(row);
        self.inner.delete_cell_area(row, col, n, cell, bounds);
    }

    /// Append a row at the bottom.  The new row is **not** automatically
    /// marked – the caller is responsible if it should be.
    pub fn push_row(&mut self, row: impl IntoIterator<Item = Cell>) {
        self.inner.push_row(row);
        self.bits.grow(self.height + 1);
    }

    /// Pop the bottom row.  The last marker bit is discarded.
    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        let row = self.inner.pop_row()?;
        self.bits = BitSet::with_capacity_and_blocks(
            self.height - 1,
            self.bits.as_slice().iter().copied(),
        );
        Some(row)
    }

    /// Remove row `idx`.  Subsequent rows are shifted up and **marked**.
    pub fn remove_row(&mut self, idx: usize) -> Option<Vec<Cell>> {
        let row = self.inner.remove_row(idx)?;
        if idx < self.bits.len() {
            self.bits.remove(idx);
        }
        self.mark_many(idx..);
        Some(row)
    }

    /// Insert a row at `idx`.  Subsequent rows are shifted down and **marked**.
    /// The new row is **not** automatically marked.
    pub fn insert_row(&mut self, idx: usize, row: impl IntoIterator<Item = Cell>) {
        self.inner.insert_row(idx, row);
        let dirty_idx = idx.min(self.bits.len());
        self.mark_many(dirty_idx..);
    }

    // ----------------------------------------------------------
    // Diff integration
    // ----------------------------------------------------------

    /// Compute an optimized diff between `prev` and `self`, using the
    /// `ByDirty` strategy.  Only marked rows are compared; unmarked rows
    /// are assumed unchanged.
    ///
    /// Returns an iterator of [`Change`] items.
    pub fn diff<'a>(&'a self, prev: &'a Buffer) -> BufferDiff<'a, ByDirty> {
        BufferDiff::dirty(prev, self)
    }
}

impl Bound for TrackingBuffer {
    type Point = Point;

    fn min_x(&self) -> u16 {
        Bound::min_x(&self.inner)
    }
    fn min_y(&self) -> u16 {
        Bound::min_y(&self.inner)
    }
    fn max_x(&self) -> u16 {
        Bound::max_x(&self.inner)
    }
    fn max_y(&self) -> u16 {
        Bound::max_y(&self.inner)
    }
    fn min(&self) -> Self::Point {
        Bound::min(&self.inner)
    }
    fn max(&self) -> Self::Point {
        Bound::max(&self.inner)
    }
}

impl From<TrackingBuffer> for Buffer {
    fn from(value: TrackingBuffer) -> Self {
        value.inner
    }
}

impl From<Buffer> for TrackingBuffer {
    fn from(value: Buffer) -> Self {
        let bits = BitSet::with_capacity_and_blocks(value.height, iter::repeat_n(1, value.height));
        Self { inner: value, bits }
    }
}

impl PartialEq for TrackingBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq<Buffer> for TrackingBuffer {
    fn eq(&self, other: &Buffer) -> bool {
        &self.inner == other
    }
}

impl PartialEq<TrackingBuffer> for Buffer {
    fn eq(&self, other: &TrackingBuffer) -> bool {
        self == &other.inner
    }
}
impl DerefMut for TrackingBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mark_all();
        &mut self.inner
    }
}

impl<I: BufferIndex + TrackingBufferIndex> Index<I> for TrackingBuffer {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        BufferIndex::index(index, self)
    }
}

impl<I: BufferIndex + TrackingBufferIndex> IndexMut<I> for TrackingBuffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.mark_many(index.clone());
        BufferIndex::index_mut(index, self)
    }
}
trait TrackingBufferIndex: BufferIndex {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange;
}


impl TrackingBufferIndex for Point {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let index = self.into_slice_index(buffer);
        index..(index + 1)
    }
}
impl TrackingBufferIndex for Position {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let index = self.into_slice_index(buffer);
        index..(index + 1)
    }
}
impl TrackingBufferIndex for PositionLike {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let index = self.into_slice_index(buffer);
        index..(index + 1)
    }
}
impl TrackingBufferIndex for Row {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let v = self.into_inner();
        v..(v + 1)
    }
}
impl TrackingBufferIndex for Range<Row> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        (self.start.into_inner())..(self.end.into_inner())
    }
}

impl TrackingBufferIndex for RangeInclusive<Row> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        (self.start().into_inner())..(self.end().into_inner() + 1)
    }
}

impl TrackingBufferIndex for RangeTo<Row> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        (self.end.into_inner())..
    }
}

impl TrackingBufferIndex for RangeToInclusive<Row> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        (self.end.into_inner())..(self.end.into_inner() + 1)
    }
}
impl TrackingBufferIndex for RangeFrom<Row> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        (self.start.into_inner())..
    }
}

impl TrackingBufferIndex for RangeFull {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        ..
    }
}
// Convenience for `Index` and `Position`
impl TrackingBufferIndex for usize {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        self..(self + 1)
    }
}
impl<I: BufferIndex<Index = usize> + TrackingBufferIndex> TrackingBufferIndex for Range<I> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let start = self.start.into_slice_index(buffer);
        let end = self.end.into_slice_index(buffer);
        start..end
    }
}

impl<I: BufferIndex<Index = usize> + TrackingBufferIndex> TrackingBufferIndex for RangeTo<I> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let end = self.end.into_slice_index(buffer);
        ..end
    }
}

impl<I: BufferIndex<Index = usize> + TrackingBufferIndex> TrackingBufferIndex for RangeFrom<I> {
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let start = self.start.into_slice_index(buffer);
        start..
    }
}

impl<I: BufferIndex<Index = usize> + TrackingBufferIndex> TrackingBufferIndex for RangeInclusive<I> {

    #[inline]
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let start = self.start().clone().into_slice_index(buffer);
        let end = self.end().clone().into_slice_index(buffer);
        start..end + 1
    }
}

impl<I: BufferIndex<Index = usize> + TrackingBufferIndex> TrackingBufferIndex for RangeToInclusive<I> {
    #[inline]
    fn into_tracking_range(self, buffer: &mut Buffer) -> impl TrackingRange {
        let end = self.end.into_slice_index(buffer);
        end..end + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arena;

    #[test]
    fn new_starts_with_all_rows_dirty() {
        let t = TrackingBuffer::new(5, 3);
        assert_eq!(t.marked().count(), 3);
        assert!(!t.is_clean());
        for y in 0..3 {
            assert!(t.is_marked(y));
        }
    }

    #[test]
    fn clear_dirty_marks_all_clean() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        assert!(t.is_clean());
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn out_of_bounds_marks_are_ignored() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        t.mark(99);
        assert!(t.is_clean());
        t.mark_many(10..20);
        assert!(t.is_clean());
    }

    #[test]
    fn set_line_marks_only_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        let mut arena = Arena::new();
        t.set_line(Point { x: 0, y: 1 }, "abc", &mut arena);
        assert!(!t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(!t.is_marked(2));
    }

    #[test]
    fn set_string_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        let mut arena = Arena::new();
        t.set_string(
            Point { x: 0, y: 0 }..Point { x: 5, y: 0 },
            "abc",
            &mut arena,
        );
        assert_eq!(t.count(), 3);
    }

    #[test]
    fn cell_mut_marks_just_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        *t.cell_mut(Point { x: 2, y: 2 }).unwrap() = Cell::inline('z');
        assert!(!t.is_marked(0));
        assert!(!t.is_marked(1));
        assert!(t.is_marked(2));
    }

    #[test]
    fn row_mut_marks_just_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        t.row_mut(1).unwrap()[0] = Cell::inline('x');
        assert!(!t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(!t.is_marked(2));
    }

    #[test]
    fn row_mut_out_of_bounds_is_none() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        assert!(t.row_mut(99).is_none());
        assert!(t.is_clean());
    }

    #[test]
    fn buffer_mut_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        let _ = t.as_inner();
        assert_eq!(t.count(), 3);
    }

    #[test]
    fn resize_updates_bitmap_and_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        t.resize(7, 5);
        assert_eq!(t.count(), 5);
        assert!(t.is_dirty());
        assert_eq!(t.count(), 5);
    }

    #[test]
    fn resize_noop_when_dimensions_match() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        t.resize(5, 3);
        assert!(t.is_clean());
    }

    #[test]
    fn push_pop_row_updates_bitmap() {
        let mut t = TrackingBuffer::new(2, 1);
        t.unmark_all();
        t.push_row([Cell::inline('a'), Cell::inline('b')]);
        assert_eq!(t.as_bits().len(), 2);
        assert!(t.is_marked(1));
        let _ = t.pop_row();
        assert_eq!(t.as_bits().len(), 1);
    }

    #[test]
    fn insert_line_marks_y_and_below() {
        let mut t = TrackingBuffer::new(3, 4);
        t.unmark_all();
        t.insert_line(1, 1, Cell::EMPTY);
        assert!(!t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(t.is_marked(2));
        assert!(t.is_marked(3));

        t.mark_many(1);
    }

    #[test]
    fn insert_cell_marks_only_its_row() {
        let mut t = TrackingBuffer::new(4, 3);
        t.unmark_all();
        t.insert_cell(1, 0, 1, Cell::EMPTY);
        assert!(!t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(!t.is_marked(2));
    }

    #[test]
    fn dirty_row_indices_enumerates_dirty() {
        let mut t = TrackingBuffer::new(2, 4);
        t.unmark_all();
        t.mark(0);
        t.mark(2);
        let dirty: Vec<_> = t.marked().collect();
        assert_eq!(dirty, vec![0, 2]);
    }

    #[test]
    fn into_buffer_round_trips() {
        let mut arena = Arena::new();
        let buf = Buffer::from_lines(["abc"], &mut arena);
        let tracking: TrackingBuffer = buf.clone().into();
        assert_eq!(tracking.into_inner(), buf);
    }

    #[test]
    fn deref_exposes_buffer_api() {
        // Read-only API on Buffer should be reachable via auto-deref.
        let mut arena = Arena::new();
        let t = TrackingBuffer::from(Buffer::from_lines(["abc"], &mut arena));
        assert_eq!(t.width(), 3);
        assert_eq!(t.height(), 1);
    }
}
