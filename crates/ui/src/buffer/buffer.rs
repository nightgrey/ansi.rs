//! The terminal framebuffer — a styled 2D cell grid.
//!
//! [`Buffer`] is the central data structure of the rendering pipeline. It
//! stores a flat, row-major `Vec<Cell>` together with its grid dimensions
//! (`width` × `height`), and derefs to `&[Cell]` / `&mut [Cell]` for seamless
//! slice-based access.
//!
//! # Indexing
//!
//! [`Buffer`] implements `Index<I>` and `IndexMut<I>` for every `I:
//! BufferIndex`, so you can write `buf[(x, y)]` or `buf[0..5]` as if it were
//! a native array. Geometry-aware queries (`x`, `y`, `contains`, …) are
//! available through [`BufferIndexExt`] methods dispatched on the index
//! value — e.g. `buf.index_of((3, 7))` yields the flat `usize` offset.
//!
//! # Text writing
//!
//! [`set_string`](Self::set_string) and [`set_line`](Self::set_line) accept any
//! index type, delegate grapheme measurement and writing to [`Cells::write`],
//! and handle clipped strings gracefully.
//!
//! # Resize
//!
//! [`resize`](Self::resize) reallocates in-place when the dimensions change,
//! preserving existing content where the old and new grids overlap. Cells that
//! fall outside the new bounds are dropped; new cells are initialised to
//! [`Cell::EMPTY`].

use super::{BufferIndex, BufferIndexExt, BufferIndexMany, Cell, Cells, Graphemes};
use ansi::Style;
use derive_more::{Deref, DerefMut, IntoIterator};
use geometry::{Bounded, Point};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops;
use unicode_width::UnicodeWidthStr;

#[derive(Deref, DerefMut, IntoIterator, Clone, PartialEq)]
/// A styled 2D grid of terminal cells — the framebuffer.
///
/// Stored as a flat, row-major `Vec<Cell>` with explicit `width` and `height`.
/// Derefs to `&[Cell]` / `&mut [Cell]` so all slice operations work directly.
///
/// # Example
///
/// ```ignore
/// use ui::buffer::{Buffer, Cell, Cells, Graphemes};
/// let mut arena = Graphemes::new();
/// let mut buf = Buffer::new(10, 3);
/// buf[(0, 0)] = Cell::new('>').with_foreground(Color::Green);
/// buf.set_string(1.., "Hello!", None, &mut arena);
/// ```
pub struct Buffer {
    #[deref(forward)]
    #[deref_mut(forward)]
    #[into_iterator(owned, ref, ref_mut)]
    pub(crate) inner: Vec<Cell>,
    width: u16,
    height: u16,
}

impl Buffer {
    pub const EMPTY: Self = Self {
        inner: Vec::new(),
        width: 0,
        height: 0,
    };

