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
#[derive(Clone, Debug, Deref, AsRef, From)]
pub struct TrackingBuffer {
    #[deref]
    #[as_ref]
    inner: Buffer,
    /// Per-row dirty flags. Invariant: `dirty.len() == inner.height`.
    bits: BitSet
}

impl TrackingBuffer {
    /// Empty tracking buffer.
    pub const EMPTY: Self = Self {
        inner: Buffer::EMPTY,
        bits: BitSet::new(),
    };

    /// Create a new tracking buffer of the given size with every row dirty.
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

    /// Creates a tracking buffer from a buffer.
    ///
    /// Scans for non-empty rows and marks them dirty.
    pub fn from_buffer_scanned(buffer: Buffer) -> Self {
        let bits = BitSet::with_capacity_and_blocks(buffer.height, buffer.iter_rows().map(|row| !row.is_empty() as usize));

        Self {
            inner: buffer,
            bits,
        }
    }

    /// Creates a tracking buffer from a buffer.
    ///
    /// All rows are marked dirty.
    pub fn from_buffer_dirty(buffer: Buffer) -> Self {
        Self {
            bits: BitSet::with_capacity_and_blocks(buffer.height, iter::repeat_n(1, buffer.height)),
            inner: buffer,
        }
    }

    /// Creates a tracking buffer from a buffer.
    ///
    /// Rows are unmarked.
    pub fn from_buffer_clean(buffer: Buffer) -> Self {
        Self {
            bits: BitSet::with_capacity_and_blocks(buffer.height, iter::repeat_n(0, buffer.height)),
            inner: buffer,
        }
    }

    #[inline]
    pub fn mark(&mut self, y: usize) {
        self.bits.set(y, true);
    }

    #[inline]
    pub fn mark_many(&mut self, y_range: impl TrackingBufferIndex) {
        let range = y_range.into_tracking_range(&mut self.inner);
        self.bits.set_range(range, true);
    }

    /// Mark every row dirty.
    #[inline]
    pub fn mark_all(&mut self) {
        self.mark_many(..);
    }

    #[inline]
    pub fn unmark(&mut self, y: usize) {
        self.bits.set(y, false);
    }

    #[inline]
    pub fn unmark_many(&mut self, y_range: impl TrackingRange) {
        self.bits.set_range(y_range, false);
    }

    /// Reset all dirty flags. Call this once the previous frame has been
    /// applied so the next [`diff_dirty`](Self::diff) can skip
    /// untouched rows.
    #[inline]
    pub fn unmark_all(&mut self) {
        self.bits.clear();
    }

    /// Whether row `y` is marked dirty. Out-of-bounds rows return `false`.
    #[inline]
    pub fn is_marked(&self, y: usize) -> bool {
        self.bits.contains(y)
    }

    #[inline]
    pub fn is_any_marked(&self, y_range: impl TrackingRange) -> bool {
        self.bits.contains_any_in_range(y_range)
    }

    #[inline]
    pub fn is_all_marked(&self, y_range: impl TrackingRange) -> bool {
        self.bits.contains_all_in_range(y_range)
    }

    #[inline]
    pub fn is_any_unmarked(&self, y_range: impl TrackingRange) -> bool {
        !self.is_all_marked(y_range)
    }

    #[inline]
    pub fn is_all_unmarked(&self, y_range: impl TrackingRange) -> bool {
        !self.is_any_marked(y_range)
    }

    #[inline]
    pub fn is_clean(&self) -> bool {
        self.bits.is_empty()
    }

    /// Unmark all rows.
    pub fn clean(&mut self) {
        self.bits.clear();
    }

    /// Returns true if any rows are marked.
    pub fn is_dirty(&self) -> bool {
        !self.is_clean()
    }

    /// Mark all rows.
    pub fn dirty(&mut self) {
        self.bits.set_range(.., true);
    }

    /// Iterator over the indices of rows currently marked dirty.
    pub fn marked(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.ones()
    }

    /// Returns the number of rows marked dirty.
    pub fn count(&self) -> usize {
        self.count_marked(..)
    }

    /// Returns the number of rows marked dirty in the given range.
    pub fn count_marked(&self, range: impl TrackingRange) -> usize {
        self.bits.count_ones(range)
    }

