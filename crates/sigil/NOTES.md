# Notes

**Possibly useful?**

- https://lib.rs/crates/stable-vec
- https://github.com/HarrisonMc555/array2d

# request

```
use std::ops::{RangeBounds, Bound};

/// 2D position
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position { 
    pub row: usize, 
    pub col: usize 
}

impl From<(usize, usize)> for Position {
    fn from(t: (usize, usize)) -> Self { 
        Position { row: t.0, col: t.1 } 
    }
}

/// Rectangular bounds (min inclusive, max exclusive)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bounds {
    pub min: Position,
    pub max: Position,
}

pub struct Grid<T> {
    inner: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    fn idx(&self, p: Position) -> usize { p.row * self.width + p.col }
}

// ============================================================================
// Bounds operations
// ============================================================================

impl Bounds {
    /// Check if bounds have zero area
    pub fn is_empty(&self) -> bool {
        self.min.row >= self.max.row || self.min.col >= self.max.col
    }
    
    /// Area of the bounds
    pub fn area(&self) -> usize {
        if self.is_empty() { 0 } else {
            (self.max.row - self.min.row) * (self.max.col - self.min.col)
        }
    }
    
    /// Clamp bounds to grid dimensions (ensures within 0..height, 0..width)
    pub fn clip(&self, grid: &Grid<impl Sized>) -> Self {
        Bounds {
            min: Position {
                row: self.min.row.min(grid.height),
                col: self.min.col.min(grid.width),
            },
            max: Position {
                row: self.max.row.min(grid.height),
                col: self.max.col.min(grid.width),
            },
        }
    }
    
    /// Find intersection between two bounds. Returns None if disjoint.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let min = Position {
            row: self.min.row.max(other.min.row),
            col: self.min.col.max(other.min.col),
        };
        let max = Position {
            row: self.max.row.min(other.max.row),
            col: self.max.col.min(other.max.col),
        };
        
        if min.row < max.row && min.col < max.col {
            Some(Bounds { min, max })
        } else {
            None
        }
    }
}

// ============================================================================
// GridSelector trait (base iteration)
// ============================================================================

pub struct BoundsIter {
    cur: Position,
    bounds: Bounds,
    width: usize,
}

impl Iterator for BoundsIter {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.row >= self.bounds.max.row { return None; }
        let idx = self.cur.row * self.width + self.cur.col;
        
        self.cur.col += 1;
        if self.cur.col >= self.bounds.max.col {
            self.cur.col = self.bounds.min.col;
            self.cur.row += 1;
        }
        Some(idx)
    }
}

pub trait GridSelector {
    type Iter: Iterator<Item = usize>;
    
    /// Iterate indices in row-major order
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter;
    
    /// Logical bounds (may exceed grid dimensions)
    fn bounds(&self, grid: &Grid<impl Sized>) -> Bounds;
    
    /// Check if clipped bounds are empty
    fn is_empty(&self, grid: &Grid<impl Sized>) -> bool {
        self.bounds(grid).clip(grid).is_empty()
    }
    
    /// Count of cells within clipped bounds
    fn len(&self, grid: &Grid<impl Sized>) -> usize {
        self.bounds(grid).clip(grid).area()
    }
}

// ============================================================================
// GridSelectorBounds trait (extended bounds operations)
// ============================================================================

pub trait GridSelectorBounds: GridSelector {
    /// Start bound with inclusivity info (similar to RangeBounds)
    fn start_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position>;
    
    /// End bound with inclusivity info (similar to RangeBounds)
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position>;
    
    /// Intersection with another selector. Returns (start_bound, end_bound).
    /// Handles Bound::Included/Excluded/Unbounded correctly.
    fn intersect(
        &self, 
        other: &impl GridSelectorBounds, 
        grid: &Grid<impl Sized>
    ) -> (Bound<Position>, Bound<Position>) {
        // Helper to extract position value
        fn pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: 0, col: 0 },
            }
        }
        
        // Max of starts
        let self_start = self.start_bound(grid);
        let other_start = other.start_bound(grid);
        let start = match (&self_start, &other_start) {
            (Bound::Unbounded, b) => b.clone(),
            (b, Bound::Unbounded) => b.clone(),
            (a, b) => {
                let pos_a = pos(*a);
                let pos_b = pos(*b);
                let max_pos = if pos_a >= pos_b { pos_a } else { pos_b };
                // If both included, result is included; else excluded
                match (a, b) {
                    (Bound::Included(_), Bound::Included(_)) => Bound::Included(max_pos),
                    _ => Bound::Excluded(max_pos),
                }
            }
        };
        
        // Min of ends
        fn end_pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
            }
        }
        
        let self_end = self.end_bound(grid);
        let other_end = other.end_bound(grid);
        let end = match (&self_end, &other_end) {
            (Bound::Unbounded, b) => b.clone(),
            (b, Bound::Unbounded) => b.clone(),
            (a, b) => {
                let pos_a = end_pos(*a);
                let pos_b = end_pos(*b);
                let min_pos = if pos_a <= pos_b { pos_a } else { pos_b };
                // If both excluded, result is excluded; else included
                match (a, b) {
                    (Bound::Excluded(_), Bound::Excluded(_)) => Bound::Excluded(min_pos),
                    _ => Bound::Included(min_pos),
                }
            }
        };
        
        (start, end)
    }
    
    /// Union with another selector (covering both areas). Returns (start_bound, end_bound).
    fn union(
        &self, 
        other: &impl GridSelectorBounds, 
        grid: &Grid<impl Sized>
    ) -> (Bound<Position>, Bound<Position>) {
        fn pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: 0, col: 0 },
            }
        }
        
        fn end_pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
            }
        }
        
        // Min of starts
        let self_start = self.start_bound(grid);
        let other_start = other.start_bound(grid);
        let start = match (&self_start, &other_start) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => Bound::Unbounded,
            (a, b) => {
                let pos_a = pos(*a);
                let pos_b = pos(*b);
                let min_pos = if pos_a <= pos_b { pos_a } else { pos_b };
                // If either included, result is included
                match (a, b) {
                    (Bound::Excluded(_), Bound::Excluded(_)) => Bound::Excluded(min_pos),
                    _ => Bound::Included(min_pos),
                }
            }
        };
        
        // Max of ends
        let self_end = self.end_bound(grid);
        let other_end = other.end_bound(grid);
        let end = match (&self_end, &other_end) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => Bound::Unbounded,
            (a, b) => {
                let pos_a = end_pos(*a);
                let pos_b = end_pos(*b);
                let max_pos = if pos_a >= pos_b { pos_a } else { pos_b };
                // If either excluded, result is excluded
                match (a, b) {
                    (Bound::Included(_), Bound::Included(_)) => Bound::Included(max_pos),
                    _ => Bound::Excluded(max_pos),
                }
            }
        };
        
        (start, end)
    }
    
    /// Clamp bounds to grid boundaries (safe for iteration)
    fn clip(&self, grid: &Grid<impl Sized>) -> Bounds {
        self.into_bounds(grid).clip(grid)
    }
    
    /// Convert to Bounds, handling inclusive/exclusive/unbounded correctly.
    /// Also handles row wrapping: if col >= width, wraps to next row(s).
    fn into_bounds(&self, grid: &Grid<impl Sized>) -> Bounds {
        // Convert start bound to inclusive Position
        let min = match self.start_bound(grid) {
            Bound::Included(p) => p,
            Bound::Excluded(p) => Position { 
                row: p.row, 
                col: p.col.saturating_add(1) 
            },
            Bound::Unbounded => Position { row: 0, col: 0 },
        };
        
        // Convert end bound to exclusive Position
        let max = match self.end_bound(grid) {
            Bound::Included(p) => Position { 
                row: p.row, 
                col: p.col.saturating_add(1) 
            },
            Bound::Excluded(p) => p,
            Bound::Unbounded => Position { row: grid.height, col: grid.width },
        };
        
        // Handle row wrapping: normalize positions where col >= grid.width
        let wrap = |p: Position| -> Position {
            if p.col >= grid.width {
                Position {
                    row: p.row.saturating_add(p.col / grid.width),
                    col: p.col % grid.width,
                }
            } else {
                p
            }
        };
        
        Bounds { 
            min: wrap(min), 
            max: wrap(max) 
        }
    }
}

// ============================================================================
// Implementations for GridSelector
// ============================================================================

impl GridSelector for Bounds {
    type Iter = BoundsIter;
    
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        let clipped = self.clip(grid);
        BoundsIter {
            cur: clipped.min,
            bounds: clipped,
            width: grid.width,
        }
    }
    
    fn bounds(&self, _grid: &Grid<impl Sized>) -> Bounds { *self }
}

impl GridSelector for Position {
    type Iter = std::iter::Once<usize>;
    
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        std::iter::once(grid.idx(self))
    }
    
    fn bounds(&self, _grid: &Grid<impl Sized>) -> Bounds {
        Bounds {
            min: *self,
            max: Position { row: self.row + 1, col: self.col + 1 },
        }
    }
}

impl GridSelector for (usize, usize) {
    type Iter = std::iter::Once<usize>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        Position::from(self).iter(grid)
    }
    fn bounds(&self, grid: &Grid<impl Sized>) -> Bounds {
        Position::from(*self).bounds(grid)
    }
}

impl GridSelector for usize {
    type Iter = std::iter::Once<usize>;
    fn iter(self, _grid: &Grid<impl Sized>) -> Self::Iter {
        std::iter::once(self)
    }
    fn bounds(&self, grid: &Grid<impl Sized>) -> Bounds {
        let pos = Position { 
            row: self / grid.width, 
            col: self % grid.width 
        };
        pos.bounds(grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Row(pub usize);
#[derive(Clone, Copy, Debug)]
pub struct Col(pub usize);

impl GridSelector for Row {
    type Iter = std::ops::Range<usize>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        let start = self.0.saturating_mul(grid.width);
        let end = ((self.0 + 1) * grid.width).min(grid.inner.len());
        start..end
    }
    fn bounds(&self, grid: &Grid<impl Sized>) -> Bounds {
        Bounds {
            min: Position { row: self.0, col: 0 },
            max: Position { row: self.0 + 1, col: grid.width },
        }
    }
}

impl GridSelector for Col {
    type Iter = std::iter::StepBy<std::ops::Range<usize>>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        let start = self.0;
        let end = grid.inner.len();
        (start..end).step_by(grid.width)
    }
    fn bounds(&self, grid: &Grid<impl Sized>) -> Bounds {
        Bounds {
            min: Position { row: 0, col: self.0 },
            max: Position { row: grid.height, col: self.0 + 1 },
        }
    }
}

// Support (0,0)..(10,20) syntax via RangeBounds
impl<R: RangeBounds<Position> + Copy> GridSelector for R {
    type Iter = BoundsIter;
    
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        self.bounds(grid).iter(grid)
    }
    
    fn bounds(&self, _grid: &Grid<impl Sized>) -> Bounds {
        let min = match self.start_bound() {
            Bound::Included(p) => *p,
            Bound::Excluded(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Unbounded => Position { row: 0, col: 0 },
        };
        let max = match self.end_bound() {
            Bound::Included(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Excluded(p) => *p,
            Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
        };
        Bounds { min, max }
    }
}

// ============================================================================
// Implementations for GridSelectorBounds
// ============================================================================

impl GridSelectorBounds for Bounds {
    fn start_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(self.min)
    }
    
    fn end_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(self.max)
    }
}

impl GridSelectorBounds for Position {
    fn start_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(*self)
    }
    
    fn end_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(Position { 
            row: self.row + 1, 
            col: self.col + 1 
        })
    }
}

impl GridSelectorBounds for (usize, usize) {
    fn start_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position::from(*self))
    }
    
    fn end_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        let p = Position::from(*self);
        Bound::Excluded(Position { 
            row: p.row + 1, 
            col: p.col + 1 
        })
    }
}

impl GridSelectorBounds for usize {
    fn start_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position {
            row: self / grid.width,
            col: self % grid.width,
        })
    }
    
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        let pos = Position {
            row: self / grid.width,
            col: self % grid.width,
        };
        Bound::Excluded(Position {
            row: pos.row + 1,
            col: pos.col + 1,
        })
    }
}

impl GridSelectorBounds for Row {
    fn start_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position { row: self.0, col: 0 })
    }
    
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(Position { 
            row: self.0 + 1, 
            col: grid.width 
        })
    }
}

impl GridSelectorBounds for Col {
    fn start_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position { row: 0, col: self.0 })
    }
    
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(Position { 
            row: grid.height, 
            col: self.0 + 1 
        })
    }
}

// Delegate to RangeBounds trait methods
impl<R: RangeBounds<Position> + Copy> GridSelectorBounds for R {
    fn start_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        self.start_bound()
    }
    
    fn end_bound(&self, _grid: &Grid<impl Sized>) -> Bound<Position> {
        self.end_bound()
    }
}

// ============================================================================
// Usage examples
// ============================================================================

fn demo(grid: &Grid<impl Sized>) {
    use std::ops::Bound;
    
    // 1. Get raw bounds with inclusivity info
    let row = Row(5);
    assert!(matches!(row.start_bound(grid), Bound::Included(_)));
    
    // 2. Intersection returns (Bound, Bound) like RangeBounds
    let range1 = (0, 0)..(10, 10);
    let range2 = (5, 5)..(15, 15);
    let (start, end) = range1.intersect(&range2, grid);
    // start = Included((5,5)), end = Excluded((10,10))
    
    // 3. Convert to concrete Bounds (handles inclusive/exclusive/unbounded)
    let bounds: Bounds = range1.into_bounds(grid);
    
    // 4. Union covers both areas
    let cell1 = Position { row: 0, col: 0 };
    let cell2 = Position { row: 5, col: 5 };
    let (start, end) = cell1.union(&cell2, grid);
    // Returns range covering both cells
    
    // 5. Clip ensures within grid
    let big = (0, 0)..(1000, 1000);
    let clipped = big.clip(grid); // clamped to grid size
    
    // 6. Row wrapping: linear index 5 in width-3 grid becomes (1, 2)
    let idx = 5usize;
    let bounds = idx.into_bounds(grid); // handles col overflow to next row
}
```

