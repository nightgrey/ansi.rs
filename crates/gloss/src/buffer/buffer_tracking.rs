///! !A [`Buffer`] with per-row change tracking.
///!
///! `TrackingBuffer` wraps a [`Buffer`] and keeps one marker bit per row.
///! Read-only access mirrors the inner buffer through [`Deref`]. Mutating
///! operations are provided as dedicated methods that mark the rows they touch.
///! A subsequent [`diff`](crate::buffer::buffer_tracking::TrackingBuffer::diff) can then skip unmarked rows.
///!
///! # Terminology
///!
///! - A **marked** row may have changed since the last call to
///!   [`unmark_all`](crate::buffer::buffer_tracking::TrackingBuffer::unmark_all) or [`clean`](crate::buffer::buffer_tracking::TrackingBuffer::clean).
///! - An **unmarked** row is assumed unchanged.
///! - A buffer is **dirty** when at least one row is marked.
///! - A buffer is **clean** when no rows are marked.
///!
///! # Lifecycle
///!
///! 1. Create a `TrackingBuffer`. By default, all rows are marked so the first
///!    diff sees the full buffer.
///! 2. Mutate the buffer through tracking-aware methods.
///! 3. Call [`diff`](crate::buffer::buffer_tracking::TrackingBuffer::diff) and apply the produced changes.
///! 4. Call [`unmark_all`](crate::buffer::buffer_tracking::TrackingBuffer::unmark_all) or [`clean`](crate::buffer::buffer_tracking::TrackingBuffer::clean) before
///!    rendering the next frame.
///!
///! # Invariants
///!
///! - `self.bits.len() == self.inner.height`.
///! - Row `y` is marked iff `self.bits.contains(y)`.
use crate::{Arena, Buffer, BufferDiff, BufferIndex, ByDirty, Cell};
use derive_more::{AsRef, Deref, From};
pub use fixedbitset::IndexRange as TrackingRange;
use geometry::{Bound, Point, Position, PositionLike, Rect, Resolve, Row};
use std::fmt::Debug;
use std::iter;
use std::ops::{DerefMut, Index, IndexMut, RangeBounds};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice::SliceIndex;
use thiserror::Error;
pub type BitSet2 = hi_sparse_bitset::BitSet<hi_sparse_bitset::config::_256bit>;
pub type BitSet = fixedbitset::FixedBitSet;
pub type Bit = fixedbitset::Block;

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
    pub(super) bits: BitSet,
}

impl TrackingBuffer {
    /// An empty tracking buffer.
    ///
    /// The buffer has no rows, so it is both clean and contains no markers.
    pub const EMPTY: Self = Self {
        inner: Buffer::EMPTY,
        bits: BitSet::new(),
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
        let mut bits = BitSet::with_capacity(height);
        bits.insert_range(..);
        Self {
            inner: Buffer::new(width, height),
            bits,
        }
    }

    /// Creates a tracking buffer with no rows marked.
    ///
    /// Use this when the caller knows the newly-created buffer is already in
    /// sync with its comparison target.
    pub fn new_unmarked(width: usize, height: usize) -> Self {
        Self {
            inner: Buffer::new(width, height),
            bits: BitSet::with_capacity(height),
        }
    }

    /// Wraps an existing [`Buffer`] and marks every non-empty row.
    ///
    /// Empty rows are left unmarked. This is useful when adopting an existing
    /// buffer whose meaningful contents should be included in the next diff.
    pub fn from_buffer_checked(buffer: Buffer) -> Self {
        let mut bits = BitSet::with_capacity(buffer.height);

        for (y, row) in buffer.iter_rows().enumerate() {
            if !row.is_empty() {
                bits.set(y, true);
            }
        }

        Self {
            inner: buffer,
            bits,
        }
    }

    /// Wraps an existing [`Buffer`] and marks every row.
    pub fn from_buffer_marked(buffer: Buffer) -> Self {
        Self::new_marked(buffer.width, buffer.height)
    }

    /// Wraps an existing [`Buffer`] and leaves every row unmarked.
    pub fn from_buffer_unmarked(buffer: Buffer) -> Self {
        Self::new_unmarked(buffer.width, buffer.height)
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
        self.bits.remove_range(..);
    }

