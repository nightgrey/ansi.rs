use super::Cells;
use crate::buffer::{Buffer, Cell};
use crate::{CellsIter, Map, TrackingBuffer};
use derive_more::{AsRef, Deref};
use geometry::Point;
use std::iter::FusedIterator;

/// A zero-allocation buffer diffing iterator
///
/// # Zero-width cells
/// Trailing positions of a wide grapheme are skipped: they are implicitly redrawn by the wide base
/// cell, so diffing them separately would produce redundant updates.
///
/// # Differing heights
/// When the two buffers have different heights, the iterator only walks the
/// overlapping region (the shorter of the two).
///
/// # Known limitation: VS16 emoji presentation
/// Some terminals fail to clear the continuation cell when re-rendering an
/// emoji that uses the variation selector U+FE0F (VS16). This iterator does
/// not carry the arena needed to inspect grapheme bytes, so it cannot detect
/// that case. If you target terminals with this quirk, emit an explicit clear
/// of the trailing cell after each wide emoji update in a post-pass.
#[derive(Debug)]
pub struct BufferDiff<'a, Strategy: DiffStrategy<'a> = ByCells> {
    state: Strategy::State,
    strategy: Strategy,
}

impl<'a> BufferDiff<'a, ByCells> {
    /// A zero-allocation diffing iterator over changed cells.
    pub fn cells(prev: &'a Buffer, next: &'a Buffer) -> Self {
        BufferDiff {
            state: BaseDiffState::new(prev, next),
            strategy: ByCells,
        }
    }

    pub fn into_runs(self) -> BufferDiff<'a, ByRuns> {
        BufferDiff {
            state: self.state,
            strategy: ByRuns,
        }
    }
}

impl<'a> BufferDiff<'a, ByRuns> {
    /// A zero-allocation diffing iterator over changed runs of consecutive cells on the same row.
    ///
    /// See [`BufferDiff`] and the [`ByRuns`] [`DiffStrategy`] for details.
    pub fn runs(prev: &'a Buffer, next: &'a Buffer) -> Self {
        BufferDiff {
            state: BaseDiffState::new(prev, next),
            strategy: ByRuns,
        }
    }

    pub fn into_cells(self) -> BufferDiff<'a, ByCells> {
        BufferDiff {
            state: self.state,
            strategy: ByCells,
        }
    }
}

impl<'a> BufferDiff<'a, ByDirty> {
    /// A zero-allocation diffing iterator over changed rows.
    /// See [`BufferDiff`] and the [`ByDirty`] for details.
    pub fn dirty(prev: &'a Buffer, next: &'a TrackingBuffer) -> Self {
        assert_eq!(
            prev.width, next.width,
            "buffers must have the same width: prev={}, next={}",
            prev.width, next.width,
        );

        BufferDiff {
            state: DirtyDiffState {
                next,
                prev,
                width: prev.width,
                height: prev.height.min(next.height),
                y: 0,
                x: 0,
                bits: next.as_bits(),
            },
            strategy: ByDirty,
        }
    }
}

impl<'a, Strategy: DiffStrategy<'a>> Iterator for BufferDiff<'a, Strategy> {
    type Item = Strategy::Item;

    fn next(&mut self) -> Option<Self::Item> {
        Strategy::next(&mut self.state)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        Strategy::size_hint(&self.state)
    }
}
impl<'a, Strategy: DiffStrategy<'a>> FusedIterator for BufferDiff<'a, Strategy> {}

/// A changed cell.
#[derive(Copy, Clone, Debug, Deref, PartialEq)]
pub struct Change<'a> {
    pub x: u16,
    pub y: u16,
    #[deref]
    pub cell: &'a Cell,
}

impl From<Change<'_>> for Point {
    fn from(change: Change<'_>) -> Self {
        Point {
            x: change.x,
            y: change.y,
        }
    }
}
/// A run of consecutive changed cells on the same row.
///
/// `cells` is the contiguous slice of cells in the run, including any
/// zero-width continuations of wide base cells. Iteration yields only
/// base cells, paired with their absolute x coordinate.
#[derive(Copy, Clone, Debug, AsRef)]
pub struct Run<'a> {
    pub x: u16,
    pub y: u16,
    #[as_ref]
    cells: &'a [Cell],
}

