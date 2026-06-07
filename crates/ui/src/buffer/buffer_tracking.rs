//! A [`Buffer`] with per-row change tracking.
//!
//! `TrackingBuffer` wraps a [`Buffer`] and keeps one marker bit per row.
//! Read-only access mirrors the inner buffer through [`Deref`]. Mutating
//! operations are provided as dedicated methods that mark the rows they touch.
//! A subsequent [`diff`](TrackingBuffer::diff) can then skip unmarked rows.
//!
//! # Terminology
//!
//! - A **marked** row may have changed since the last call to
//!   [`unmark_all`](TrackingBuffer::unmark_all) or [`clean`](TrackingBuffer::clean).
//! - An **unmarked** row is assumed unchanged.
//! - A buffer is **dirty** when at least one row is marked.
//! - A buffer is **clean** when no rows are marked.
//!
//! # Lifecycle
//!
//! 1. Create a `TrackingBuffer`. By default, all rows are marked so the first
//!    diff sees the full buffer.
//! 2. Mutate the buffer through tracking-aware methods.
//! 3. Call [`diff`](TrackingBuffer::diff) and apply the produced changes.
//! 4. Call [`unmark_all`](TrackingBuffer::unmark_all) or [`clean`](TrackingBuffer::clean) before
//!    rendering the next frame.
//!
//! # Invariants
//!
//! - `self.bits.len() == self.inner.height`.
//! - Row `y` is marked iff `self.bits.contains(y)`.
use crate::{Arena, Buffer, BufferDiff, BufferIndex, ByDirty, Cell};
use derive_more::{AsRef, Deref, From};
use geometry::{Bound, Point, Position, PositionLike, Rect, Row};
use std::fmt::Debug;
use std::ops::{DerefMut, Index, IndexMut};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice::SliceIndex;
use thiserror::Error;

pub type Map = sparsemap::SparseMap;

/// Errors that can occur when manipulating a [`TrackingBuffer`].
#[derive(Error, Debug)]
pub enum TrackingBufferError {
    #[error("Index out of bounds")]
    OutOfBounds,
}

#[derive(Clone, Debug, Deref, AsRef, From)]
pub struct TrackingBuffer {
    #[deref]
    #[as_ref]
    pub(super) inner: Buffer,

    /// Per-row markers.
    ///
    /// Invariant: `bits.len() == inner.height`.
    pub(super) map: Map,
}

impl TrackingBuffer {
    /// An empty tracking buffer.
    ///
    /// The buffer has no rows, so it is both clean and contains no markers.
    pub const EMPTY: Self = Self {
        inner: Buffer::EMPTY,
        map: Map::new(),
    };

    /// Creates a tracking buffer with all rows marked.
    ///
    /// This is the default constructor because a freshly-created buffer should
    /// usually be fully diffed once before incremental tracking begins.
    pub fn new(width: usize, height: usize) -> Self {
        Self::new_marked(width, height)
    }

    /// Creates a tracking buffer with all rows marked.
    ///
    /// This is equivalent to [`new`](Self::new), but makes the initial marker
    /// state explicit at the call site.
    pub fn new_marked(width: usize, height: usize) -> Self {
        let mut bits = Map::new();
        bits.insert_range(0, height as u64);
        Self {
            inner: Buffer::new(width, height),
            map: bits,
        }
    }

    /// Creates a tracking buffer with no rows marked.
    ///
    /// Use this when the caller knows the newly-created buffer is already in
    /// sync with its comparison target.
    pub fn new_unmarked(width: usize, height: usize) -> Self {
        Self {
            inner: Buffer::new(width, height),
            map: Map::new(),
        }
    }

    /// Wraps an existing [`Buffer`] and marks every non-empty row.
    ///
    /// Empty rows are left unmarked. This is useful when adopting an existing
    /// buffer whose meaningful contents should be included in the next diff.
    pub fn from_buffer_checked(buffer: Buffer) -> Self {
        let mut bits = Map::new();

        for (y, row) in buffer.iter_rows().enumerate() {
            if !row.is_empty() {
                bits.insert(y as u64);
            }
        }

        Self {
            inner: buffer,
            map: bits,
        }
    }

    /// Wraps an existing [`Buffer`] and marks every row.
    pub fn from_buffer_marked(buffer: Buffer) -> Self {
        let mut map = Map::new();
        map.insert_range(0, buffer.height as u64);
        Self {
            inner: buffer,
            map,
        }
    }