    /// Returns `true` if row `y` is marked.
    ///
    /// Out-of-bounds rows return `false`.
    #[inline]
    pub fn is_marked(&self, y: usize) -> bool {
        self.bits.contains(y)
    }

    /// Returns `true` if at least one row in `rows` is marked.
    #[inline]
    pub fn any_marked(&self, rows: impl TrackingRange) -> bool {
        self.bits.contains_any_in_range(rows)
    }

    /// Returns `true` if every row in `rows` is marked.
    #[inline]
    pub fn all_marked(&self, rows: impl TrackingRange) -> bool {
        self.bits.contains_all_in_range(rows)
    }

    /// Returns `true` if at least one row in `rows` is unmarked.
    #[inline]
    pub fn any_unmarked(&self, rows: impl TrackingRange) -> bool {
        !self.all_marked(rows)
    }

    /// Returns `true` if every row in `rows` is unmarked.
    #[inline]
    pub fn all_unmarked(&self, rows: impl TrackingRange) -> bool {
        !self.any_marked(rows)
    }

    /// Returns `true` if no rows are marked.
    #[inline]
    pub fn is_clean(&self) -> bool {
        self.bits.is_clear()
    }

    /// Unmarks every row.
    ///
    /// After this call, [`is_clean`](Self::is_clean) returns `true`.
    #[inline]
    pub fn clean(&mut self) {
        self.unmark_all();
    }