impl<'a> Run<'a> {
    #[inline]
    pub fn iter(&self) -> RunIter<'a> {
        RunIter {
            inner: Cells(self.cells).run(),
            x: self.x,
            y: self.y,
        }
    }

    /// Total column width spanned by this run (sum of base-cell widths).
    ///
    /// For an all-narrow run this equals [`Self::count`]; runs containing
    /// wide cells span more columns than they have base cells.
    #[inline]
    pub fn width(&self) -> u16 {
        self.cells.iter().map(|c| c.width() as u16).sum()
    }

    /// Amount of base cells in this run.
    ///
    /// Counts every non-continuation cell, matching what [`Self::iter`] yields:
    /// cleared cells are zero-width but are *not* continuations, so they count.
    #[inline]
    pub fn count(&self) -> usize {
        self.cells.iter().filter(|c| !c.is_continuation()).count()
    }
}

impl<'a> IntoIterator for Run<'a> {
    type Item = Change<'a>;
    type IntoIter = RunIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Yields the base cells of a [`Run`] as [`Change`]s, in column order.
///
/// A thin wrapper over [`BaseCells`] that offsets each cell's slice-relative
/// column by the run's absolute `x`/`y`.
#[derive(Clone, Debug)]
pub struct RunIter<'a> {
    inner: CellsIter<'a>,
    x: u16,
    y: u16,
}

impl<'a> Iterator for RunIter<'a> {
    type Item = Change<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (col, cell) = self.inner.next()?;
        Some(Change {
            x: self.x + col,
            y: self.y,
            cell,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl FusedIterator for RunIter<'_> {}

/// State of a [`BufferDiff`] iteration.
/// See [`DiffStrategy`].
#[derive(Debug)]
pub struct BaseDiffState<'a> {
    prev: &'a [Cell],
    next: &'a [Cell],
    height: usize,
    width: usize,
    len: usize,
    pos: usize,
}

impl<'a> BaseDiffState<'a> {
    /// Builds the shared state for a [`BufferDiff`] over the overlapping
    /// (shorter) region of two equal-width buffers.
    ///
    /// **Panics** if the buffers differ in width.
    fn new(prev: &'a Buffer, next: &'a Buffer) -> Self {
        assert_eq!(
            prev.width, next.width,
            "buffers must have the same width: prev={}, next={}",
            prev.width, next.width,
        );

        let width = prev.width;
        let height = prev.height.min(next.height);

        BaseDiffState {
            next,
            prev,
            width,
            height,
            len: width * height,
            pos: 0,
        }
    }
}

/// State of a [`BufferDiff`] iteration.
/// See [`DiffStrategy`].
#[derive(Debug)]
pub struct DirtyDiffState<'a> {
    prev: &'a [Cell],
    next: &'a [Cell],
    bits: &'a Map,
    height: usize,
    width: usize,
    y: usize,
    x: usize,
}

impl<'a> DirtyDiffState<'a> {
    pub fn is_marked(&self, y: usize) -> bool {
        self.bits.contains(y as u64)
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Strategy for iterating over the differences between two buffers.
///
/// This trait is sealed: only [`ByCells`], [`ByRuns`], and [`ByDirty`]
/// implement it.
pub trait DiffStrategy<'a>: Default + sealed::Sealed {
    /// The type of items yielded by the diff.
    type Item;
    /// The state of the diffing operation.
    type State;

    /// Advances the diff and returns the next item.
    fn next(state: &mut Self::State) -> Option<Self::Item>;
    /// Returns an estimate of the number of items remaining in the iterator.
    fn size_hint(state: &Self::State) -> (usize, Option<usize>);
}

/// Diffs [`Change`]s for each cell that is different.
/// See [`DiffStrategy`].
#[derive(Debug, Default)]
pub struct ByCells;

impl sealed::Sealed for ByCells {}
impl<'a> DiffStrategy<'a> for ByCells {
    type Item = Change<'a>;
    type State = BaseDiffState<'a>;