please help me add/edit and improve this by doing:

- second clip fn arg should be another GridSelectorBounds + Grid, not Grid alone
- remove GridSelector::bounds() - replaced by GridSelectorBounds::into_bounds()
- add iterators for rows/rects and others (if possible) that return row-wise slices
- other optimizations?

focus: performance, ergonomics, clean naming and code

# Response 1

Below is a cleaned‑up, more ergonomic and faster version of your API that incorporates your requests:

✅ **changes**  
- `GridSelector::bounds()` removed – use `GridSelectorBounds::into_bounds()`  
- `clip()` now takes **another `GridSelectorBounds` + `Grid`**, not only `Grid`  
- adds **row‑wise slice iterators** (rectangular slicing)  
- improved naming and small perf tweaks  

---

## ✅ Core changes

### Bounds: clip to grid or to another selector
```rust
impl Bounds {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.row >= self.max.row || self.min.col >= self.max.col
    }

    #[inline]
    pub fn area(&self) -> usize {
        if self.is_empty() { 0 }
        else { (self.max.row - self.min.row) * (self.max.col - self.min.col) }
    }

    #[inline]
    pub fn clip_to_grid(&self, grid: &Grid<impl Sized>) -> Self {
        Bounds {
            min: Position {
                row: self.min.row.min(grid.height),
                col: self.min.col.min(grid.width),
            },
            max: Position {
                row: self.max.row.min(grid.height),
                col: self.max.col.min(grid.width),
            },
        }
    }

    #[inline]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let min = Position {
            row: self.min.row.max(other.min.row),
            col: self.min.col.max(other.min.col),
        };
        let max = Position {
            row: self.max.row.min(other.max.row),
            col: self.max.col.min(other.max.col),
        };
        if min.row < max.row && min.col < max.col {
            Some(Bounds { min, max })
        } else {
            None
        }
    }
}
```

