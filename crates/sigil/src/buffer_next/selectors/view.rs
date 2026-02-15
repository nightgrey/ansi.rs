use geometry::{Bounds, Position};
use super::super::{Buffer, Cell};
use super::SelectorBounds;

pub struct Sub<'a> {
    grid: &'a Buffer,
    bounds: Bounds,
}

pub struct SubMut<'a> {
    grid: &'a mut Buffer,
    bounds: Bounds,
}

impl<'a> Sub<'a> {
    pub fn get(&self, pos: Position) -> Option<&'a Cell> {
        let r = self.bounds.min.row + pos.row;
        let c = self.bounds.min.col + pos.col;
        if r < self.bounds.max.row && c < self.bounds.max.col {
            let idx = r * self.grid.width + c;
            Some(&self.grid[idx])
        } else { None }
    }
}


impl<'a> SubMut<'a> {
    pub fn get_mut(&'a mut self, pos: Position) -> Option<&'a mut Cell> {
        let r = self.bounds.min.row + pos.row;
        let c = self.bounds.min.col + pos.col;
        if r < self.bounds.max.row && c < self.bounds.max.col {
            let idx = r * self.grid.width + c;
            Some(&mut self.grid[idx])
        } else { None }
    }
}


impl Buffer {
    pub fn sub<S: SelectorBounds>(&self, selector: S) -> Sub<'_> {
        // Sub{ grid: self, bounds: selector.into_concrete_bounds(self) }
        todo!() 
    }
    pub fn sub_mut<S: SelectorBounds>(&mut self, selector: S) -> SubMut<'_> {
        // let bounds = selector.into_concrete_bounds(self);
        // SubMut{ grid: self, bounds }
        todo!()
    }
}