    #[inline]
    fn next(state: &mut Self::State) -> Option<Self::Item> {
        while state.pos < state.len {
            let row_end = state.pos - (state.pos % state.width) + state.width;

            // At a row boundary, skip the whole row if it's unchanged. The
            // slice equality lowers to a tight loop (or memcmp) over the row's
            // cells, which is far faster than walking cell-by-cell.
            if state.pos % state.width == 0
                && state.next[state.pos..row_end] == state.prev[state.pos..row_end]
            {
                state.pos = row_end;
                continue;
            }

            while state.pos < row_end {
                let i = state.pos;
                let current = &state.next[i];

                // Continuations are the trailing positions of a wide grapheme,
                // redrawn implicitly by that base cell. Empty cells are cleared
                // cells, not continuations, and must be reported as changes.
                if current.is_continuation() {
                    state.pos += 1;
                    continue;
                }

                state.pos += current.advance();

                if current != &state.prev[i] {
                    let x = (i % state.width) as u16;
                    let y = (i / state.width) as u16;
                    return Some(Change {
                        x,
                        y,
                        cell: current,
                    });
                }
            }
        }
        None
    }

    #[inline]
    fn size_hint(state: &BaseDiffState<'a>) -> (usize, Option<usize>) {
        (0, Some(state.len.saturating_sub(state.pos)))
    }
}

#[derive(Debug, Default)]
pub struct ByDirty;

impl sealed::Sealed for ByDirty {}
impl<'a> DiffStrategy<'a> for ByDirty {
    type Item = Change<'a>;
    type State = DirtyDiffState<'a>;

    fn next(state: &mut Self::State) -> Option<Self::Item> {
        while state.y < state.height {
            if state.x == 0 {
                // Skip non-dirty rows in O(1).
                if !state.is_marked(state.y) {
                    state.y += 1;
                    continue;
                }
                // Even within a dirty row, a slice-equality fast-path beats
                // walking cells when the row was marked conservatively
                // (e.g. a re-render that produced the same content).
                let row_start = state.y * state.width;
                let row_end = row_start + state.width;
                if state.next[row_start..row_end] == state.prev[row_start..row_end] {
                    state.y += 1;
                    continue;
                }
            }

            let row_start = state.y * state.width;
            while state.x < state.width {
                let i = row_start + state.x;
                let cell = &state.next[i];
                if cell.is_continuation() {
                    // Trailing continuation of a wide cell — implicitly
                    // redrawn by its base. Empty cells are cleared cells and
                    // must still be reported.
                    state.x += 1;
                    continue;
                }
                state.x += cell.advance();
                if *cell != state.prev[i] {
                    return Some(Change {
                        x: (i % state.width) as u16,
                        y: (i / state.width) as u16,
                        cell,
                    });
                }
            }
            state.y += 1;
            state.x = 0;
        }
        None
    }

    fn size_hint(state: &Self::State) -> (usize, Option<usize>) {
        let remaining = state.height.saturating_sub(state.y) * state.width;
        (0, Some(remaining))
    }
}

/// Diffs [`Run`]s of consecutive changed cells on the same row.
#[derive(Debug, Default)]
pub struct ByRuns;

impl sealed::Sealed for ByRuns {}
impl<'a> DiffStrategy<'a> for ByRuns {
    type Item = Run<'a>;
    type State = BaseDiffState<'a>;

    #[inline]
    fn next(state: &mut BaseDiffState<'a>) -> Option<Self::Item> {
        // Scan to the first base cell that differs, fast-skipping whole rows
        // that are unchanged.
        let start = loop {
            if state.pos >= state.len {
                return None;
            }

            if state.pos.is_multiple_of(state.width) {
                let row_end = state.pos + state.width;
                if state.next[state.pos..row_end] == state.prev[state.pos..row_end] {
                    state.pos = row_end;
                    continue;
                }
            }

            let i = state.pos;
            let cell = &state.next[i];

            if cell.is_continuation() {
                // Trailing continuation — implicitly redrawn by its base.
                state.pos += 1;
                continue;
            }

            state.pos += cell.advance();
            if cell != &state.prev[i] {
                break i;
            }
        };

        let start_x = (start % state.width) as u16;
        let start_y = (start / state.width) as u16;
        let row_end = (start_y as usize + 1) * state.width;

        // Extend within the row, absorbing continuations of the running base
        // cell and stopping at the first base cell that matches `prev`.
        while state.pos < row_end {
            let j = state.pos;
            let cell = &state.next[j];

            if cell.is_continuation() {
                // Trailing continuation — implicitly redrawn by its base.
                state.pos += 1;
                continue;
            }

            if cell == &state.prev[j] {
                break;
            }

            state.pos += cell.advance();
        }

        // Cap at `row_end` defensively: a malformed buffer with a wide cell
        // straddling a row boundary could overshoot, which would otherwise
        // bleed cells from the next row into the run.
        let end = state.pos.min(row_end);
        Some(Run {
            x: start_x,
            y: start_y,
            cells: &state.next[start..end],
        })
    }