    /// Create a buffer filled with [`Cell::EMPTY`] cells.
    ///
    /// The backing storage is a flat `Vec<Cell>` of length `width × height`,
    /// laid out row-major. Every cell starts as a blank space with default
    /// style.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut buf = Buffer::new(80, 24);
    /// assert_eq!(buf.width(), 80);
    /// assert_eq!(buf.height(), 24);
    /// assert_eq!(buf.len(), 80 * 24);
    /// ```
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            inner: vec![Cell::EMPTY; (width * height) as usize],
            width,
            height,
        }
    }

    /// Create a buffer from a slice of `(index, char, style)` tuples.
    ///
    /// Each tuple places a styled character at the given index. Indices can
    /// be any [`BufferIndexExt`] type — points set a single cell, ranges fill
    /// a region.
    ///
    /// Mostly used for constructing expected buffers in tests.
    pub fn from_elements<I: BufferIndexExt>(
        width: u16,
        height: u16,
        elements: &[(I, char, Style)],
    ) -> Self {
        let mut buffer = Self::new(width, height);
        for (index, ch, style) in elements {
            for cell in buffer.iter_index_mut(index.clone()) {
                *cell = Cell::new(*ch).with_style(*style);
            }
        }
        buffer
    }

    /// Create a buffer from an iterator of string slices, one per row.
    ///
    /// The buffer's width is the display width of the longest line;
    /// its height is the number of lines. Shorter lines are padded with
    /// empty cells. Every line is written via [`set_line`](Self::set_line),
    /// so wide characters and style handling go through the normal path.
    ///
    /// Empty input produces a zero-sized buffer.
    #[must_use]
    pub fn from_lines<'a>(lines: impl IntoIterator<Item = &'a str>, arena: &mut Graphemes) -> Self {
        let lines = lines.into_iter().collect::<Vec<_>>();
        let height = lines.len();
        let width = lines
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or_default();
        let mut buffer = Self::new(width as u16, height as u16);
        for (y, line) in lines.iter().enumerate() {
            buffer.set_line((0, y as u16), line, None, arena);
        }
        buffer
    }

    /// Creates a [`Buffer`] by calling `f` for each cell position.
    ///
    /// # Example
    /// ```ignore
    /// let buf = Buffer::from_fn(10, 5, |row, col| Cell::inline(char::from(b'A' + (row * 10 + col) as u8 % 26)));
    /// ```
    pub fn from_fn(width: u16, height: u16, f: impl Fn(u16, u16) -> Cell) -> Self {
        let mut inner = Vec::with_capacity((width * height) as usize);
        for row in 0..height {
            for col in 0..width {
                inner.push(f(row, col));
            }
        }
        Self {
            inner,
            width,
            height,
        }
    }

    /// The width of the buffer in terminal columns.
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// The height of the buffer in terminal rows.
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    #[inline]
    pub fn get<I: BufferIndex>(&self, index: I) -> Option<&I::Output> {
        index.get(self)
    }

    /// Returns a slice of shared references to the output at this location, if in
    /// bounds.
    #[inline]
    pub fn get_many<I: BufferIndexMany>(&self, index: I) -> Option<&[Cell]> {
        index.get_many(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    #[inline]
    pub fn get_mut<I: BufferIndex>(&mut self, index: I) -> Option<&mut I::Output> {
        index.get_mut(self)
    }

    /// Returns a slice of mutable references to the output at this location, if in
    /// bounds.
    #[inline]
    pub fn get_many_mut<I: BufferIndexMany>(&mut self, index: I) -> Option<&mut [Cell]> {
        index.get_many_mut(self)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn get_unchecked<I: BufferIndex>(&self, index: I) -> *const I::Output {
        unsafe { index.get_unchecked(self) }
    }

    /// Returns a slice of pointers to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn get_many_unchecked<I: BufferIndexMany>(&self, index: I) -> *const [Cell] {
        index.get_many_unchecked(self)
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn get_unchecked_mut<I: BufferIndex>(&mut self, index: I) -> *mut I::Output {
        unsafe { index.get_unchecked_mut(self) }
    }

    /// Returns a slice of mutable pointers to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn get_many_unchecked_mut<I: BufferIndexMany>(&mut self, index: I) -> *mut [Cell] {
        index.get_many_unchecked_mut(self)
    }

    /// Returns `true` if `index` lies within the buffer's bounds.
    ///
    /// Accepts any index type — points, rows, ranges, and raw `usize`
    /// offsets — and delegates to [`BufferIndexExt::within`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// let buf = Buffer::new(10, 5);
    /// assert!(buf.contains((5, 3)));          // valid point
    /// assert!(!buf.contains((15, 3)));         // x out of bounds
    /// assert!(buf.contains(0usize..50usize));  // valid range
    /// ```
    #[inline]
    pub fn contains(&self, index: impl BufferIndexExt) -> bool {
        index.within(self)
    }

    /// Convert `index` to a flat `usize` offset into the backing `Vec<Cell>`.
    ///
    /// This is the `y * width + x` calculation for point-like indices and the
    /// start offset for range-like indices. Does **not** perform bounds
    /// checking — use [`contains`](Self::contains) or
    /// [`within`](BufferIndexExt::within) first if needed.
    #[inline]
    pub fn index_of(&self, index: impl BufferIndexExt) -> usize {
        index.as_index(self)
    }

    /// Convert `index` to a [`Point`] representing the first cell covered.
    ///
    /// For range indices this returns the `(x, y)` of the range's start.
    #[inline]
    pub fn point_of(&self, index: impl BufferIndexExt) -> Point {
        index.as_point(self)
    }

    /// The column (`x`) of the first cell covered by `index`.
    #[inline]
    pub fn x_of(&self, index: impl BufferIndexExt) -> u16 {
        index.x(self)
    }

    /// The row (`y`) of the first cell covered by `index`.
    #[inline]
    pub fn y_of(&self, index: impl BufferIndexExt) -> u16 {
        index.y(self)
    }

    /// Convert `index` to a `Range<usize>` in flat storage.
    #[inline]
    pub fn range_of(&self, index: impl BufferIndexExt) -> ops::Range<usize> {
        index.as_range(self)
    }

    /// The number of cells covered by `index`.
    ///
    /// Returns `1` for point-like indices and the slice length for range
    /// indices.
    #[inline]
    pub fn len_of(&self, index: impl BufferIndexExt) -> usize {
        index.len(self)
    }

    /// The flat offset of the first cell covered by `index`.
    #[inline]
    pub fn start_of(&self, index: impl BufferIndexExt) -> usize {
        index.start(self)
    }

    /// The flat offset just past the last cell covered by `index`.
    #[inline]
    pub fn end_of(&self, index: impl BufferIndexExt) -> usize {
        index.end(self)
    }

    /// Print the given string until the end of the given index.
    pub fn set_string(
        &mut self,
        index: impl BufferIndexExt,
        string: impl AsRef<str>,
        style: Option<Style>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        Some(Cells::write(
            self.get_many_mut(index)?,
            string.as_ref(),
            style,
            arena,
        ))
    }

    /// Write `string` from `index` to the end of its row.
    ///
    /// Clamps the write to the row's right edge — the string never wraps to
    /// the next line. This is a convenience wrapper around
    /// [`set_string`](Self::set_string) that computes the row-extent range
    /// from `index`'s `(x, y)` to `(width, y)`.
    ///
    /// Returns `None` if `index` is out of bounds; otherwise the number of
    /// columns written (see [`Cells::write`]).
    pub fn set_line(
        &mut self,
        index: impl BufferIndexExt<Output = Cell>,
        string: impl AsRef<str>,
        style: Option<Style>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        let point = self.point_of(index);

        self.set_string(
            point..Point::new(self.width(), point.y),
            string,
            style,
            arena,
        )
    }

    /// Resize the buffer to new dimensions, preserving existing content.
    ///
    /// Cells that fall within the overlap of the old and new grids keep
    /// their values. New cells (from widening or growing taller) are
    /// initialised to [`Cell::EMPTY`]. Cells that fall outside the new
    /// bounds are dropped.
    ///
    /// When only the height changes, rows are appended or truncated in
    /// place. When the width changes, each row is shifted to its new offset
    /// and new columns are filled with [`Cell::EMPTY`].
    ///
    /// This is O(n) in the total number of cells. It does **not** interact
    /// with the grapheme arena — release arena storage separately if needed.
    pub fn resize(&mut self, width: u16, height: u16) {
        let (current_width, current_height) = (self.width as usize, self.height as usize);
        let (width, height) = (width as usize, height as usize);
        if current_width == width && current_height == height {
            return;
        }

        if width != current_width {
            let copy_w = width.min(current_width);

            if width > current_width {
                // Growing: extend first, then shift rows back-to-front
                self.inner.resize(width * current_height, Cell::EMPTY);
                for y in (1..current_height).rev() {
                    let src = y * current_width;
                    let dst = y * width;
                    self.inner.copy_within(src..src + copy_w, dst);
                    // Fill the new columns
                    self.inner[dst + copy_w..dst + width].fill(Cell::EMPTY);
                }
                // Row 0: just fill the tail
                self.inner[copy_w..width].fill(Cell::EMPTY);
            } else {
                // Shrinking: shift rows front-to-back, then truncate
                for y in 1..current_height {
                    let src = y * current_width;
                    let dst = y * width;
                    self.inner.copy_within(src..src + copy_w, dst);
                }
                self.inner.truncate(width * current_height);
            }

            self.width = width as u16;
        }

        if height > current_height {
            self.inner.resize(width * height, Cell::EMPTY);
        } else if height < current_height {
            self.inner.truncate(width * height);
        }
        self.height = height as u16;
    }

    /// Iterate over the cells covered by `index`.
    ///
    /// Out-of-bounds indices produce an empty iterator — no panic, no `None`
    /// wrapper. This is the ergonomic alternative to `get`/`get_mut` for
    /// loops that should silently skip invalid ranges.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for cell in buf.iter_index(Row(3)) {
    ///     // operates on row 3, does nothing if row 3 is out of bounds
    /// }
    /// ```
    #[inline]
    pub fn iter_index(&self, index: impl BufferIndexExt) -> impl Iterator<Item = &Cell> {
        index.iter(self)
    }

    /// Iterate mutably over the cells covered by `index`.
    ///
    /// See [`iter_index`](Self::iter_index) for the borrowing equivalent.
    #[inline]
    pub fn iter_index_mut(
        &mut self,
        index: impl BufferIndexExt,
    ) -> impl Iterator<Item = &mut Cell> {
        index.iter_mut(self)
    }

    /// Iterate over each row, yielding an iterator over the row's cells.
    ///
    /// The outer iterator yields one inner iterator per row; each inner
    /// iterator walks the cells of that row left-to-right. Uses
    /// [`chunks_exact`](slice::chunks_exact) for zero-overhead row slicing.
    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        self.inner
            .chunks_exact(self.width as usize)
            .map(|row| row.iter())
    }

    /// Iterate over each column, yielding an iterator over the column's cells.
    ///
    /// The outer iterator yields one inner iterator per column; each inner
    /// iterator walks the cells of that column top-to-bottom. Uses
    /// [`step_by`](Iterator::step_by) on a slice window, so column access is
    /// strided rather than contiguous.
    pub fn cols(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        let width = self.width as usize;
        (0..width).map(move |col| self[col..].iter().step_by(width))
    }

    /// Reset every cell in the buffer to [`Cell::EMPTY`].
    ///
    /// This is O(n) in the number of cells. It does **not** release the
    /// allocation — capacity is preserved for reuse. If you also need to
    /// reclaim grapheme arena storage, call [`Graphemes::clear`] separately.
    pub fn clear(&mut self) {
        self.inner.fill(Cell::EMPTY);
    }
}