---

## ✅ GridSelector (simple iteration only)

```rust
pub trait GridSelector {
    type Iter: Iterator<Item = usize>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter;
}
```

---

## ✅ GridSelectorBounds – bounds/clip/union/intersection

```rust
pub trait GridSelectorBounds {
    fn start_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position>;
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position>;

    fn into_bounds(&self, grid: &Grid<impl Sized>) -> Bounds {        let min = match self.start_bound(grid) {
            Bound::Included(p) => p,
            Bound::Excluded(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Unbounded => Position { row: 0, col: 0 },
        };

        let max = match self.end_bound(grid) {
            Bound::Included(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Excluded(p) => p,
            Bound::Unbounded => Position { row: grid.height, col: grid.width },
        };

        // row wrapping for linear bounds
        let wrap = |p: Position| {
            if p.col >= grid.width {
                Position {
                    row: p.row + p.col / grid.width,
                    col: p.col % grid.width,
                }
            } else { p }
        };

        Bounds { min: wrap(min), max: wrap(max) }
    }

    /// Clip to another selector inside this grid
    fn clip(&self, other: &impl GridSelectorBounds, grid: &Grid<impl Sized>) -> Bounds {
        let (s, e) = self.intersect(other, grid);
        Bounds::from_bounds(s, e, grid).clip_to_grid(grid)
    }

    /// Intersection (preserves inclusivity)
    fn intersect(
        &self,
        other: &impl GridSelectorBounds,
        grid: &Grid<impl Sized>,
    ) -> (Bound<Position>, Bound<Position>) { /* same logic as before */ }

    fn union(
        &self,
        other: &impl GridSelectorBounds,
        grid: &Grid<impl Sized>,
    ) -> (Bound<Position>, Bound<Position>) { /* same logic as before */ }
}

// helper: construct Bounds from a range of Bounds
impl Bounds {
    pub fn from_bounds(
        start: Bound<Position>,
        end: Bound<Position>,
        grid: &Grid<impl Sized>,
    ) -> Bounds {
        struct Tmp(Bound<Position>, Bound<Position>);
        impl GridSelectorBounds for Tmp {
            fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> { self.0 }
            fn end_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> { self.1 }
        }
        Tmp(start, end).into_bounds(grid)
    }
}
```