    /// Wraps an existing [`Buffer`] and leaves every row unmarked.
    pub fn from_buffer_unmarked(buffer: Buffer) -> Self {
        let bits = Map::new();
        Self {
            inner: buffer,
            map: bits,
        }
    }

    // ----------------------------------------------------------
    // Marker manipulation
    // ----------------------------------------------------------

    /// Marks every row in the given index.
    ///
    /// **Panics** if out-of-bounds.
    #[inline]
    pub fn mark<I: TrackingBufferIndex>(&mut self, index: I) {
        index.mark(self);
    }

    /// Tries to mark every row in the given index.
    #[inline]
    pub fn try_mark(&mut self, index: impl TrackingBufferIndex) -> Result<(), TrackingBufferError> {
        index.try_mark(self)
    }

    /// Marks every row.
    #[inline]
    pub fn mark_all(&mut self) {
        self.mark(..);
    }

    /// Unmarks every row in the given index.
    ///
    /// **Panics** if out-of-bounds.
    #[inline]
    pub fn unmark<I: TrackingBufferIndex>(&mut self, index: I) {
        index.unmark(self);
    }

    /// Tries to unmark every row in the given index.
    #[inline]
    pub fn try_unmark(
        &mut self,
        index: impl TrackingBufferIndex,
    ) -> Result<(), TrackingBufferError> {
        index.try_unmark(self)
    }

    /// Unmarks every row.
    ///
    /// Call this after applying a diff so the next diff only compares rows
    /// touched after this call.
    #[inline]
    pub fn unmark_all(&mut self) {
        self.map.clear();
    }

    /// Returns `true` if row `y` is marked.
    ///
    /// Out-of-bounds rows return `false`.
    #[inline]
    pub fn is_marked(&self, y: usize) -> bool {
        self.map.contains(y as u64)
    }


    /// Returns `true` if no rows are marked.
    #[inline]
    pub fn is_clean(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns `true` if at least one row is marked.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        !self.is_clean()
    }

    /// Unmarks every row.
    ///
    /// After this call, [`is_clean`](Self::is_clean) returns `true`.
    #[inline]
    pub fn clean(&mut self) {
        self.unmark_all();
    }
    /// Marks every row.
    ///
    /// After this call, [`is_dirty`](Self::is_dirty) returns `true` unless the
    /// buffer has zero rows.
    #[inline]
    pub fn dirty(&mut self) {
        self.mark_all();
    }

    /// Returns the number of marked rows.
    pub fn count_marked(&self) -> usize {
        self.map.cardinality() as usize
    }

    /// Returns the number of unmarked rows.
    pub fn count_unmarked(&self) -> usize {
        self.inner.height - self.count_marked()
    }

    /// Returns the raw marker set.
    ///
    /// `marks().contains(y)` is `true` when row `y` is marked.
    #[inline]
    pub fn as_bits(&self) -> &Map {
        &self.map
    }

