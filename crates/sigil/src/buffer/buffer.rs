use std::ops::{Deref};
use std::slice::SliceIndex;
use derive_more::{AsMut, AsRef, Deref, DerefMut, Index, IndexMut};
use geometry::{Grid, GridIndex, IntoLocation, Position, Bounds, Row, Column};
use super::{ Cell, GraphemePool};

#[derive(Debug, Clone, Index, IndexMut, Deref, DerefMut, AsRef, AsMut)]
pub struct Buffer {
    #[index]
    #[index_mut]
    #[deref]
    #[deref_mut]
    #[as_ref(forward)]
    #[as_mut(forward)]
    inner: Grid<Cell>,
    pub pool: GraphemePool,
}

impl Buffer {
    pub const EMPTY: Self = Self {
        inner: Grid::EMPTY,
        pool: GraphemePool::EMPTY,
    };

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: Grid::new(width, height),
            pool: GraphemePool::new(),
        }
    }

    pub fn with_capacity(width: usize, height: usize, capacity: usize) -> Self {
        Self {
            inner: Grid::new(width, height),
            pool: GraphemePool::with_capacity(capacity),
        }
    }

    pub fn with_pool(width: usize, height: usize, pool: GraphemePool) -> Self {
        Self {
            inner: Grid::new(width, height),
            pool,
        }
    }

    pub fn clone_from_region(&mut self, bounds: Bounds) -> Self {
        Self {
            inner: self.inner.clone_from_region(bounds),
            pool: self.pool.clone(),
        }
    }

    /// Clear the entire buffer, releasing all pool storage.
    pub fn clear(&mut self) {
        // Release all extended graphemes.
        for cell in &mut self.inner {
            cell.release(&mut self.pool);
            cell.clear();
        }

        self.pool.clear();
    }

    pub fn to_string(&self) -> String {
        self.iter().map(|cell| cell.as_str(&self.pool)).collect()
    }
}

impl IntoLocation<Position> for Buffer {
    fn into_index(&self, location: Position) -> usize {
        (location.row) * self.width + (location.col)
    }

    fn into_position(&self, location: Position) -> Position {
        location
    }

    fn into_row(&self, location: Position) -> Row {
        Row((location.row) / self.width)
    }

    fn into_col(&self, location: Position) -> Column {
        Column((location.col) % self.width)
    }
}

#[test]
fn qwe() {
    let buffer = Buffer::new(10, 5);
    let idx = buffer.into_position(Position::new(3, 4));
}