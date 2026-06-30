use crate::{Graphemes, Cell, Cells, BufferIndex};
use ansi::Style;
use std::borrow::{Borrow, BorrowMut};
use derive_more::{ Deref, DerefMut, IntoIterator};
use geometry::{Bounded, Point, Row};
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use super::BufferIndexExt;

#[derive(Deref, DerefMut, IntoIterator, Clone, PartialEq)]
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
    pub fn from_elements(width: u16, height: u16, elements: &[(impl BufferIndexExt, char, Style)]) -> Self {
        let mut buffer = Self::new(width, height);
        for (index, ch, style) in elements {
            for cell in index.iter_mut(&mut buffer) {
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
            buffer.set_line((0, y as u16), line, arena);
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
    pub fn get<I: BufferIndex>(
        &self,
        index: I,
    ) -> Option<&I::Output> {
        index.get(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    #[inline]
    pub fn get_mut<I: BufferIndex>(
        &mut self,
        index: I,
    ) -> Option<&mut I::Output> {
        index.get_mut(self)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn get_unchecked<I: BufferIndex>(
        &self,
        index: I,
    ) -> *const I::Output {
        unsafe { index.get_unchecked(self) }
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn get_unchecked_mut<I: BufferIndex>(
        &mut self,
        index: I,
    ) -> *mut I::Output {
        unsafe { index.get_unchecked_mut(self) }
    }

    #[inline]
    pub fn contains(&self, index: impl BufferIndexExt) -> bool {
        index.within(self)
    }

    #[inline]
    pub fn index_of(&self, index: impl BufferIndexExt) -> usize {
        index.into_index(self)
    }

    #[inline]
    pub fn point_of(&self, index: impl BufferIndexExt) -> Point {
        index.into_point(self)
    }

    #[inline]
    pub fn range_of(&self, range: impl BufferIndexExt) -> ops::Range<usize> {
        range.into_range(self)
    }

    #[inline]
    pub fn len_of(&self, range: impl BufferIndexExt) -> usize {
        range.len(self)
    }

    #[inline]
    pub fn start_of(&self, range: impl BufferIndexExt) -> usize {
        range.start(self)
    }

    #[inline]
    pub fn end_of(&self, range: impl BufferIndexExt) -> usize {
        range.end(self)
    }

    /// Print the given string until the end of the given index.
    pub fn set_string(
        &mut self,
        index: impl BufferIndexExt,
        string: impl AsRef<str>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
       self.set_string_impl(index, string, None, arena)
    }

    pub fn set_string_styled(
        &mut self,
        index: impl BufferIndexExt,
        string: impl AsRef<str>,
        style: Style,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        self.set_string_impl(index, string, Some(style), arena)
    }

    /// Write one measured grapheme and all of its continuation cells.
    ///
    /// Returns `None` when the grapheme is zero-width or its complete display
    /// width does not fit in the buffer row.
    pub fn set_grapheme_styled(
        &mut self,
        index: impl BufferIndexExt<Output = Cell>,
        grapheme: &str,
        width: usize,
        style: Style,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        if !index.within(self) {
            return None;
        }

        let position = index.into_point(self);

        let start_x = position.x;
        let end_x = start_x.checked_add(width as u16)?;

        if end_x > self.width {
            return None;
        }

        let start = position.y * self.width + start_x;
        Some(
            Cells::write_into(&mut self.inner[(start as usize)..((start as usize + width))], grapheme, width, Some(style), arena),
        )
    }

    /// Print the given string until the end of the given index.
    fn set_string_impl(
        &mut self,
        index: impl BufferIndexExt,
        string: impl AsRef<str>,
        style: Option<Style>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        let slice = self.get_mut(index.into_range(self))?;
        let mut remaining = slice.len();
        let mut i = 0;

        for grapheme in string.as_ref().graphemes(true) {
            if grapheme.contains(char::is_control) {
                continue;
            }

            let width = grapheme.width();
            remaining = remaining.checked_sub(width)?;

            i += Cells::write_into((&mut slice[i..]), grapheme, width, style, arena);
        }

        Some(i)
    }
    /// Print the given string until the end of the line.
    pub fn set_line(
        &mut self,
        index: impl BufferIndexExt<Output = Cell>,
        string: impl AsRef<str>,
        arena: &mut Graphemes,
    ) -> Option<usize> {
        let point = index.into_point(self);

        self.set_string(
            point..Point {
                x: self.width(),
                y: point.y,
            },
            string,
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

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        self.inner.chunks_exact(self.width as usize).map(|row| row.iter())
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
                    self
                        .iter()
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
            return Ok(())
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

    fn min(&self) -> Point{
        Point::ZERO
    }

    fn max(&self) -> Point{
        Point::new(self.max_x(), self.max_y())
    }
}