const impl AsRef<[Cell]> for Buffer {
    fn as_ref(&self) -> &[Cell] {
        &self.inner
    }
}
const impl AsMut<[Cell]> for Buffer {
    fn as_mut(&mut self) -> &mut [Cell] {
        &mut self.inner
    }
}
const impl Borrow<[Cell]> for Buffer {
    fn borrow(&self) -> &[Cell] {
        self.as_ref()
    }
}
const impl BorrowMut<[Cell]> for Buffer {
    fn borrow_mut(&mut self) -> &mut [Cell] {
        self.as_mut()
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if self.width > 0 {
            if f.alternate() {
                writeln!(f)?;
                /*
                    WARNING

                    Compound types becoming enormous as the entire `fmt::Debug` width is applied to each item individually.
                    For tuples and structs define padding and precision arguments manually to improve readability.
                */
                let width = f.width().unwrap_or_else(|| {
                    // Conditionally calculate the longest item by default.
                    self.iter()
                        .map(|i| format!("{i:?}").len())
                        .max()
                        .unwrap_or(0)
                });
                let precision = f.precision().unwrap_or(2);
                for mut row in self.rows().map(Iterator::peekable) {
                    write!(f, "    [")?;
                    while let Some(item) = row.next() {
                        write!(f, " {item:width$.precision$?}")?;
                        if row.peek().is_some() {
                            write!(f, ",")?;
                        }
                    }
                    writeln!(f, "]")?;
                }
            } else {
                for row in self.rows() {
                    f.debug_list().entries(row).finish()?;
                }
            }
        }
        write!(f, "]")
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.height == 0 || self.width == 0 {
            return Ok(());
        }

        for y in 0..self.height {
            if y > 0 {
                f.write_char('\n')?;
            }

            let start = y * self.width;
            let end = start + self.width;

            for cell in &self.inner[(start as usize)..(end as usize)] {
                write!(f, "{:?}", cell)?;
            }
        }
        Ok(())
    }
}

impl Bounded for Buffer {
    fn min_x(&self) -> u16 {
        0
    }

    fn min_y(&self) -> u16 {
        0
    }

    fn max_x(&self) -> u16 {
        self.width
    }

    fn max_y(&self) -> u16 {
        self.height
    }

    fn min(&self) -> Point {
        Point::ZERO
    }

    fn max(&self) -> Point {
        Point::new(self.max_x(), self.max_y())
    }
}
