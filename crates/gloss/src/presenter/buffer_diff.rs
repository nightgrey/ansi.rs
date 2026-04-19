//! Zero-allocation iterator over contiguous runs of changed cells.
//!
//! Compared to [`crate::buffer::BufferDiff`] (which yields one entry per
//! changed cell), this walks in row-major order and groups consecutive
//! changes into a single [`Run`]. That shape maps directly onto what
//! a terminal presenter wants: one cursor-move per run, then a straight
//! run of cell emissions.
//!
//! The iterator is designed for steady-state redraw paths, where most of
//! the frame is unchanged. Inside each row a blockwise coarse skip jumps
//! past clean 32-cell stretches in one slice comparison before falling
//! back to cell-level scanning at the first difference.

use crate::{Buffer, Cell};
use derive_more::{AsRef, Deref};
use std::iter::{Enumerate, FilterMap, FlatMap, FusedIterator};
use std::ops::Range;

#[derive(Debug, Clone, Copy, AsRef, Deref)]
pub struct Change<T> {
    pub row: usize,
    pub col: usize,
    #[as_ref]
    #[deref]
    pub value: T,
}

/// A contiguous range of changed cells within a single row.
///
/// `cells` is a borrow into the `next` buffer covering columns `[x, x + cells.len())`.
/// Zero-width cells (wide-char continuations, empty cells) that sit inside
/// the run are absorbed into the slice: they never *start* or *extend* a
/// run on their own, but once a run is open they ride along so the caller
/// can emit the range without gaps.
type Run<'a> = Change<&'a [Cell]>;

impl Run<'_> {
    /// Exclusive end column — the first column past the run.
    #[inline]
    pub fn end(&self) -> usize {
        self.col + self.value.len()
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.col
    }

    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

type Cellular<'a> = Change<&'a Cell>;

impl Cellular<'_> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl PartialEq<Range<usize>> for Run<'_> {
    fn eq(&self, other: &Range<usize>) -> bool {
        PartialEq::eq(&(self.start()..self.end()), other)
    }
}

/// Iterator yielding [`Run`]s between two buffers in row-major order.
///
/// # Allocation
///
/// None. The iterator holds only slice references and two integer cursors.
///
/// # Panics
///
/// `new` panics if the two buffers have different widths. If heights differ
/// the iterator walks the overlapping region.
#[derive(Debug)]
pub struct Diff<'p, 'n> {
    prev: &'p [Cell],
    next: &'n [Cell],
    width: usize,
    height: usize,
    row: usize,
    /// Resume column within the current row. Always `0` after a row roll-over.
    col: usize,
}

impl<'p, 'n> Diff<'p, 'n> {
    pub fn new(prev: &'p Buffer, next: &'n Buffer) -> Self {
        assert_eq!(
            prev.width, next.width,
            "buffers must have the same width: prev={}, next={}",
            prev.width, next.width,
        );

        let height = prev.height.min(next.height);
        Self {
            prev: prev.as_ref(),
            next: next.as_ref(),
            width: prev.width,
            height,
            row: 0,
            col: 0,
        }
    }

    /// Flatten into per-cell changes.
    ///
    /// Zero-width continuation cells inside a run are dropped so each yielded tuple corresponds to
    /// one emittable cell.
    pub fn cellular(self) -> impl Iterator<Item = Cellular<'n>> {
        self.flat_map(|run| {
            let row = run.row;
            let col = run.col;

            run.iter().enumerate().filter_map(move |(offset, cell)| {
                if cell.is_none() {
                    None
                } else {
                    Some(Cellular {
                        row,
                        col: col + offset,
                        value: cell,
                    })
                }
            })
        })
    }

    /// Upper bound on remaining cells (not runs). Useful for reserving
    /// presenter output buffers.
    pub fn remaining(&self) -> usize {
        let consumed = self.row * self.width + self.col;
        (self.width * self.height).saturating_sub(consumed)
    }

    /// Find the first column `>= from` where `prev` and `next` differ on an
    /// emittable (width > 0) cell. Zero-width cells are transparent — they're
    /// implicitly redrawn by their base cell so style-only changes on a
    /// continuation don't register as a diff.
    #[inline]
    fn find_start(prev: &[Cell], next: &[Cell], from: usize) -> Option<usize> {
        debug_assert_eq!(prev.len(), next.len());
        let len = prev.len();
        let mut i = from;

        /// Coarse block size for the row-internal skip. A 32-cell block is roughly
        /// 2 cache lines at the current `Cell` size; large enough that the whole-slice
        /// equality amortises its overhead, small enough that dense-change rows
        /// don't thrash it.
        const BLOCK: usize = 32;

        // Coarse skip: hop whole matching blocks. Slice equality on `[Cell]`
        // bottoms out in `Cell::eq`, which LLVM auto-vectorises across the block.
        while i + BLOCK <= len {
            if prev[i..i + BLOCK] != next[i..i + BLOCK] {
                break;
            }
            i += BLOCK;
        }

        // Fine scan: advance by cell width, skipping zero-width cells.
        while i < len {
            let w = next[i].width() as usize;
            if w == 0 {
                i += 1;
                continue;
            }
            if prev[i] != next[i] {
                return Some(i);
            }
            i += w;
        }
        None
    }

    /// Extend a run forward from `start` (known to differ) until the first
    /// emittable cell that matches. Zero-width cells are absorbed.
    #[inline]
    fn find_end(prev: &[Cell], next: &[Cell], start: usize) -> usize {
        debug_assert!(start < prev.len());
        let len = prev.len();

        // `find_start` only returns positions where `next[start]` is
        // emittable (width ≥ 1), so this max() is belt-and-braces.
        let step = (next[start].width() as usize).max(1);
        let mut i: usize = start + step;

        while i < len {
            if next[i].is_continuation() {
                // Absorb continuation/empty into the run.
                i += 1;
                continue;
            }
            if prev[i] == next[i] {
                return i;
            }
            i += next[i].width() as usize;
        }

        eprintln!("next[{}] (return)", i);

        i
    }
}