    #[inline]
    fn size_hint(state: &BaseDiffState<'a>) -> (usize, Option<usize>) {
        (0, Some(state.len.saturating_sub(state.pos)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arena;

    mod by_runs {
        use super::*;

        #[test]
        fn empty_buffers_yield_no_diffs() {
            let buf = Buffer::new(5, 1);
            let diff: Vec<_> = BufferDiff::runs(&buf, &buf).collect();
            assert!(diff.is_empty());
            assert_eq!(diff.iter().size_hint(), (0, Some(0)));
        }

        #[test]
        fn identical_buffers_yield_no_diffs() {
            let mut arena = Arena::new();
            let buf = Buffer::from_lines(["hello"], &mut arena);
            let diff: Vec<_> = BufferDiff::runs(&buf, &buf).collect();
            assert!(diff.is_empty());
        }

        #[test]
        fn single_cell_change() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["hello"], &mut arena);
            let next = Buffer::from_lines(["hallo"], &mut arena);
            let mut runs = BufferDiff::runs(&prev, &next);
            let mut run = runs.next().unwrap().iter();
            assert_eq!(run.size_hint(), (0, Some(1)));
            let change = run.next().unwrap();
            assert_eq!((change.x, change.y, change.as_str(&arena)), (1, 0, "a"));
            assert_eq!(run.next(), None);
        }

        #[test]
        fn all_cells_changed() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["aaa"], &mut arena);
            let next = Buffer::from_lines(["bbb"], &mut arena);
            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();
            let one = runs[0].iter();

            assert_eq!(one.size_hint(), (0, Some(3)));

            assert_eq!(
                one.map(|change| change.as_str(&arena)).collect::<String>(),
                "bbb"
            );
        }

        #[test]
        fn continuation_cells_are_skipped() {
            let mut arena = Arena::new();
            let prev = Buffer::new(4, 1);
            let mut next = Buffer::new(4, 1);
            // Layout: [中, CONT, X, SPACE] — the wide 中 has a continuation at x=1.
            next.set_string(0..4, "中X", &mut arena);

            let diff: Vec<_> = BufferDiff::runs(&prev, &next)
                .next()
                .unwrap()
                .iter()
                .collect();

            // Only the two base cells (x=0, x=2) differ; the continuation at x=1
            // is not emitted.
            assert_eq!(diff.len(), 2);
            assert_eq!((diff[0].x, diff[0].y), (0, 0));
            assert_eq!((diff[1].x, diff[1].y), (2, 0));
        }

        #[test]
        fn equal_cell_splits_run() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["abcde"], &mut arena);
            let next = Buffer::from_lines(["AbCde"], &mut arena);

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();

            // 'b' at x=1 matches, splitting [A] and [C].
            assert_eq!(runs.len(), 2);
            assert_eq!((runs[0].x, runs[0].y), (0, 0));
            assert_eq!((runs[1].x, runs[1].y), (2, 0));
            assert_eq!(
                runs[0].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "A"
            );
            assert_eq!(
                runs[1].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "C"
            );
        }