    /// Returns the number of rows not marked dirty in the given range.
    pub fn count_unmarked(&self, range: impl TrackingRange) -> usize {
        self.bits.count_zeroes(range)
    }

    /// Borrow the bitset of this tracking buffer.
    #[inline]
    pub fn as_bits(&self) -> &BitSet {
        &self.bits
    }

    /// Borrow the inner buffer.
    #[inline]
    pub fn as_inner(&self) -> &Buffer {
        &self.inner
    }

    /// Mutable borrow of the inner buffer.
    ///
    /// Escape hatch: Bypasses dirty tracking.
    pub fn as_mut_inner(&mut self) -> &mut Buffer {
        &mut self.inner
    }

    /// Consume `self` and return the inner buffer, dropping dirty state.
    #[inline]
    pub fn into_inner(self) -> Buffer {
        self.inner
    }

    // --------------------------------------------------------------
    // Granular mutators
    // --------------------------------------------------------------


    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: BufferIndex>(
        &mut self,
        index: I,
    ) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output> {
        index.get_mut(self)
    }
    /// Mutable access to row `y`, marking it dirty.
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [Cell]> {
        if y >= self.inner.height {
            return None;
        }
        self.mark(y);
        let width = self.inner.width;
        Some(&mut self.inner[y * width..(y + 1) * width])
    }

    /// Mutable access to a single cell, marking its row dirty.
    pub fn cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        self.mark(point.y as usize);
        self.inner.get_mut(point)
    }

    // --------------------------------------------------------------
    // Wrapped mutation methods
    // --------------------------------------------------------------

    /// Clear the buffer to default cells.
    pub fn clear(&mut self) {
        self.mark_many(..);
        self.inner.clear();
    }

    /// Resize the buffer. Marks every row dirty since width changes shift
    /// existing cells.
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

    /// Write a string starting at `start`, marking the affected row dirty.
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
        self.mark_all();
        self.inner.set_string(index, string, arena)
    }

    /// Insert `n` lines at row `y`, shifting following rows down.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.mark_many(y..self.inner.height);
        self.inner.insert_line(y, n, cell);
    }

    /// Delete `n` lines at row `y`, shifting following rows up.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.mark_many(y..self.inner.height);
        self.inner.delete_line(y, n, cell);
    }

    /// Insert `n` cells at `(row, col)`, shifting cells right on that row.
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.mark(row);
        self.inner.insert_cell(row, col, n, cell);
    }

    /// Delete `n` cells at `(row, col)`, shifting cells left on that row.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.mark(row);
        self.inner.delete_cell(row, col, n, cell);
    }

    /// Bounded variant of [`insert_line`](Self::insert_line).
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.mark_many(bounds.min.y as usize..bounds.max.y as usize);
        self.inner.insert_line_area(y, n, cell, bounds);
    }

    /// Bounded variant of [`delete_line`](Self::delete_line).
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.mark_many(bounds.min.y as usize..bounds.max.y as usize);
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
        self.mark(row);
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
        self.mark(row);
        self.inner.delete_cell_area(row, col, n, cell, bounds);
    }

    /// Append a row.
    pub fn push_row(&mut self, row: impl IntoIterator<Item = Cell>) {
        self.inner.push_row(row);
        self.bits.grow(self.height + 1);
    }

    /// Pop the bottom row.
    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        let row = self.inner.pop_row()?;
        self.bits = BitSet::with_capacity_and_blocks(self.height - 1, self.bits.as_slice().iter().copied());
        Some(row)
    }

    /// Remove row `idx`. Following rows shift up and are marked dirty.
    pub fn remove_row(&mut self, idx: usize) -> Option<Vec<Cell>> {
        let row = self.inner.remove_row(idx)?;
        if idx < self.bits.len() {
            self.bits.remove(idx);
        }
        self.mark_many(idx..);
        Some(row)
    }

    /// Insert a row at `idx`. Following rows shift down and are marked dirty.
    pub fn insert_row(&mut self, idx: usize, row: impl IntoIterator<Item = Cell>) {
        self.inner.insert_row(idx, row);
        let dirty_idx = idx.min(self.bits.len());
        self.mark_many(dirty_idx..);
    }

    // --------------------------------------------------------------
    // Diff integration
    // --------------------------------------------------------------

    /// Optimized diff using the [`ByDirty`] diff strategy.
    ///
    /// Yields [`Change`]s.
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
