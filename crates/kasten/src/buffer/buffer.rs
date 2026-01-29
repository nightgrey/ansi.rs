use crate::{BufferIndex, BufferSelector, Cell, Point, Rect};
use ansi::fmt::Write;
use ansi::{Escape, Style};
use derive_more::{Deref, DerefMut};
use geometry::{Position, Region, RegionIter};
use std::fmt::{Display, Formatter};
use std::io::Write as _;
use std::slice::SliceIndex;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// TODO: Check https://lib.rs/crates/stable-vec
// https://github.com/HarrisonMc555/array2d
#[derive(Clone, PartialEq, Deref, DerefMut, Debug)]
pub struct Buffer {
    #[deref]
    #[deref_mut]
    inner: Vec<Cell>,
    pub bounds: Rect,
}

impl Buffer {
    pub const ZERO: Self = Self {
        bounds: Rect::ZERO,
        inner: Vec::new(),
    };

    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            inner: vec![Cell::EMPTY; bounds.area()],
        }
    }

    pub const fn min(&self) -> Point {
        self.bounds.min
    }

    pub const fn max(&self) -> Point {
        self.bounds.max
    }

    pub const fn width(&self) -> usize {
        self.bounds.width()
    }

    pub const fn height(&self) -> usize {
        self.bounds.height()
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    pub fn get<Index>(&self, index: Index) -> Option<&Index::Output>
    where
        Index: BufferIndex,
    {
        index.get(self)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    pub fn get_mut<Index>(&mut self, index: Index) -> Option<&mut Index::Output>
    where
        Index: BufferIndex,
    {
        index.get_mut(self)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked<Index>(&self, index: Index) -> *const Index::Output
    where
        Index: BufferIndex,
    {
        index.get_unchecked(self)
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut<Index>(&mut self, index: Index) -> &mut Index::Output
    where
        Index: BufferIndex,
    {
        &mut *index.get_unchecked_mut(self)
    }

    pub fn select<'a>(
        &'a self,
        selector: &'a impl BufferSelector,
    ) -> impl Iterator<Item = Position> + 'a {
        selector.positions(self)
    }

    pub fn text(
        &mut self,
        index: impl BufferIndex<Output = [Cell]>,
        string: impl AsRef<str>,
        style: &Style,
    ) {
        if let Some(cells) = index.get_mut(self) {
            let mut remaining = cells.len();
            let mut i = 0;

            for (grapheme, width) in string
                .as_ref()
                .graphemes(true)
                .filter(|symbol| !symbol.contains(char::is_control))
                .map(|symbol| (symbol, symbol.width()))
                .filter(|(_symbol, width)| *width > 0)
                .map_while(|(symbol, width)| {
                    remaining = remaining.checked_sub(width)?;
                    Some((symbol, width))
                })
            {
                // Set the starting cell
                cells[i].set_content(grapheme);
                cells[i].set_style(style);
                let next_symbol = i + width;
                i += 1;

                // Reset subsequent cells for multi-width graphemes
                while i < next_symbol {
                    cells[i].clear();
                    i += 1;
                }
            }
        }
    }

    pub fn style(&mut self, index: impl BufferIndex<Output = [Cell]>, style: Style) {
        if let Some(cells) = index.get_mut(self) {
            for cell in cells {
                cell.set_style(&style);
            }
        }
    }

    pub fn contains(&self, position: &Position) -> bool {
        self.bounds.contains(&Point::from(*position))
    }

    pub fn index_of(&self, index: &Position) -> Option<usize> {
        index.index_of(self)
    }

    pub fn position_of(&self, index: usize) -> Position {
        Position {
            row: index / self.width(),
            col: index % self.width(),
        }
    }

    pub fn as_slice(&self) -> &[Cell] {
        &self.inner
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Cell> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Cell> {
        self.inner.iter_mut()
    }

    pub fn iter_rows(&self) -> std::slice::Chunks<'_, Cell> {
        self.inner.chunks(self.width())
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_escape(self)
    }
}

impl IntoIterator for Buffer {
    type Item = Cell;
    type IntoIter = std::vec::IntoIter<Cell>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl AsRef<[Cell]> for Buffer {
    fn as_ref(&self) -> &[Cell] {
        self.inner.as_ref()
    }
}
impl AsMut<[Cell]> for Buffer {
    fn as_mut(&mut self) -> &mut [Cell] {
        self.inner.as_mut()
    }
}

impl Escape for Buffer {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use ansi::io::Write;

        let mut last_style = Style::EMPTY;

        for (position, cell) in self
            .iter()
            .enumerate()
            .map(|(i, cell)| (self.position_of(i), cell))
        {
            if cell.style != last_style {
                w.write_escape(&last_style.diff(cell.style))?;

                last_style = cell.style;
            }

            w.write(&cell.as_bytes())?;

            if position.col == self.width() - 1 {
                w.write_escape(&Style::Reset)?;
                w.write(b"\n")?;
                last_style = Style::EMPTY;
            }
        }
        Ok(())
    }
}