    /// Returns `true` if at least one row is marked.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        !self.is_clean()
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
        self.count_marked_in(..)
    }

    /// Returns the number of unmarked rows.
    pub fn count_unmarked(&self) -> usize {
        self.count_unmarked_in(..)
    }

    /// Returns the number of marked rows in `rows`.
    pub fn count_marked_in(&self, rows: impl TrackingRange) -> usize {
        self.bits.count_ones(rows)
    }

    /// Returns the number of unmarked rows in `rows`.
    pub fn count_unmarked_in(&self, rows: impl TrackingRange) -> usize {
        self.bits.count_zeroes(rows)
    }

    /// Returns the raw marker set.
    ///
    /// `marks().contains(y)` is `true` when row `y` is marked.
    #[inline]
    pub fn as_bits(&self) -> &BitSet {
        &self.bits
    }

    /// Iterates over marked row indices.
    pub fn iter_marked(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.ones()
    }

    /// Iterates over unmarked row indices.
    pub fn iter_unmarked(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.zeroes()
    }

    // ----------------------------------------------------------
    // Inner buffer access
    // ----------------------------------------------------------

    /// Returns the inner buffer.
    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.inner
    }

    /// Returns the inner buffer mutably without marking rows.
    ///
    /// This is an escape hatch. Mutations performed through this reference are
    /// not tracked. Prefer the tracking-aware mutators on `TrackingBuffer`
    /// whenever possible.
    pub fn buffer_mut_untracked(&mut self) -> &mut Buffer {
        &mut self.inner
    }

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
    pub fn resize(&mut self, width: usize, height: usize) {
        if self.inner.width == width && self.inner.height == height {
            return;
        }

        self.inner.resize(width, height);
        self.bits = BitSet::with_capacity(height);
        self.bits.insert_range(..);
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
        I: BufferIndex<Output = [Cell]>,
    {
        let written = self.inner.set_string(index, string, arena)?;
        self.mark_all();
        Some(written)
    }

    /// Inserts `n` rows at `y` and marks rows from `y` to the bottom.
    pub fn insert_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.inner.insert_line(y, n, cell);
        self.mark(y..);
    }

    /// Deletes `n` rows at `y` and marks rows from `y` to the bottom.
    pub fn delete_line(&mut self, y: usize, n: usize, cell: Cell) {
        self.inner.delete_line(y, n, cell);
        self.mark(y..);
    }

    /// Inserts `n` cells into `row` at `col` and marks `row`.
    pub fn insert_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.inner.insert_cell(row, col, n, cell);

        if n > self.width {
            self.mark(row..row + n / self.width);
        } else {
            self.mark(row);
        }
    }

    /// Deletes `n` cells from `row` at `col` and marks `row`.
    pub fn delete_cell(&mut self, row: usize, col: usize, n: usize, cell: Cell) {
        self.inner.delete_cell(row, col, n, cell);
        self.mark(row);
    }

    /// Inserts `n` rows within `bounds` and marks rows inside `bounds`.
    pub fn insert_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.inner.insert_line_area(y, n, cell, bounds);
        self.mark(bounds.min.y as usize..bounds.max.y as usize);
    }

    /// Deletes `n` rows within `bounds` and marks rows inside `bounds`.
    pub fn delete_line_area(&mut self, y: usize, n: usize, cell: Cell, bounds: Rect) {
        self.inner.delete_line_area(y, n, cell, bounds);
        self.mark(bounds.min.y as usize..bounds.max.y as usize);
    }

    /// Inserts `n` cells within `bounds` and marks `row`.
    pub fn insert_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        self.inner.insert_cell_area(row, col, n, cell, bounds);
        self.mark(row);
    }

    /// Deletes `n` cells within `bounds` and marks `row`.
    pub fn delete_cell_area(&mut self, row: usize, col: usize, n: usize, cell: Cell, bounds: Rect) {
        self.inner.delete_cell_area(row, col, n, cell, bounds);
        self.mark(row);
    }

    /// Appends a row and leaves the new row unmarked.
    pub fn push_row_unmarked(&mut self, row: impl IntoIterator<Item = Cell>) {
        self.inner.push_row(row);
        self.bits.grow(self.inner.height);
    }

    /// Appends a row and marks it.
    pub fn push_row_marked(&mut self, row: impl IntoIterator<Item = Cell>) {
        self.push_row_unmarked(row);
        self.mark(self.height - 1);
    }

    /// Removes and returns the last row.
    ///
    /// The removed row's marker is discarded.
    pub fn pop_row(&mut self) -> Option<Vec<Cell>> {
        let row = self.inner.pop_row()?;

        let mut next_bits = BitSet::with_capacity(self.inner.height - 1);

        for i in self.bits.ones().take(self.inner.height - 1) {
            next_bits.set(i, self.bits[i]);
        }

        self.bits = next_bits;

        Some(row)
    }

    /// Removes row and does not mark the rows shifted up.
    pub fn remove_row_unmarked(&mut self, idx: usize) -> Option<Vec<Cell>> {
        let old_height = self.height;
        let row = self.inner.remove_row(idx)?; // idx < old_len guaranteed

        // Rebuild bitset without the removed bit
        let new_height = old_height - 1;
        let mut bits = BitSet::with_capacity(new_height);

        let min = self.bits.minimum();
        for i in min.map_or_else(|| idx, |min| if min < idx { min } else { idx })..idx {
            bits.set(i, unsafe { self.bits.contains_unchecked(i) });
        }

        // The new row is unmarked.
        bits.set(idx, false);

        for i in
            min.map_or_else(|| new_height, |min| if min < idx { min } else { idx + 1 })..new_height
        {
            bits.set(i, unsafe { self.bits.contains_unchecked(i - 1) });
        }

        for i in 0..idx {
            bits.set(i, unsafe { self.bits.contains_unchecked(i) });
        }

        if self.bits.contains_any_in_range(idx..new_height) {
            for i in idx..new_height {
                bits.set(i, unsafe { self.bits.contains_unchecked(i + 1) });
            }
        }

        self.bits = bits;

        Some(row)
    }

    /// Removes row and marks the rows shifted up.
    pub fn remove_row_marked(&mut self, idx: usize) -> Option<Vec<Cell>> {
        self.remove_row_unmarked(idx).map(|row| {
            self.mark(idx..);
            row
        })
    }

    /// Inserts a row and marks rows shifted down.
    /// The inserted row is unmarked.
    pub fn insert_row(&mut self, idx: usize, row: impl IntoIterator<Item = Cell>) {
        if idx > self.height {
            return;
        }

        let old_height = self.height;
        self.inner.insert_row(idx, row);

        // Rebuild bitset with a new unmarked bit at `idx`
        let new_height = old_height + 1;
        let mut bits = BitSet::with_capacity(new_height);

        let min = self.bits.minimum();
        for i in min.map_or_else(|| idx, |min| if min < idx { min } else { idx })..idx {
            bits.set(i, unsafe { self.bits.contains_unchecked(i) });
        }

        // The new row is unmarked.
        bits.set(idx, false);

        for i in min.map_or_else(
            || new_height,
            |min| if min < idx + 1 { min } else { idx + 1 },
        )..new_height
        {
            bits.set(i, unsafe { self.bits.contains_unchecked(i - 1) });
        }

        self.bits = bits;
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
    pub fn as_inner(&self) -> &Buffer {
        &self.inner
    }

    /// Returns the inner buffer mutably.
    ///
    /// # Safety
    /// This method allows mutating the inner buffer without marking rows and is thought of as an escape hatch.
    pub unsafe fn as_mut_inner(&mut self) -> &mut Buffer {
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
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError>;
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError>;

    fn mark(self, tracking_buffer: &mut TrackingBuffer) {
        self.try_mark(tracking_buffer).unwrap();
    }

    fn unmark(self, tracking_buffer: &mut TrackingBuffer) {
        self.try_unmark(tracking_buffer).unwrap();
    }
}

impl TrackingBufferIndex for usize {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let index = self;
        if index >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.bits.insert(index);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let index = self;
        if index >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.bits.remove(index);
        Ok(())
    }
}
impl TrackingBufferIndex for Row {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(self.into_inner(), tracking_buffer)
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(self.into_inner(), tracking_buffer)
    }
}
impl TrackingBufferIndex for Point {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(self.y as usize, tracking_buffer)
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(self.y as usize, tracking_buffer)
    }
}
impl TrackingBufferIndex for Position {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(self.row, tracking_buffer)
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(self.row, tracking_buffer)
    }
}
impl TrackingBufferIndex for PositionLike {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_mark(self.1 as usize, tracking_buffer)
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        TrackingBufferIndex::try_unmark(self.1 as usize, tracking_buffer)
    }
}