    /// Iterates over marked row indices.
    pub fn iter_marked(&self) -> impl Iterator<Item=usize> + '_ {
        self.map.iter().map(|x| x as usize)
    }

    // ----------------------------------------------------------
    // Inner buffer access
    // ----------------------------------------------------------

    /// Consumes the tracking wrapper and returns the inner [`Buffer`].
    ///
    /// Marker state is discarded.
    #[inline]
    pub fn into_inner(self) -> Buffer {
        self.inner
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: TrackingBufferIndex>(
        &mut self,
        index: I,
    ) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output> {
        index.clone().mark(self);
        self.inner.get_mut(index)
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<I: TrackingBufferIndex>(
        &mut self,
        index: I,
    ) -> *mut <I::Index as SliceIndex<[Cell]>>::Output {
        unsafe {
            index.clone().mark(self);
            self.inner.get_unchecked_mut(index)
        }
    }

    /// Clears the buffer and marks every row.
    pub fn clear(&mut self) {
        self.mark_all();
        self.inner.clear();
    }

    /// Resizes the buffer and marks every remaining row.
    ///
    /// Resizing can move existing cells, remove rows, or create new rows, so
    /// the conservative and predictable behavior is to mark the full resized
    /// buffer whenever the dimensions change.
    pub fn resize(&mut self, next_width: usize, next_height: usize) {
        let previous_height = self.inner.height;
        let previous_width = self.inner.width;

        if previous_width == next_width && previous_height == next_height {
            return;
        }

        // If the height shrinks, remove the excess rows.
        if next_height < previous_height {
            self.map.remove_range(next_height as u64, previous_height as u64);
        }

        // If the height grows, add the new rows.
        if next_height > previous_height {
            self.map.insert_range(0, next_height as u64);
        }

        self.inner.resize(next_width, next_height);
    }

    /// Writes `string` starting at `start` and marks `start.y`.
    ///
    /// Returns the number of written cells, or `None` if the write starts out
    /// of bounds.
    pub fn set_line(
        &mut self,
        start: Point,
        string: impl AsRef<str>,
        arena: &mut Arena,
    ) -> Option<usize> {
        let written = self.inner.set_line(start, string, arena)?;
        self.mark(start.y as usize);
        Some(written)
    }

    /// Writes `string` at `index` and marks every row.
    ///
    /// The generic index does not expose which rows are touched, so this method
    /// marks all rows conservatively. Prefer [`set_line`](Self::set_line) or
    /// [`row_mut`](Self::row_mut) when the affected row is known.
    pub fn set_string<I>(
        &mut self,
        index: I,
        string: impl AsRef<str>,
        arena: &mut Arena,
    ) -> Option<usize>
    where
        I: BufferIndex<Output=[Cell]>,
    {
        let written = self.inner.set_string(index, string, arena)?;
        self.mark_all();
        Some(written)
    }

    /// Inserts `n` rows at `y` and marks exactly the rows the buffer changed.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        let changed = self.inner.insert_line(y, n, cell);
        self.mark(changed);
    }

    /// Deletes `n` rows at `y` and marks exactly the rows the buffer changed.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        let changed = self.inner.delete_line(y, n, cell);
        self.mark(changed);
    }

    /// Inserts `n` cells into `row` at `col` and marks exactly the rows the
    /// buffer changed.
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        let changed = self.inner.insert_cell(row, col, n, cell);
        self.mark(changed);
    }

    /// Deletes `n` cells from `row` at `col` and marks exactly the rows the
    /// buffer changed.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        let changed = self.inner.delete_cell(row, col, n, cell);
        self.mark(changed);
    }

    /// Inserts `n` rows within `bounds` and marks exactly the rows the buffer
    /// changed.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        let changed = self.inner.insert_line_area(y, n, cell, bounds);
        self.mark(changed);
    }

    /// Deletes `n` rows within `bounds` and marks exactly the rows the buffer
    /// changed.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        let changed = self.inner.delete_line_area(y, n, cell, bounds);
        self.mark(changed);
    }

    /// Inserts `n` cells within `bounds` and marks exactly the rows the buffer
    /// changed.
    pub fn insert_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        let changed = self.inner.insert_cell_area(row, col, n, cell, bounds);
        self.mark(changed);
    }

    /// Deletes `n` cells within `bounds` and marks exactly the rows the buffer
    /// changed.
    pub fn delete_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        let changed = self.inner.delete_cell_area(row, col, n, cell, bounds);
        self.mark(changed);
    }

    /// Appends a row and leaves the new row unmarked.
    pub fn push_row_unmarked(&mut self, row: impl IntoIterator<Item=Cell>) {
        self.inner.push_row(row);
    }

    /// Appends a row and marks it.
    pub fn push_row_marked(&mut self, row: impl IntoIterator<Item=Cell>) {
        self.push_row_unmarked(row);
        self.mark(self.height - 1);
    }

    /// Removes and returns the last row.
    ///
    /// The removed row's marker is discarded.
    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        let row = self.inner.pop_row()?;

        // The popped row was the last one, so every surviving marker keeps its
        // index. Only the marker for the now-removed row can be out of range, so
        // drop just that single trailing bit.
        self.map.remove(self.inner.height as u64);

        Some(row)
    }

    /// Removes row and does not mark the rows shifted up.
    pub fn remove_row_unmarked(&mut self, idx: usize) -> Option<Vec<Cell>> {
        let row = self.inner.remove_row(idx)?; // idx < old height guaranteed

        // Removing row `idx` shifts everything below it up by one:
        // `new[i] = old[i]` for `i < idx`, and `new[i] = old[i + 1]` otherwise.
        // Shifted rows keep their existing marks (this method does not add any).
        let idx = idx as u64;

        // Markers strictly below `idx` stay put; markers strictly above `idx`
        // move up one row; the marker for the removed row itself is dropped.
        let mut below = Map::new();
        below.insert_range(0, idx); // [0, idx)
        let mut through = Map::new();
        through.insert_range(0, idx + 1); // [0, idx]

        let low = &self.map & &below;
        let high = (&self.map - &through).shifted(-1);
        self.map = &low | &high;

        Some(row)
    }

    /// Removes row and marks the rows shifted up.
    pub fn remove_row_marked(&mut self, idx: usize) -> Option<Vec<Cell>> {
        self.remove_row_unmarked(idx).inspect(|_row| {
            self.mark(idx..);
        })
    }

    /// Inserts a row and marks rows shifted down.
    /// The inserted row is unmarked.
    pub fn insert_row(&mut self, idx: usize, row: impl IntoIterator<Item=Cell>) {
        if idx > self.height {
            return;
        }

        self.inner.insert_row(idx, row);

        // Inserting at `idx` shifts everything from `idx` down by one:
        // `new[i] = old[i]` for `i < idx`, the inserted row at `idx` is left
        // unmarked, and `new[i] = old[i - 1]` for `i > idx`.
        let idx = idx as u64;

        // Markers below `idx` stay put; markers at or above `idx` move down one
        // row, leaving the freshly inserted row at `idx` unmarked.
        let mut below = Map::new();
        below.insert_range(0, idx); // [0, idx)

        let low = &self.map & &below;
        let high = (&self.map - &below).shifted(1);
        self.map = &low | &high;
    }

    // ----------------------------------------------------------
    // Diffing
    // ----------------------------------------------------------

    /// Diffs `self` against `prev`, comparing only marked rows.
    ///
    /// Unmarked rows are assumed unchanged. After applying the returned
    /// changes, call [`unmark_all`](Self::unmark_all) or [`clean`](Self::clean)
    /// before beginning the next frame.
    pub fn diff<'a>(&'a self, prev: &'a Buffer) -> BufferDiff<'a, ByDirty> {
        BufferDiff::dirty(prev, self)
    }

    /// Returns the inner buffer.
    #[inline]
    pub fn as_inner(&self) -> &Buffer {
        &self.inner
    }

    /// Returns the inner buffer mutably **without marking rows**.
    ///
    /// This is an escape hatch: mutations made through this reference are not
    /// tracked, so a subsequent [`diff`](Self::diff) may miss them. It is a
    /// logic hazard, not a memory-safety one — prefer the tracking-aware
    /// mutators on `TrackingBuffer` whenever possible.
    #[inline]
    pub fn as_mut_inner(&mut self) -> &mut Buffer {
        &mut self.inner
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
        Self::from_buffer_marked(value)
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
        &mut self.inner
    }
}

