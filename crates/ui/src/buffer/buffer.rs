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

use super::{Cell, Cells, Graphemes, BufferIndex, BufferIndexExt, BufferIndexMany};
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

    pub fn new(width: u16, height: u16) -> Self {
        Self {
            inner: vec![Cell::EMPTY; (width * height) as usize],
            width,
            height,
        }
    }

    /// Create a buffer from a slice of fixed elements.
    ///
    /// A convenience constructor mostly used for tests.
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

    /// Create a buffer from a slice of fixed elements.
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

    pub fn width(&self) -> u16 {
        self.width
    }

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

    #[inline]
    pub fn contains(&self, index: impl BufferIndexExt) -> bool {
        index.within(self)
    }

    #[inline]
    pub fn index_of(&self, index: impl BufferIndexExt) -> usize {
        index.as_index(self)
    }

    #[inline]
    pub fn point_of(&self, index: impl BufferIndexExt) -> Point {
        index.as_point(self)
    }

    #[inline]
    pub fn x_of(&self, index: impl BufferIndexExt) -> u16 {
        index.x(self)
    }

    #[inline]
    pub fn y_of(&self, index: impl BufferIndexExt) -> u16 {
        index.y(self)
    }

    #[inline]
    pub fn range_of(&self, index: impl BufferIndexExt) -> ops::Range<usize> {
        index.as_range(self)
    }

    #[inline]
    pub fn len_of(&self, index: impl BufferIndexExt) -> usize {
        index.len(self)
    }

    #[inline]
    pub fn start_of(&self, index: impl BufferIndexExt) -> usize {
        index.start(self)
    }

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

    #[inline]
    pub fn iter_index(&self, index: impl BufferIndexExt) -> impl Iterator<Item = &Cell> {
        index.iter(self)
    }

    #[inline]
    pub fn iter_index_mut(
        &mut self,
        index: impl BufferIndexExt,
    ) -> impl Iterator<Item = &mut Cell> {
        index.iter_mut(self)
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        self.inner
            .chunks_exact(self.width as usize)
            .map(|row| row.iter())
    }

    pub fn cols(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        let width = self.width as usize;
        (0..width).map(move |col| self[col..].iter().step_by(width))
    }

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