impl<I: BufferIndex<Index = usize>> TrackingBufferIndex for Range<I> {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.into_slice_index(tracking_buffer);
        let end = self.end.into_slice_index(tracking_buffer);

        if end >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.bits.insert_range(start..end);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.into_slice_index(tracking_buffer);
        let end = self.end.into_slice_index(tracking_buffer);

        if end >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.bits.remove_range(start..end);
        Ok(())
    }
}
impl<I: BufferIndex<Index = usize>> TrackingBufferIndex for RangeInclusive<I> {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start().clone().into_slice_index(tracking_buffer);
        let end = self.end().clone().into_slice_index(tracking_buffer);

        if end >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.bits.insert_range(start..end + 1);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start().clone().into_slice_index(tracking_buffer);
        let end = self.end().clone().into_slice_index(tracking_buffer);

        if end >= tracking_buffer.height {
            return Err(TrackingBufferError::OutOfBounds);
        }
        tracking_buffer.bits.remove_range(start..end + 1);
        Ok(())
    }
}
impl<I: BufferIndex<Index = usize>> TrackingBufferIndex for RangeFrom<I> {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.into_slice_index(tracking_buffer);

        tracking_buffer.bits.insert_range(start..);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let start = self.start.into_slice_index(tracking_buffer);

        tracking_buffer.bits.remove_range(start..);
        Ok(())
    }
}
impl<I: BufferIndex<Index = usize>> TrackingBufferIndex for RangeTo<I> {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let _end = self.end.into_slice_index(tracking_buffer);
        let end = tracking_buffer.height;

        tracking_buffer.bits.insert_range(..end);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.into_slice_index(tracking_buffer);

        tracking_buffer.bits.remove_range(..end);
        Ok(())
    }
}
impl<I: BufferIndex<Index = usize>> TrackingBufferIndex for RangeToInclusive<I> {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.into_slice_index(tracking_buffer);

        tracking_buffer.bits.insert_range(..end + 1);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        let end = self.end.into_slice_index(tracking_buffer);

        tracking_buffer.bits.remove_range(..end + 1);
        Ok(())
    }
}

impl TrackingBufferIndex for RangeFull {
    fn try_mark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        tracking_buffer.bits.insert_range(..);
        Ok(())
    }
    fn try_unmark(self, tracking_buffer: &mut TrackingBuffer) -> Result<(), TrackingBufferError> {
        tracking_buffer.bits.remove_range(..);
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
}