impl<I: TrackingBufferIndex> Index<I> for TrackingBuffer {
    type Output = <I as BufferIndex>::Output;
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.inner, index)
    }
}

impl<I: TrackingBufferIndex> IndexMut<I> for TrackingBuffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.clone().mark(self);
        IndexMut::index_mut(&mut self.inner, index)
    }
}
pub trait TrackingBufferIndex: BufferIndex {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError>;
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError>;

    fn mark(&self, tracking_buffer: &mut TrackingBuffer) {
        self.try_mark(tracking_buffer).unwrap();
    }

    fn unmark(&self, tracking_buffer: &mut TrackingBuffer) {
        self.try_unmark(tracking_buffer).unwrap();
    }
}

impl TrackingBufferIndex for usize {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let index = *self;
        if index >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.map.insert(index as u64);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let index = *self;
        if index >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.map.remove(index as u64);
        Ok(())
    }
}
impl TrackingBufferIndex for Row {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(&self.into_inner(), tracking_buffer)
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(&self.into_inner(), tracking_buffer)
    }
}
impl TrackingBufferIndex for Point {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(&(self.y as usize), tracking_buffer)
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(&(self.y as usize), tracking_buffer)
    }
}
impl TrackingBufferIndex for Position {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(&self.row, tracking_buffer)
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(&self.row, tracking_buffer)
    }
}
impl TrackingBufferIndex for PositionLike {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(&self.1, tracking_buffer)
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(&self.1, tracking_buffer)
    }
}