---

## ✅ New row‑wise slice iterators (rectangular slicing)

These are fast and zero‑alloc:

```rust
impl<T> Grid<T> {
    /// Iterate row slices in a rectangular selector
    pub fn rows(&self, sel: impl GridSelectorBounds) -> impl Iterator<Item = &[T]> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        let width = self.width;
        self.inner
            .chunks_exact(width)
            .skip(b.min.row)
            .take(b.max.row - b.min.row)
            .map(move |row| &row[b.min.col..b.max.col])
    }

    /// Mutable row slices
    pub fn rows_mut(&mut self, sel: impl GridSelectorBounds) -> impl Iterator<Item = &mut [T]> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        let width = self.width;
        self.inner
            .chunks_exact_mut(width)
            .skip(b.min.row)
            .take(b.max.row - b.min.row)
            .map(move |row| &mut row[b.min.col..b.max.col])
    }
}
```

Example usage:

```rust
let rect = (2,2)..(5,6);
for row in grid.rows(rect) {
    // row: &[T] of 4 cols
}
```

---

## ✅ Updated Iteration for selectors

Without `bounds()`, just call `into_bounds()`.

```rust
impl GridSelector for Bounds {
    type Iter = BoundsIter;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        let clipped = self.clip_to_grid(grid);
        BoundsIter { cur: clipped.min, bounds: clipped, width: grid.width }
    }
}

impl<R: RangeBounds<Position> + Copy> GridSelector for R {
    type Iter = BoundsIter;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        self.into_bounds(grid).clip_to_grid(grid).iter(grid)
    }
}
```

