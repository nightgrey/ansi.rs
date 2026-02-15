use geometry::{Bounds, Position};
use super::super::{Buffer, Cell};
use super::SelectorBounds;

pub struct BoundsIter {
    row: usize,
    col: usize,
    rect: Bounds,
    width: usize,
}

impl BoundsIter {
    pub fn new(rect: Bounds, within: usize) -> Self {
        BoundsIter {
            row: rect.min.row,
            col: rect.min.col,
            rect,
            width: within,
        }
    }
}

impl Iterator for BoundsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.rect.max.row { return None; }
        let idx = self.row * self.width + self.col;
        self.col += 1;
        if self.col >= self.rect.max.col {
            self.col = self.rect.min.col;
            self.row += 1;
        }
        Some(idx)
    }
}


pub struct RowSlices<'a> {
    iter: std::iter::Take<std::iter::Skip<std::slice::ChunksExact<'a, Cell>>>,
    cols: std::ops::Range<usize>,
}
impl<'a> RowSlices<'a> {
    pub fn new(bounds: Bounds, of: &'a Buffer) -> Self {
        let b = bounds.clip(Bounds::from(of));
        RowSlices {
            iter: of.chunks_exact(of.width)
                .skip(b.min.row)
                .take(b.max.row - b.min.row),
            cols: b.min.col..b.max.col,
        }
    }
}
impl<'a> Iterator for RowSlices<'a> {
    type Item = &'a [Cell];
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|row| &row[self.cols.clone()])
    }
}

pub struct RowSlicesMut<'a> {
    iter: std::iter::Take<std::iter::Skip<std::slice::ChunksExactMut<'a, Cell>>>,
    cols: std::ops::Range<usize>,
}
impl<'a> RowSlicesMut<'a> {
    pub fn new(bounds: Bounds, of: &'a mut Buffer) -> Self {
        let width = of.width;
        let height = of.height;
        let bounds = bounds.clip(Bounds::new(Position::ZERO, Position::new(height, width)));

        RowSlicesMut {
            iter: of.chunks_exact_mut(width)
                .skip(bounds.min.row)
                .take(bounds.max.row - bounds.min.row),
             cols: bounds.min.col..bounds.max.col,
        }
    }
}
impl<'a> Iterator for RowSlicesMut<'a> {
    type Item = &'a mut [Cell];
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|row| {
            let range = self.cols.clone();
            &mut row[range]
        })
    }
}

pub struct ColIter<'a> {
    inner: &'a Buffer,
    row: usize,
    row_end: usize,
    col: usize,
}

impl<'a> Iterator for ColIter<'a> {
    type Item = &'a Cell;
    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.row_end { return None; }
        let idx = self.row * self.inner.width + self.col;
        self.row += 1;
        Some(&self.inner[idx])
    }
}

pub struct Cols<'a> {
    of: &'a Buffer,
    bounds: Bounds,
    index: usize,
}

impl<'a> Cols<'a> {
    pub fn new(bounds: Bounds, of: &'a Buffer) -> Self {
        let bounds = bounds.clip(Bounds::from(of));
        Cols {
            index: bounds.min.col,
            of,
            bounds,
        }
    }
}
impl<'a> Iterator for Cols<'a> {
    type Item = ColIter<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bounds.max.col { return None; }
        let col = self.index;
        self.index += 1;
        Some(ColIter {
            inner: self.of,
            row: self.bounds.min.row,
            row_end: self.bounds.max.row,
            col,
        })
    }
}

impl Buffer {
    pub fn rows(&self, sel: impl SelectorBounds) -> RowSlices<'_> {
        todo!()
        // RowSlices::new(sel.into_concrete_bounds(self), self)
    }

    pub fn rows_mut(&mut self, sel: impl SelectorBounds) -> RowSlicesMut<'_> {
        todo!()
        // RowSlicesMut::new(sel.into_concrete_bounds(self), self)
    }
    pub fn cols(&self, sel: impl SelectorBounds) -> Cols<'_> {
        todo!()
        // Cols::new(sel.into_concrete_bounds(self), self)
    }
}