impl<I: BufferIndex<Index=usize>> TrackingBufferIndex for Range<I> {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.clone().into_slice_index(tracking_buffer);
        let end = self.end.clone().into_slice_index(tracking_buffer);

        // `end` is exclusive, so `end == height` is the in-bounds "whole buffer".
        if end > tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.map.insert_range(start as u64, end as u64);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.clone().into_slice_index(tracking_buffer);
        let end = self.end.clone().into_slice_index(tracking_buffer);

        if end > tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.map.remove_range(start as u64, end as u64);
        Ok(())
    }
}
impl<I: BufferIndex<Index=usize>> TrackingBufferIndex for RangeInclusive<I> {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start().clone().into_slice_index(tracking_buffer);
        let end = self.end().clone().into_slice_index(tracking_buffer);

        if end >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.map.insert_range(start as u64, end as u64 + 1);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start().clone().into_slice_index(tracking_buffer);
        let end = self.end().clone().into_slice_index(tracking_buffer);

        if end >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.map.remove_range(start as u64, end as u64 + 1);
        Ok(())
    }
}
impl<I: BufferIndex<Index=usize>> TrackingBufferIndex for RangeFrom<I> {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.clone().into_slice_index(tracking_buffer);
        let end = tracking_buffer.height;

        tracking_buffer.map.insert_range(start as u64, end as u64);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.clone().into_slice_index(tracking_buffer);
        let end = tracking_buffer.height;

        tracking_buffer.map.remove_range(start as u64, end as u64);
        Ok(())
    }
}
impl<I: BufferIndex<Index=usize>> TrackingBufferIndex for RangeTo<I> {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.clone().into_slice_index(tracking_buffer);

        tracking_buffer.map.insert_range(0, end as u64);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.clone().into_slice_index(tracking_buffer);

        tracking_buffer.map.remove_range(0, end as u64);
        Ok(())
    }
}
impl<I: BufferIndex<Index=usize>> TrackingBufferIndex for RangeToInclusive<I> {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.clone().into_slice_index(tracking_buffer);

        tracking_buffer.map.insert_range(0, end as u64 + 1);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.clone().into_slice_index(tracking_buffer);

        tracking_buffer.map.remove_range(0, end as u64 + 1);
        Ok(())
    }
}

