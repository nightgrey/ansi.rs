use std::collections::Bound;
use std::ops::{Deref, DerefMut, IntoBounds, RangeBounds};
use std::slice::SliceIndex;
use ansi::Style;
use geometry::{Bounds, Position};
use super::{Index, Cell, Grapheme, GraphemePool, IntoIndex};

#[derive(Debug)]
pub struct Buffer {
    inner: Vec<Cell>,
    pool: GraphemePool,
    pub width: usize,
    pub height: usize,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![Cell::EMPTY; width * height],
            pool: GraphemePool::new(),
            width,
            height,
        }
    }

    pub fn with_capacity(width: usize, height: usize, capacity: usize) -> Self {
        Self {
            inner: vec![Cell::EMPTY; width * height],
            pool: GraphemePool::with_capacity(capacity),
            width,
            height,
        }
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    pub fn get<I: Index>(&self, index: I) -> Option<&<I::Index as SliceIndex<[Cell]>>::Output>
    {
        index.get(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut <I::Index as SliceIndex<[Cell]>>::Output>
    {
        index.index_of(self).get_mut(self.as_mut_slice())
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked<I: Index>(&self, index: I) -> *const <I::Index as SliceIndex<[Cell]>>::Output
    {
        index.index_of(self).get_unchecked(self.as_slice())
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<I: Index>(&mut self, index: I) -> *mut <I::Index as SliceIndex<[Cell]>>::Output
    {
       index.get_unchecked_mut(self)
    }

    /// Write a char at (x, y) with the given style.
    #[inline]
    pub fn put_char<I: Index<Output = Cell>>(&mut self, index: I, ch: char, style: Style) {
        let idx = index.index_of(self);
        let cell = &mut self.inner[idx];
        cell.replace_grapheme(Grapheme::from_char(ch), &mut self.pool);
        cell.set_style(style);
    }

    /// Write a string grapheme at (x, y) with the given style.
    #[inline]
    pub fn put_str(&mut self, index: impl Index<Output = Cell>, s: &str, style: Style) {
        let idx = index.index_of(self);
        let cell = &mut self.inner[idx];
        cell.replace_grapheme(Grapheme::new(s, &mut self.pool), &mut self.pool);
        cell.set_style(style);
    }

    /// Write a horizontal run of ASCII chars starting at (x, y).
    pub fn put_line(&mut self, x: usize, y: usize, text: &str, style: Style) {
        let row_start = y * self.width;
        for (i, ch) in text.chars().enumerate() {
            let col = x + i;
            if col >= self.width {
                break;
            }
            let idx = row_start + col;
            let cell = &mut self.inner[idx];
            cell.replace_grapheme(Grapheme::from_char(ch), &mut self.pool);
            cell.set_style(style);
        }
    }

    /// Clear the entire buffer, releasing all pool storage.
    pub fn clear(&mut self) {
        // Release all extended graphemes.
        for cell in &mut self.inner {
            cell.release_grapheme(&mut self.pool);
            *cell = Cell::EMPTY;
        }
        self.pool.clear();
    }

    /// Scroll up by `n` rows: move rows n.. to 0.., clear the vacated bottom.
    pub fn scroll_up(&mut self, n: usize) {
        let n = n.min(self.height);
        let w = self.width;

        // Release graphemes in the rows being scrolled off.
        for idx in 0..n * w {
            self.inner[idx].grapheme.release(&mut self.pool);
        }

        // Shift rows up.
        self.inner.copy_within(n * w.., 0);

        // Clear the vacated bottom rows.
        let start = (self.height - n) * w;
        for cell in &mut self.inner[start..] {
            *cell = Cell::EMPTY;
        }
    }

    /// Compute a diff against another buffer: yields indices where cells differ.
    pub fn diff_indices(&self, other: &Buffer) -> Vec<usize> {
        debug_assert_eq!(self.inner.len(), other.inner.len());
        self.inner
            .iter()
            .zip(other.inner.iter())
            .enumerate()
            .filter_map(|(i, (a, b))| if a != b { Some(i) } else { None })
            .collect()
    }

    pub fn index_of<I: IntoIndex<usize>>(&self, index: I) -> usize {
        index.into_index(self)
    }

    pub fn position_of<I: IntoIndex<Position>>(&self, index: I) -> Position {
        index.into_index(self)
    }
}
impl From<&Buffer> for Bounds {
    fn from(value: &Buffer) -> Self {
        Bounds::new(Position::new(0, 0), Position::new(value.height, value.width))
    }
}
impl From<&mut Buffer> for Bounds {
    fn from(value: &mut Buffer) -> Self {
        Bounds::new(Position::new(0, 0), Position::new(value.height, value.width))
    }
}
impl const Deref for Buffer {
    type Target = Vec<Cell>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl const DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl const AsRef<[Cell]> for Buffer {
    fn as_ref(&self) -> &[Cell] {
        self.inner.as_slice()
    }
}
impl const AsMut<[Cell]> for Buffer {
    fn as_mut(&mut self) -> &mut [Cell] {
        self.inner.as_mut_slice()
    }
}
#[test]
fn qwe() {
    let buffer = Buffer::new(10, 5);
    let idx = buffer.index_of(Position::new(3, 4));
}