        #[test]
        fn adjacent_changes_merge_into_one_run() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["abcde"], &mut arena);
            let next = Buffer::from_lines(["aBCDe"], &mut arena);

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();

            assert_eq!(runs.len(), 1);
            assert_eq!((runs[0].x, runs[0].y), (1, 0));
            assert_eq!(
                runs[0].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "BCD"
            );
        }

        #[test]
        fn runs_do_not_cross_rows() {
            let mut arena = Arena::new();
            let prev = Buffer::new(3, 2);
            let next = Buffer::from_lines(["abc", "def"], &mut arena);

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();

            // Even though every cell differs, the run terminates at the row
            // boundary so we get one run per row.
            assert_eq!(runs.len(), 2);
            assert_eq!((runs[0].x, runs[0].y), (0, 0));
            assert_eq!((runs[1].x, runs[1].y), (0, 1));
            assert_eq!(
                runs[0].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "abc"
            );
            assert_eq!(
                runs[1].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "def"
            );
        }

        #[test]
        fn run_at_right_edge() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["abcde"], &mut arena);
            let next = Buffer::from_lines(["abcDE"], &mut arena);

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();

            assert_eq!(runs.len(), 1);
            assert_eq!((runs[0].x, runs[0].y), (3, 0));
            assert_eq!(
                runs[0].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "DE"
            );
        }

        #[test]
        fn wide_cell_inside_run_is_iterated_correctly() {
            let mut arena = Arena::new();
            let prev = Buffer::new(5, 1);
            let mut next = Buffer::new(5, 1);
            // Layout: [a, 中, CONT, b, SPACE]
            next.set_string(0..5, "a中b", &mut arena);

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();
            assert_eq!(runs.len(), 1);
            let run = runs[0];
            assert_eq!((run.x, run.y), (0, 0));

            let bases: Vec<_> = run.iter().collect();
            // Continuation at x=2 is skipped; bases are at x=0, 1, 3.
            assert_eq!(bases.len(), 3);
            assert_eq!((bases[0].x, bases[0].as_str(&arena)), (0, "a"));
            assert_eq!((bases[1].x, bases[1].as_str(&arena)), (1, "中"));
            assert_eq!((bases[2].x, bases[2].as_str(&arena)), (3, "b"));
        }

        #[test]
        fn different_heights_use_minimum() {
            let mut arena = Arena::new();
            let prev = Buffer::new(3, 1);
            let mut next = Buffer::new(3, 3);
            next.set_string(0..3, "abc", &mut arena);
            next.set_string(3..6, "xyz", &mut arena); // row 1 — beyond prev's height

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();

            assert_eq!(runs.len(), 1);
            assert_eq!((runs[0].x, runs[0].y), (0, 0));
        }

        #[test]
        #[should_panic(expected = "buffers must have the same width")]
        fn mismatched_widths_panic() {
            let prev = Buffer::new(5, 1);
            let next = Buffer::new(10, 1);
            let _ = BufferDiff::runs(&prev, &next);
        }

        #[test]
        fn exhausted_iterator_stays_none() {
            let buf = Buffer::new(2, 1);
            let mut iter = BufferDiff::runs(&buf, &buf);
            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        }

        #[test]
        fn size_hint_is_an_upper_bound() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["aaaaa"], &mut arena);
            let next = Buffer::from_lines(["bbbbb"], &mut arena);

            let iter = BufferDiff::runs(&prev, &next);
            let (lower, upper) = iter.size_hint();
            assert_eq!(lower, 0);
            // Upper bound is conservative: at most one run per remaining cell.
            assert_eq!(upper, Some(5));
            // Actual run count is one (all cells form a contiguous run).
            assert_eq!(iter.count(), 1);
        }

        #[test]
        fn buffer_diff_runs_method_matches_free_function() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["hello"], &mut arena);
            let next = Buffer::from_lines(["hallo"], &mut arena);

            let via_method: Vec<(u16, u16)> = Buffer::diff_runs(&prev, &next)
                .map(|run| (run.x, run.y))
                .collect();
            let via_ctor: Vec<(u16, u16)> = BufferDiff::runs(&prev, &next)
                .map(|run| (run.x, run.y))
                .collect();

            assert_eq!(via_method, via_ctor);
        }

        #[test]
        fn by_runs_conversion_round_trips() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["abc"], &mut arena);
            let next = Buffer::from_lines(["AbC"], &mut arena);

            // ByCells reports the same number of base-cell changes that the
            // flattened ByRuns iteration does.
            let cell_count = BufferDiff::cells(&prev, &next).count();
            let runs_count: usize = BufferDiff::runs(&prev, &next)
                .map(|run| run.iter().count())
                .sum();

            assert_eq!(cell_count, runs_count);
        }

        #[test]
        fn run_as_ref_exposes_cells() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["abc"], &mut arena);
            let next = Buffer::from_lines(["xyz"], &mut arena);

            let run = BufferDiff::runs(&prev, &next).next().unwrap();
            let cells: &[Cell] = run.as_ref();
            assert_eq!(cells.len(), 3);
            assert_eq!(cells[0].as_str(&arena), "x");
            assert_eq!(cells[2].as_str(&arena), "z");
        }

        #[test]
        fn runner_skips_orphan_continuations() {
            // Force a slice that begins with a continuation to exercise the
            // zero-width skip branch in `Runner::next`. This shouldn't happen
            // with a well-formed buffer, but the iterator should remain robust.
            let cells = [Cell::CONTINUATION, Cell::inline('a')];
            let run = Run {
                x: 5,
                y: 2,
                cells: &cells,
            };
            let collected: Vec<_> = run.iter().map(|c| (c.x, c.y)).collect();
            assert_eq!(collected, vec![(5, 2)]);
        }

        #[test]
        fn runner_size_hint_lower_bound_is_zero() {
            // A slice consisting entirely of continuations has zero base cells,
            // so the lower bound must be 0 — not the current index.
            let cells = [Cell::CONTINUATION, Cell::CONTINUATION];
            let run = Run {
                x: 0,
                y: 0,
                cells: &cells,
            };
            assert_eq!(run.iter().size_hint(), (0, Some(2)));
        }

        #[test]
        fn run_into_iter_matches_iter() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["abc"], &mut arena);
            let next = Buffer::from_lines(["xyz"], &mut arena);

            let run = BufferDiff::runs(&prev, &next).next().unwrap();
            let via_iter: Vec<_> = run.iter().map(|c| c.x).collect();
            let via_into: Vec<_> = run.into_iter().map(|c| c.x).collect();
            assert_eq!(via_iter, via_into);
        }

        #[test]
        fn count_includes_cleared_cells_and_matches_iter() {
            // Regression: `count` used to filter on `width() > 0`, which dropped
            // cleared cells. A cleared cell (`Cell::EMPTY`) is zero-width but is
            // NOT a continuation, so `iter` yields it and `count` must too.
            let cells = [Cell::inline('a'), Cell::EMPTY, Cell::CONTINUATION];
            let run = Run {
                x: 0,
                y: 0,
                cells: &cells,
            };
            assert_eq!(run.count(), run.iter().count());
            // 'a' and the cleared cell count; the continuation does not.
            assert_eq!(run.count(), 2);
        }

        #[test]
        fn run_width_and_cell_count() {
            let mut arena = Arena::new();
            let prev = Buffer::new(5, 1);
            let mut next = Buffer::new(5, 1);
            // Layout: [a, 中, CONT, b, SPACE] — 3 base cells spanning 4 columns.
            next.set_string(0..5, "a中b", &mut arena);

            let run = BufferDiff::runs(&prev, &next).next().unwrap();
            assert_eq!(run.count(), 3);
            assert_eq!(run.width(), 4);
        }

        #[test]
        fn fast_skip_does_not_change_results_on_partial_rows() {
            // Mix of identical rows (which the fast-skip should handle) and
            // rows with changes to verify the optimization stays correct.
            let mut arena = Arena::new();
            let mut prev = Buffer::new(4, 4);
            let mut next = Buffer::new(4, 4);
            for y in 0..4 {
                prev.set_string(y * 4..y * 4 + 4, "aaaa", &mut arena);
                next.set_string(y * 4..y * 4 + 4, "aaaa", &mut arena);
            }
            // Mutate one cell on row 1 and a span on row 3.
            next.set_string(1 * 4 + 2..1 * 4 + 3, "X", &mut arena);
            next.set_string(3 * 4..3 * 4 + 3, "YYY", &mut arena);

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();
            assert_eq!(runs.len(), 2);
            assert_eq!((runs[0].x, runs[0].y), (2, 1));
            assert_eq!((runs[1].x, runs[1].y), (0, 3));
            assert_eq!(
                runs[1].iter().map(|c| c.as_str(&arena)).collect::<String>(),
                "YYY"
            );
        }

        #[test]
        fn styled_continuation_does_not_split_run() {
            use ansi::{Color, Style};

            let mut arena = Arena::new();
            let prev = Buffer::new(3, 1);
            let mut next = Buffer::new(3, 1);
            next.set_string(0..3, "中X", &mut arena);
            // Mutate the continuation's style — it remains zero-width and
            // should still be absorbed by its base cell, not break the run.
            next[1] = next[1].with_style(Style::default().foreground(Color::Red));

            let runs: Vec<_> = BufferDiff::runs(&prev, &next).collect();
            assert_eq!(runs.len(), 1);
            let bases: Vec<_> = runs[0].iter().collect();
            assert_eq!(bases.len(), 2);
            assert_eq!((bases[0].x, bases[1].x), (0, 2));
        }
    }

    mod by_cells {
        use super::*;

        #[test]
        fn empty_buffers_yield_no_diffs() {
            let buf = Buffer::new(5, 1);
            let diff: Vec<_> = BufferDiff::cells(&buf, &buf).collect();
            assert!(diff.is_empty());
        }

        #[test]
        fn identical_buffers_yield_no_diffs() {
            let mut arena = Arena::new();
            let buf = Buffer::from_lines(["hello"], &mut arena);
            let diff: Vec<_> = BufferDiff::cells(&buf, &buf).collect();
            assert!(diff.is_empty());
        }

        #[test]
        fn single_cell_change() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["hello"], &mut arena);
            let next = Buffer::from_lines(["hallo"], &mut arena);
            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();
            assert_eq!(diff.len(), 1);
            assert_eq!((diff[0].x, diff[0].y), (1, 0));
            assert_eq!(diff[0].cell.as_str(&arena), "a");
        }

        #[test]
        fn all_cells_changed() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["aaa"], &mut arena);
            let next = Buffer::from_lines(["bbb"], &mut arena);
            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();
            assert_eq!(diff.len(), 3);
        }

        #[test]
        fn continuation_cells_are_skipped() {
            let mut arena = Arena::new();
            let prev = Buffer::new(4, 1);
            let mut next = Buffer::new(4, 1);
            // Layout: [中, CONT, X, SPACE] — the wide 中 has a continuation at x=1.
            next.set_string(0..4, "中X", &mut arena);

            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();

            // Only the two base cells (x=0, x=2) differ; the continuation at x=1
            // is not emitted.
            assert_eq!(diff.len(), 2);
            assert_eq!((diff[0].x, diff[0].y), (0, 0));
            assert_eq!((diff[1].x, diff[1].y), (2, 0));
        }

        #[test]
        fn wide_to_narrow_updates_both_positions() {
            let mut arena = Arena::new();
            let mut prev = Buffer::new(2, 1);
            prev.set_string(0..2, "中", &mut arena);
            let next = Buffer::from_lines(["ab"], &mut arena);

            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();

            // prev[1] was CONT, next[1] is 'b' (narrow) — both differ and both are
            // yielded since neither cell in `next` is a continuation.
            assert_eq!(diff.len(), 2);
            assert_eq!(diff[0].x, 0);
            assert_eq!(diff[1].x, 1);
        }

        #[test]
        fn coordinates_are_row_major() {
            let mut arena = Arena::new();
            let prev = Buffer::new(3, 2);
            let mut next = Buffer::new(3, 2);
            next.set_string(1..2, "x", &mut arena); // (1, 0)
            next.set_string(5..6, "y", &mut arena); // (2, 1)

            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();

            assert_eq!(diff.len(), 2);
            assert_eq!((diff[0].x, diff[0].y), (1, 0));
            assert_eq!((diff[1].x, diff[1].y), (2, 1));
        }

        #[test]
        fn different_heights_use_minimum() {
            let mut arena = Arena::new();
            let prev = Buffer::new(3, 1);
            let mut next = Buffer::new(3, 3);
            next.set_string(0..3, "abc", &mut arena); // row 0
            next.set_string(3..6, "xyz", &mut arena); // row 1 — beyond prev's height

            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();

            assert_eq!(diff.len(), 3);
            assert!(diff.iter().all(|change| change.y == 0));
        }

        #[test]
        #[should_panic(expected = "buffers must have the same width")]
        fn mismatched_widths_panic() {
            let prev = Buffer::new(5, 1);
            let next = Buffer::new(10, 1);
            let _ = BufferDiff::cells(&prev, &next);
        }

        #[test]
        fn size_hint_is_an_upper_bound() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["aaa"], &mut arena);
            let next = Buffer::from_lines(["bbb"], &mut arena);

            let iter = BufferDiff::cells(&prev, &next);
            let (lower, upper) = iter.size_hint();
            assert_eq!(lower, 0);
            assert_eq!(upper, Some(3));
            assert_eq!(iter.count(), 3);
        }

        #[test]
        fn exhausted_iterator_stays_none() {
            let buf = Buffer::new(2, 1);
            let mut iter = BufferDiff::cells(&buf, &buf);
            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        }

        #[test]
        fn buffer_diff_method_matches_free_function() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["hello"], &mut arena);
            let next = Buffer::from_lines(["hallo"], &mut arena);

            let via_method: Vec<Point> = Buffer::diff_cells(&prev, &next).map(Into::into).collect();
            let via_ctor: Vec<Point> = BufferDiff::cells(&prev, &next).map(Into::into).collect();

            assert_eq!(via_method, via_ctor);
        }

        #[test]
        fn styled_zero_width_cells_are_still_skipped() {
            use ansi::{Color, Style};

            let mut arena = Arena::new();
            let prev = Buffer::new(2, 1);
            let mut next = Buffer::new(2, 1);
            next.set_string(0..2, "中", &mut arena);
            // Mutate the continuation's style so it no longer equals Cell::CONTINUATION
            // exactly — the diff should still treat it as zero-width.
            next[1] = next[1].with_style(Style::default().foreground(Color::Red));
            assert_eq!(next[1].width(), 0);

            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();
            assert_eq!(diff.len(), 1);
            assert_eq!((diff[0].x, diff[0].y), (0, 0));
        }

        #[test]
        fn fast_skip_identical_rows_preserves_results() {
            // A multi-row buffer where most rows are identical; only one
            // mid-row change. Exercises the per-row fast-skip path.
            let mut arena = Arena::new();
            let mut prev = Buffer::new(4, 5);
            let mut next = Buffer::new(4, 5);
            for y in 0..5 {
                prev.set_string(y * 4..y * 4 + 4, "....", &mut arena);
                next.set_string(y * 4..y * 4 + 4, "....", &mut arena);
            }
            next.set_string(3 * 4 + 1..3 * 4 + 2, "X", &mut arena);

            let diff: Vec<_> = BufferDiff::cells(&prev, &next).collect();
            assert_eq!(diff.len(), 1);
            assert_eq!((diff[0].x, diff[0].y), (1, 3));
            assert_eq!(diff[0].cell.as_str(&arena), "X");
        }
    }

    mod by_dirty {
        use super::*;

        #[test]
        fn diff_dirty_skips_clean_rows_and_unchanged_dirty_rows() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["aaa", "bbb", "ccc"], &mut arena);
            let mut next =
                TrackingBuffer::from(Buffer::from_lines(["aaa", "bbb", "ccc"], &mut arena));
            next.unmark_all();

            // Re-render row 1 with the same content. Dirty bit set, but slice-eq
            // fast path should still produce no changes.
            next.set_line(Point { x: 0, y: 1 }, "bbb", &mut arena);
            assert!(next.is_marked(1));
            let changes: Vec<_> = next.diff(&prev).collect();
            assert!(changes.is_empty());

            // Mutate row 2 with different content.
            next.set_line(Point { x: 0, y: 2 }, "cCc", &mut arena);
            let changes: Vec<_> = next.diff(&prev).collect();
            assert_eq!(changes.len(), 1);
            assert_eq!((changes[0].x, changes[0].y), (1, 2));
        }

        #[test]
        fn diff_dirty_matches_buffer_diff_when_all_dirty() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["hello", "world"], &mut arena);
            let next_buf = Buffer::from_lines(["hallo", "wXrld"], &mut arena);
            let next = TrackingBuffer::from(next_buf.clone());

            let via_dirty: Vec<_> = next.diff(&prev).map(|c| (c.x, c.y)).collect();
            let via_buffer: Vec<_> = Buffer::diff_cells(&prev, &next_buf)
                .map(|c| (c.x, c.y))
                .collect();

            assert_eq!(via_dirty, via_buffer);
        }

        #[test]
        fn diff_dirty_size_hint_is_upper_bound() {
            let mut arena = Arena::new();
            let prev = Buffer::from_lines(["aaa"], &mut arena);
            let next = TrackingBuffer::from(Buffer::from_lines(["bbb"], &mut arena));

            let iter = next.diff(&prev);
            let (lower, upper) = iter.size_hint();
            assert_eq!(lower, 0);
            assert_eq!(upper, Some(3));
            assert_eq!(iter.count(), 3);
        }

        #[test]
        fn diff_dirty_exhausted_iterator_stays_none() {
            let prev = Buffer::new(2, 1);
            let next = TrackingBuffer::new(2, 1);
            let mut iter = next.diff(&prev);
            // Single row, both empty: slice eq skips it immediately.
            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        }

        #[test]
        #[should_panic(expected = "buffers must have the same width")]
        fn diff_dirty_mismatched_widths_panic() {
            let prev = Buffer::new(5, 1);
            let next = TrackingBuffer::new(10, 1);
            let _ = next.diff(&prev);
        }
    }
}