impl TrackingBufferIndex for RangeFull {
    fn try_mark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        tracking_buffer.map.insert_range(0, tracking_buffer.height as u64);
        Ok(())
    }
    fn try_unmark(&self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        tracking_buffer.map.remove_range(0, tracking_buffer.height as u64);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arena;

    #[test]
    fn new_starts_with_all_rows_dirty() {
        let t = TrackingBuffer::new(5, 3);
        assert_eq!(t.count_marked(), 3);
        assert!(!t.is_clean());
        for y in 0..3 {
            assert!(t.is_marked(y));
        }
    }

    #[test]
    fn new_marked_starts_with_all_rows_marked() {
        let t = TrackingBuffer::new_marked(5, 3);
        assert!(t.is_dirty());
        for y in 0..3 {
            assert!(t.is_marked(y));
        }
    }

    #[test]
    fn clear_dirty_marks_all_clean() {
        let mut t = TrackingBuffer::new_marked(5, 3);
        assert!(t.is_dirty());
        assert_eq!(t.count_marked(), 3);
        t.unmark_all();
        assert!(t.is_clean());
        assert_eq!(t.count_marked(), 0);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_marks_trigger_exception() {
        let mut t = TrackingBuffer::new_marked(5, 3);
        t.unmark_all();
        t.mark(99);
        assert!(t.is_clean());
        t.mark(10..20);
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
        assert_eq!(t.count_marked(), 3);
    }

    #[test]
    fn get_mut_marks_just_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        t.get_mut(Row(1)).unwrap()[0] = Cell::inline('x');
        assert!(!t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(!t.is_marked(2));
    }
    #[test]
    fn index_mut_marks_just_its_row() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        t[Row(1)][0] = Cell::inline('x');
        assert!(!t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(!t.is_marked(2));
    }

    #[test]
    fn try_mark_returns_out_of_bounds_error() {
        let mut t = TrackingBuffer::new(5, 3);
        t.unmark_all();
        assert!(t.try_mark(Row(99)).is_err());
        assert!(t.is_clean());
    }

    #[test]
    fn buffer_mut_marks_all_dirty() {
        let mut t = TrackingBuffer::new(5, 3);
        t.mark_all();
        assert_eq!(t.count_marked(), 3);
    }

    #[test]
    fn resize_updates_bitmap_and_marks_all_dirty() {
        let mut t = TrackingBuffer::new_unmarked(5, 3);
        assert!(t.is_clean());
        t.resize(7, 5);
        assert_eq!(t.count_marked(), 5);
        assert!(t.is_dirty());
        assert_eq!(t.count_marked(), 5);
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
        let mut t = TrackingBuffer::new_unmarked(2, 1);
        t.push_row_marked([Cell::inline('a'), Cell::inline('b')]);
        dbg!(t.count_marked());
        assert!(t.is_marked(1));
        let _ = t.pop_row();
        assert_eq!(t.count_marked(), 0);
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

        t.mark(1);
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
    fn insert_cell_with_large_n_still_marks_only_its_row() {
        // Regression: the old heuristic marked `row..row + n / width` for
        // `n > width`, over-marking rows the buffer never touched. `insert_cell`
        // clamps `n` to the row's width, so only `row` ever changes.
        let mut t = TrackingBuffer::new(4, 3);
        t.unmark_all();
        t.insert_cell(1, 0, 99, Cell::EMPTY);
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
        let dirty: Vec<_> = t.iter_marked().collect();
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

    #[test]
    fn from_buffer_unmarked_preserves_contents() {
        // Regression: this used to discard the buffer and allocate a fresh,
        // empty one of the same size.
        let mut arena = Arena::new();
        let buf = Buffer::from_lines(["abc"], &mut arena);
        let t = TrackingBuffer::from_buffer_unmarked(buf.clone());
        assert!(t.is_clean());
        assert_eq!(t.as_inner(), &buf);
    }

    #[test]
    fn mark_range_to_only_marks_below_end() {
        // Regression: `RangeTo::try_mark` ignored its endpoint and marked the
        // whole buffer.
        let mut t = TrackingBuffer::new_unmarked(2, 5);
        t.mark(..3);
        assert!(t.is_marked(0));
        assert!(t.is_marked(1));
        assert!(t.is_marked(2));
        assert!(!t.is_marked(3));
        assert!(!t.is_marked(4));
        assert_eq!(t.count_marked(), 3);
    }

    #[test]
    fn mark_exclusive_range_up_to_height_is_in_bounds() {
        // Regression: `0..height` is the natural "mark all rows" for an
        // exclusive range and must not be rejected as out of bounds.
        let mut t = TrackingBuffer::new_unmarked(2, 4);
        assert!(t.try_mark(0..4).is_ok());
        assert_eq!(t.count_marked(), 4);
        // One past the end is still out of bounds.
        let mut t = TrackingBuffer::new_unmarked(2, 4);
        assert!(t.try_mark(0..5).is_err());
        assert!(t.is_clean());
    }

    #[test]
    fn remove_row_unmarked_shifts_markers_up() {
        let mut t = TrackingBuffer::new_unmarked(2, 4);
        // Mark rows 0 and 3.
        t.mark(0);
        t.mark(3);
        // Remove row 1: row 3 shifts up to index 2, row 0 stays put.
        t.remove_row_unmarked(1);
        assert_eq!(t.height(), 3);
        assert!(t.is_marked(0));
        assert!(!t.is_marked(1));
        assert!(t.is_marked(2));
    }

    #[test]
    fn insert_row_shifts_markers_down_and_leaves_new_row_unmarked() {
        let mut t = TrackingBuffer::new_unmarked(2, 3);
        t.mark(0);
        t.mark(2);
        // Insert at index 1: the new row is unmarked, old rows 1 and 2 shift to
        // 2 and 3.
        t.insert_row(1, [Cell::EMPTY, Cell::EMPTY]);
        assert_eq!(t.height(), 4);
        assert!(t.is_marked(0));
        assert!(!t.is_marked(1));
        assert!(!t.is_marked(2));
        assert!(t.is_marked(3));
    }

    #[test]
    fn pop_row_drops_only_the_last_marker() {
        let mut t = TrackingBuffer::new_unmarked(2, 3);
        t.mark(0);
        t.mark(2);
        t.pop_row();
        assert_eq!(t.height(), 2);
        assert!(t.is_marked(0));
        assert!(!t.is_marked(1));
        assert_eq!(t.count_marked(), 1);
    }
}