impl<'n> Iterator for Diff<'_, 'n> {
    type Item = Run<'n>;

    fn next(&mut self) -> Option<Run<'n>> {
        let w = self.width;

        while self.row < self.height {
            let start = self.row * w;
            let prev = &self.prev[start..start + w];
            let next = &self.next[start..start + w];

            if let Some(start) = Diff::find_start(prev, next, self.col) {
                let end = Diff::find_end(prev, next, start);

                self.col = end;

                return Some(Run {
                    row: self.row,
                    col: start,
                    value: &next[start..end],
                });
            }

            self.row += 1;
            self.col = 0;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining()))
    }
}

impl FusedIterator for Diff<'_, '_> {}

pub fn diff<'prev, 'next>(prev: &'prev Buffer, next: &'next Buffer) -> Diff<'prev, 'next> {
    Diff::new(prev, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arena, Buffer};
    use ansi::{Color, Style};

    fn test(prev: &Buffer, next: &Buffer) -> Vec<(usize, usize, usize)> {
        diff(prev, next).map(|r| (r.col, r.row, r.len())).collect()
    }

    #[test]
    fn identical_buffers_yield_no_runs() {
        let mut arena = Arena::new();
        let buf = Buffer::from_lines(["hello"], &mut arena);
        assert!(Diff::new(&buf, &buf).next().is_none());
    }

    #[test]
    fn single_cell_change_is_a_run_of_one() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["hello"], &mut arena);
        let next = Buffer::from_lines(["hallo"], &mut arena);

        let got = test(&prev, &next);
        assert_eq!(got, vec![(1, 0, 1)]);
    }

    #[test]
    fn consecutive_changes_merge_into_one_run() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["hello world"], &mut arena);
        let next = Buffer::from_lines(["hXYZo world"], &mut arena);

        let got = test(&prev, &next);
        assert_eq!(got, vec![(1, 0, 3)]);
    }

    #[test]
    fn gap_of_one_matching_cell_splits_runs() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["abcde"], &mut arena);
        let next = Buffer::from_lines(["AbCdE"], &mut arena);

        let got = test(&prev, &next);
        assert_eq!(got, vec![(0, 0, 1), (2, 0, 1), (4, 0, 1)]);
    }

    #[test]
    fn multi_row_runs_are_row_major() {
        let mut arena = Arena::new();
        let prev = Buffer::new(4, 3);
        let mut next = Buffer::new(4, 3);
        next.set_string(0..2, "ab", &mut arena); // row 0, cols 0-1
        next.set_string(4 + 2..4 + 4, "cd", &mut arena); // row 1, cols 2-3;
        next.set_string(8..8 + 1, "e", &mut arena); // row 2, col 0
        let got = test(&prev, &next);
        assert_eq!(got, vec![(0, 0, 2), (2, 1, 2), (0, 2, 1)]);
    }

    #[test]
    fn wide_char_run_includes_continuation() {
        let mut arena = Arena::new();
        let prev = Buffer::new(4, 1);
        let mut next = Buffer::new(4, 1);
        next.set_string(0..4, "中X", &mut arena); // [中, CONT, X, EMPTY]

        let mut iter = diff(&prev, &next);
        let run = iter.next().expect("first run");

        // Run should span columns 0..=2 (中 + CONT + X); EMPTY at col 3
        // is zero-width and matches prev's empty cell so it terminates the run.
        assert_eq!(run.range(), 0..3);
        assert_eq!(run[0].width(), 2);
        assert_eq!(run[1].width(), 0); // CONT absorbed
        assert_eq!(run[2].width(), 1);
        assert!(iter.next().is_none());
    }

    #[test]
    fn styled_continuation_alone_is_not_a_run() {
        // Continuation-cell style changes with the base wide char unchanged
        // must not produce a run — the base cell implicitly redraws them.
        let mut arena = Arena::new();
        let mut prev = Buffer::new(2, 1);
        prev.set_string(0..2, "中", &mut arena);
        let mut next = prev.clone();
        next[1] = next[1].with_style(Style::default().foreground(Color::Red));

        assert_eq!(next[1].width(), 0);
        assert!(Diff::new(&prev, &next).next().is_none());
    }

    #[test]
    fn wide_to_narrow_emits_both_cells_in_a_run() {
        let mut arena = Arena::new();
        let mut prev = Buffer::new(2, 1);
        prev.set_string(0..2, "中", &mut arena);
        let next = Buffer::from_lines(["ab"], &mut arena);

        let got = test(&prev, &next);
        assert_eq!(got, vec![(0, 0, 2)]);
    }

    #[test]
    fn narrow_to_wide_emits_single_run_of_width_2() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["ab"], &mut arena);
        let mut next = Buffer::new(2, 1);
        next.set_string(0..2, "中", &mut arena);

        let got = test(&prev, &next);
        // [中, CONT] — both positions emitted as a single run.
        assert_eq!(got, vec![(0, 0, 2)]);
    }

    #[test]
    fn mismatched_widths_panic() {
        let prev = Buffer::new(4, 1);
        let next = Buffer::new(8, 1);
        let result = std::panic::catch_unwind(|| Diff::new(&prev, &next));
        assert!(result.is_err());
    }

    #[test]
    fn different_heights_walk_overlap_only() {
        let mut arena = Arena::new();
        let prev = Buffer::new(3, 1);
        let mut next = Buffer::new(3, 3);
        next.set_string(0..3, "abc", &mut arena); // row 0 — in overlap
        next.set_string(3..6, "xyz", &mut arena); // row 1 — past prev height

        let got = test(&prev, &next);
        // Only the first row is compared; row 1's changes are invisible.
        assert_eq!(got, vec![(0, 0, 3)]);
    }

    #[test]
    fn coarse_block_skip_does_not_miss_late_changes() {
        // Buffer wider than BLOCK (32). Only the very last cell differs, so
        // the blockwise skip must NOT swallow it.
        let mut arena = Arena::new();
        let width = 40;
        let prev = Buffer::from_lines([" ".repeat(width).as_str()], &mut arena);
        let mut next = prev.clone();
        next[width - 1] = Cell::inline('X', Style::None);

        let got = test(&prev, &next);
        assert_eq!(got, vec![((width - 1), 0, 1)]);
    }

    #[test]
    fn coarse_block_skip_skips_clean_prefix_before_change() {
        // Ensure a long clean prefix is skipped by the block path and the
        // change in the tail block is still reported correctly.
        let mut arena = Arena::new();
        let width = 64;
        let prev = Buffer::from_lines([" ".repeat(width).as_str()], &mut arena);
        let mut next = prev.clone();
        next[50] = Cell::inline('Q', Style::None);
        next[51] = Cell::inline('R', Style::None);

        let got = test(&prev, &next);
        assert_eq!(got, vec![(50, 0, 2)]);
    }

    #[test]
    fn cells_adapter_matches_classic_per_cell_shape() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["hello"], &mut arena);
        let next = Buffer::from_lines(["HeLLo"], &mut arena);

        let per_cell: Vec<(usize, usize)> = Diff::new(&prev, &next)
            .cellular()
            .map(|change| (change.col, change.row))
            .collect();
        assert_eq!(per_cell, vec![(0, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn cells_adapter_drops_continuations() {
        // Wide char at col 0 → run covers cols 0..=1, but `cells()` should
        // only yield the base cell (the continuation is width-0 and redrawn
        // implicitly).
        let mut arena = Arena::new();
        let prev = Buffer::new(3, 1);
        let mut next = Buffer::new(3, 1);
        next.set_string(0..2, "中", &mut arena);

        let per_cell: Vec<(usize, usize)> = Diff::new(&prev, &next)
            .cellular()
            .map(|change| (change.col, change.row))
            .collect();
        assert_eq!(per_cell, vec![(0, 0)]);
    }

    #[test]
    fn fused_after_exhaustion() {
        let buf = Buffer::new(2, 1);
        let mut iter = Diff::new(&buf, &buf);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