---

## ✅ Optional micro‑optimizations

You can add:

```rust
#[inline(always)]
fn idx(&self, p: Position) -> usize { p.row * self.width + p.col }

impl Iterator for BoundsIter {
    type Item = usize;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> { /* unchanged */ }
}
```

---

# ✅ Summary

What you now get:

✔ clean selector API  
✔ performant bounds/clip semantics  
✔ support for **row‑wise slices** for rectangles  
✔ removed redundant `bounds()`  

---

If you want, I can also:

- add `col()` / `cols()` slice iterators
- add `SubGrid` view type
- add `IntoIterator` for selectors
- make row/col iteration `ExactSizeIterator`  

Just say the word.


# Response 2

Below is a **full upgrade** with everything you asked for:

✅ **All of these are included**

- **`GridSelector::bounds()` removed** – use `GridSelectorBounds::into_bounds()`
- **`clip()` now takes another selector + grid**
- **row‑wise slicing iterators** (rectangular)
- **column iterators**
- **SubGrid view type** (`Grid::select()` / `select_mut()`)
- **IntoIterator for SubGrid**
- **ExactSizeIterator where possible**
- small perf tweaks & inlining

---

# ✅ Full upgraded version

```rust
use std::ops::{Bound, RangeBounds};

// ============================================================================
// Position + Bounds
// ============================================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position { pub row: usize, pub col: usize }

impl From<(usize, usize)> for Position {
    #[inline] fn from(t: (usize, usize)) -> Self {
        Position { row: t.0, col: t.1 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bounds { pub min: Position, pub max: Position }

impl Bounds {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.row >= self.max.row || self.min.col >= self.max.col
    }
    #[inline]
    pub fn area(&self) -> usize {
        if self.is_empty() { 0 }
        else { (self.max.row - self.min.row) * (self.max.col - self.min.col) }
    }

    #[inline]
    pub fn clip_to_grid(&self, grid: &Grid<impl Sized>) -> Self {
        Bounds {
            min: Position {
                row: self.min.row.min(grid.height),
                col: self.min.col.min(grid.width),
            },
            max: Position {
                row: self.max.row.min(grid.height),
                col: self.max.col.min(grid.width),
            },
        }
    }

    #[inline]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let min = Position {
            row: self.min.row.max(other.min.row),
            col: self.min.col.max(other.min.col),
        };
        let max = Position {
            row: self.max.row.min(other.max.row),
            col: self.max.col.min(other.max.col),
        };
        if min.row < max.row && min.col < max.col {
            Some(Bounds { min, max })
        } else { None }
    }

    pub fn from_bounds(
        start: Bound<Position>,
        end: Bound<Position>,
        grid: &Grid<impl Sized>,
    ) -> Bounds {
        struct Tmp(Bound<Position>, Bound<Position>);
        impl GridSelectorBounds for Tmp {
            fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> { self.0 }
            fn end_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> { self.1 }
        }
        Tmp(start, end).into_bounds(grid)
    }
}

// ============================================================================
// Grid
// ============================================================================

pub struct Grid<T> {
    inner: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    #[inline] pub fn width(&self) -> usize { self.width }
    #[inline] pub fn height(&self) -> usize { self.height }

    #[inline]
    fn idx(&self, p: Position) -> usize { p.row * self.width + p.col }

    // ------------------------------------------------------------------------
    // SubGrid views
    // ------------------------------------------------------------------------
    #[inline]
    pub fn select(&self, sel: impl GridSelectorBounds) -> SubGrid<'_, T> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        SubGrid { grid: self, bounds: b }
    }

    #[inline]
    pub fn select_mut(&mut self, sel: impl GridSelectorBounds) -> SubGridMut<'_, T> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        SubGridMut { grid: self, bounds: b }
    }

    // ------------------------------------------------------------------------
    // Row-wise slice iterators
    // ------------------------------------------------------------------------
    pub fn rows(&self, sel: impl GridSelectorBounds) -> RowSlices<'_, T> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        RowSlices {
            iter: self.inner.chunks_exact(self.width)
                .skip(b.min.row)
                .take(b.max.row - b.min.row),
            col_range: b.min.col..b.max.col,
        }
    }

    pub fn rows_mut(&mut self, sel: impl GridSelectorBounds) -> RowSlicesMut<'_, T> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        RowSlicesMut {
            iter: self.inner.chunks_exact_mut(self.width)
                .skip(b.min.row)
                .take(b.max.row - b.min.row),
            col_range: b.min.col..b.max.col,
        }
    }

    // ------------------------------------------------------------------------
    // Column iterators (not contiguous slices, yields &T)
    // ------------------------------------------------------------------------
    pub fn cols(&self, sel: impl GridSelectorBounds) -> Cols<'_, T> {
        let b = sel.into_bounds(self).clip_to_grid(self);
        Cols {
            grid: self,
            bounds: b,
            cur_col: b.min.col,
        }
    }
}

// ============================================================================
// Bounds iterator (ExactSizeIterator)
// ============================================================================

pub struct BoundsIter {
    cur: Position,
    bounds: Bounds,
    width: usize,
    remaining: usize,
}

impl BoundsIter {
    #[inline]
    fn new(bounds: Bounds, width: usize) -> Self {
        BoundsIter { cur: bounds.min, bounds, width, remaining: bounds.area() }
    }
}

impl Iterator for BoundsIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 { return None; }
        let idx = self.cur.row * self.width + self.cur.col;
        self.remaining -= 1;

        self.cur.col += 1;
        if self.cur.col >= self.bounds.max.col {
            self.cur.col = self.bounds.min.col;
            self.cur.row += 1;
        }
        Some(idx)
    }

    #[inline] fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl ExactSizeIterator for BoundsIter {}

// ============================================================================
// GridSelector
// ============================================================================

pub trait GridSelector {
    type Iter: Iterator<Item = usize>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter;
}

// ============================================================================
// GridSelectorBounds
// ============================================================================

pub trait GridSelectorBounds {
    fn start_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position>;
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position>;

    fn into_bounds(&self, grid: &Grid<impl Sized>) -> Bounds {
        let min = match self.start_bound(grid) {
            Bound::Included(p) => p,
            Bound::Excluded(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Unbounded => Position { row: 0, col: 0 },
        };

        let max = match self.end_bound(grid) {
            Bound::Included(p) => Position { row: p.row, col: p.col + 1 },
            Bound::Excluded(p) => p,
            Bound::Unbounded => Position { row: grid.height, col: grid.width },
        };

        let wrap = |p: Position| {
            if p.col >= grid.width {
                Position { row: p.row + p.col / grid.width, col: p.col % grid.width }
            } else { p }
        };

        Bounds { min: wrap(min), max: wrap(max) }
    }

    fn clip(&self, other: &impl GridSelectorBounds, grid: &Grid<impl Sized>) -> Bounds {
        let (s, e) = self.intersect(other, grid);
        Bounds::from_bounds(s, e, grid).clip_to_grid(grid)
    }

    fn intersect(
        &self,
        other: &impl GridSelectorBounds,
        grid: &Grid<impl Sized>,
    ) -> (Bound<Position>, Bound<Position>) {
        fn pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: 0, col: 0 },
            }
        }
        fn end_pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
            }
        }

        let s1 = self.start_bound(grid);
        let s2 = other.start_bound(grid);
        let start = match (&s1, &s2) {
            (Bound::Unbounded, b) => b.clone(),
            (b, Bound::Unbounded) => b.clone(),
            (a, b) => {
                let maxp = if pos(*a) >= pos(*b) { pos(*a) } else { pos(*b) };
                match (a, b) {
                    (Bound::Included(_), Bound::Included(_)) => Bound::Included(maxp),
                    _ => Bound::Excluded(maxp),
                }
            }
        };

        let e1 = self.end_bound(grid);
        let e2 = other.end_bound(grid);
        let end = match (&e1, &e2) {
            (Bound::Unbounded, b) => b.clone(),
            (b, Bound::Unbounded) => b.clone(),
            (a, b) => {
                let minp = if end_pos(*a) <= end_pos(*b) { end_pos(*a) } else { end_pos(*b) };
                match (a, b) {
                    (Bound::Excluded(_), Bound::Excluded(_)) => Bound::Excluded(minp),
                    _ => Bound::Included(minp),
                }
            }
        };

        (start, end)
    }

    fn union(
        &self,
        other: &impl GridSelectorBounds,
        grid: &Grid<impl Sized>,
    ) -> (Bound<Position>, Bound<Position>) {
        fn pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: 0, col: 0 },
            }
        }
        fn end_pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
            }
        }

        let s1 = self.start_bound(grid);
        let s2 = other.start_bound(grid);
        let start = match (&s1, &s2) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => Bound::Unbounded,
            (a, b) => {
                let minp = if pos(*a) <= pos(*b) { pos(*a) } else { pos(*b) };
                match (a, b) {
                    (Bound::Excluded(_), Bound::Excluded(_)) => Bound::Excluded(minp),
                    _ => Bound::Included(minp),
                }
            }
        };

        let e1 = self.end_bound(grid);
        let e2 = other.end_bound(grid);
        let end = match (&e1, &e2) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => Bound::Unbounded,
            (a, b) => {
                let maxp = if end_pos(*a) >= end_pos(*b) { end_pos(*a) } else { end_pos(*b) };
                match (a, b) {
                    (Bound::Included(_), Bound::Included(_)) => Bound::Included(maxp),
                    _ => Bound::Excluded(maxp),
                }
            }
        };

        (start, end)
    }
}

// ============================================================================
// Selector implementations (Bounds, Position, Range, Row, Col, usize)
// ============================================================================

impl GridSelector for Bounds {
    type Iter = BoundsIter;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        BoundsIter::new(self.clip_to_grid(grid), grid.width)
    }
}

impl GridSelectorBounds for Bounds {
    fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(self.min)
    }
    fn end_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(self.max)
    }
}

impl GridSelector for Position {
    type Iter = std::iter::Once<usize>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        std::iter::once(grid.idx(self))
    }
}
impl GridSelectorBounds for Position {
    fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(*self)
    }
    fn end_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(Position { row: self.row + 1, col: self.col + 1 })
    }
}

impl GridSelector for (usize, usize) {
    type Iter = std::iter::Once<usize>;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter { Position::from(self).iter(grid) }
}
impl GridSelectorBounds for (usize, usize) {
    fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position::from(*self))
    }
    fn end_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        let p = Position::from(*self);
        Bound::Excluded(Position { row: p.row + 1, col: p.col + 1 })
    }
}

impl GridSelector for usize {
    type Iter = std::iter::Once<usize>;
    fn iter(self, _: &Grid<impl Sized>) -> Self::Iter { std::iter::once(self) }
}
impl GridSelectorBounds for usize {
    fn start_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position { row: self / grid.width, col: self % grid.width })
    }
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        let pos = Position { row: self / grid.width, col: self % grid.width };
        Bound::Excluded(Position { row: pos.row + 1, col: pos.col + 1 })
    }
}

#[derive(Clone, Copy, Debug)] pub struct Row(pub usize);
#[derive(Clone, Copy, Debug)] pub struct Col(pub usize);

impl GridSelectorBounds for Row {
    fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position { row: self.0, col: 0 })
    }
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(Position { row: self.0 + 1, col: grid.width })
    }
}
impl GridSelectorBounds for Col {
    fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Included(Position { row: 0, col: self.0 })
    }
    fn end_bound(&self, grid: &Grid<impl Sized>) -> Bound<Position> {
        Bound::Excluded(Position { row: grid.height, col: self.0 + 1 })
    }
}

impl<R: RangeBounds<Position> + Copy> GridSelectorBounds for R {
    fn start_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> { self.start_bound() }
    fn end_bound(&self, _: &Grid<impl Sized>) -> Bound<Position> { self.end_bound() }
}
impl<R: RangeBounds<Position> + Copy> GridSelector for R {
    type Iter = BoundsIter;
    fn iter(self, grid: &Grid<impl Sized>) -> Self::Iter {
        self.into_bounds(grid).clip_to_grid(grid).iter(grid)
    }
}

// ============================================================================
// Row slice iterators
// ============================================================================

pub struct RowSlices<'a, T> {
    iter: std::iter::Take<std::iter::Skip<std::slice::ChunksExact<'a, T>>>,
    col_range: std::ops::Range<usize>,
}
impl<'a, T> Iterator for RowSlices<'a, T> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|row| &row[self.col_range.clone()])
    }
}

pub struct RowSlicesMut<'a, T> {
    iter: std::iter::Take<std::iter::Skip<std::slice::ChunksExactMut<'a, T>>>,
    col_range: std::ops::Range<usize>,
}
impl<'a, T> Iterator for RowSlicesMut<'a, T> {
    type Item = &'a mut [T];
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|row| {
            let range = self.col_range.clone();
            &mut row[range]
        })
    }
}

// ============================================================================
// Column iterators (read-only)
// ============================================================================

pub struct ColIter<'a, T> {
    grid: &'a Grid<T>,
    row: usize,
    row_end: usize,
    col: usize,
}
impl<'a, T> Iterator for ColIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.row_end { return None; }
        let idx = self.row * self.grid.width + self.col;
        self.row += 1;
        Some(&self.grid.inner[idx])
    }
}

pub struct Cols<'a, T> {
    grid: &'a Grid<T>,
    bounds: Bounds,
    cur_col: usize,
}
impl<'a, T> Iterator for Cols<'a, T> {
    type Item = ColIter<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_col >= self.bounds.max.col { return None; }
        let col = self.cur_col;
        self.cur_col += 1;
        Some(ColIter {
            grid: self.grid,
            row: self.bounds.min.row,
            row_end: self.bounds.max.row,
            col,
        })
    }
}

// ============================================================================
// SubGrid views
// ============================================================================

pub struct SubGrid<'a, T> {
    grid: &'a Grid<T>,
    bounds: Bounds,
}

pub struct SubGridMut<'a, T> {
    grid: &'a mut Grid<T>,
    bounds: Bounds,
}

impl<'a, T> SubGrid<'a, T> {
    #[inline] pub fn bounds(&self) -> Bounds { self.bounds }
    pub fn iter(&self) -> BoundsIter {
        BoundsIter::new(self.bounds, self.grid.width)
    }
    pub fn rows(&self) -> RowSlices<'a, T> { self.grid.rows(self.bounds) }
    pub fn cols(&self) -> Cols<'a, T> { self.grid.cols(self.bounds) }
}

impl<'a, T> SubGridMut<'a, T> {
    #[inline] pub fn bounds(&self) -> Bounds { self.bounds }
    pub fn rows_mut(&mut self) -> RowSlicesMut<'_, T> {
        self.grid.rows_mut(self.bounds)
    }
}

// IntoIterator for SubGrid (row-major)
impl<'a, T> IntoIterator for SubGrid<'a, T> {
    type Item = &'a T;
    type IntoIter = SubGridIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        SubGridIter {
            grid: self.grid,
            iter: BoundsIter::new(self.bounds, self.grid.width),
        }
    }
}

pub struct SubGridIter<'a, T> {
    grid: &'a Grid<T>,
    iter: BoundsIter,
}
impl<'a, T> Iterator for SubGridIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|idx| &self.grid.inner[idx])
    }
}
```

---

# ✅ Usage Examples

```rust
let rect = (2,2)..(5,6);

for row in grid.rows(rect) { /* contiguous slices */ }

for col in grid.cols(rect) { 
    for item in col { /* column values */ }
}

let view = grid.select((1,1)..(3,4));
for v in view { /* row-major iteration of &T */ }
```

---

# ✅ Want more?

If you want:

- mutable column iterators (unsafe but possible)
- a `SubGrid2D` that supports index `(r,c)` directly
- SIMD iteration or custom stride
- parallel iterators

Just ask.
