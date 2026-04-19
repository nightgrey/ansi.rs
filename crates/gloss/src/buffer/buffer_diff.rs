use std::iter::FusedIterator;

use crate::buffer::{Buffer, Cell};

/// A zero-allocation iterator over the differences between two buffers of the same width.
///
/// Yields `(x, y, &Cell)` tuples for each cell in `next` that differs from the
/// corresponding cell in `prev`. Zero-width cells (the trailing positions of a
/// wide grapheme) are skipped: they are implicitly redrawn by the wide base
/// cell, so diffing them separately would produce redundant updates.
///
/// When the two buffers have different heights, the iterator only walks the
/// overlapping region (the shorter of the two).
///
/// # Known limitation: VS16 emoji presentation
///
/// Some terminals fail to clear the continuation cell when re-rendering an
/// emoji that uses the variation selector U+FE0F (VS16). This iterator does
/// not carry the arena needed to inspect grapheme bytes, so it cannot detect
/// that case. If you target terminals with this quirk, emit an explicit clear
/// of the trailing cell after each wide emoji update in a post-pass.
#[derive(Debug)]
pub struct BufferDiff<'prev, 'next> {
    next: &'next [Cell],
    prev: &'prev [Cell],
    width: usize,
    len: usize,
    pos: usize,
}

impl<'prev, 'next> BufferDiff<'prev, 'next> {
    /// Creates a new iterator over the cells that differ between `prev` and `next`.
    ///
    /// # Panics
    ///
    /// Panics if the buffers do not have the same width.
    pub fn new(prev: &'prev Buffer, next: &'next Buffer) -> Self {
        assert_eq!(
            prev.width, next.width,
            "buffers must have the same width: prev={}, next={}",
            prev.width, next.width,
        );

        let width = prev.width;
        let height = prev.height.min(next.height);

        Self {
            next: &next,
            prev: &prev,
            width,
            len: width * height,
            pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'next> Iterator for BufferDiff<'_, 'next> {
    type Item = (usize, usize, &'next Cell);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.len {
            let i = self.pos;
            let current = &self.next[i];
            let cell_width = current.width() as usize;

            // Zero-width cells are the trailing positions of a wide grapheme
            // and are redrawn implicitly by that base cell.
            if cell_width == 0 {
                self.pos += 1;
                continue;
            }

            self.pos += cell_width;

            if current != &self.prev[i] {
                let x = i % self.width;
                let y = i / self.width;
                return Some((x, y, current));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len.saturating_sub(self.pos)))
    }
}

impl FusedIterator for BufferDiff<'_, '_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Arena;

    #[test]
    fn empty_buffers_yield_no_diffs() {
        let buf = Buffer::new(5, 1);
        let diff: Vec<_> = BufferDiff::new(&buf, &buf).collect();
        assert!(diff.is_empty());
    }

    #[test]
    fn identical_buffers_yield_no_diffs() {
        let mut arena = Arena::new();
        let buf = Buffer::from_lines(["hello"], &mut arena);
        let diff: Vec<_> = BufferDiff::new(&buf, &buf).collect();
        assert!(diff.is_empty());
    }

    #[test]
    fn single_cell_change() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["hello"], &mut arena);
        let next = Buffer::from_lines(["hallo"], &mut arena);
        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();
        assert_eq!(diff.len(), 1);
        assert_eq!((diff[0].0, diff[0].1), (1, 0));
        assert_eq!(diff[0].2.as_str(&arena), "a");
    }

    #[test]
    fn all_cells_changed() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["aaa"], &mut arena);
        let next = Buffer::from_lines(["bbb"], &mut arena);
        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();
        assert_eq!(diff.len(), 3);
    }

    #[test]
    fn continuation_cells_are_skipped() {
        let mut arena = Arena::new();
        let prev = Buffer::new(4, 1);
        let mut next = Buffer::new(4, 1);
        // Layout: [中, CONT, X, SPACE] — the wide 中 has a continuation at x=1.
        next.set_string(0..4, "中X", &mut arena);

        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();

        // Only the two base cells (x=0, x=2) differ; the continuation at x=1
        // is not emitted.
        assert_eq!(diff.len(), 2);
        assert_eq!((diff[0].0, diff[0].1), (0, 0));
        assert_eq!((diff[1].0, diff[1].1), (2, 0));
    }

    #[test]
    fn wide_to_narrow_updates_both_positions() {
        let mut arena = Arena::new();
        let mut prev = Buffer::new(2, 1);
        prev.set_string(0..2, "中", &mut arena);
        let next = Buffer::from_lines(["ab"], &mut arena);

        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();

        // prev[1] was CONT, next[1] is 'b' (narrow) — both differ and both are
        // yielded since neither cell in `next` is a continuation.
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].0, 0);
        assert_eq!(diff[1].0, 1);
    }

    #[test]
    fn coordinates_are_row_major() {
        let mut arena = Arena::new();
        let prev = Buffer::new(3, 2);
        let mut next = Buffer::new(3, 2);
        next.set_string(1..2, "x", &mut arena); // (1, 0)
        next.set_string(5..6, "y", &mut arena); // (2, 1)

        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();

        assert_eq!(diff.len(), 2);
        assert_eq!((diff[0].0, diff[0].1), (1, 0));
        assert_eq!((diff[1].0, diff[1].1), (2, 1));
    }

    #[test]
    fn different_heights_use_minimum() {
        let mut arena = Arena::new();
        let prev = Buffer::new(3, 1);
        let mut next = Buffer::new(3, 3);
        next.set_string(0..3, "abc", &mut arena); // row 0
        next.set_string(3..6, "xyz", &mut arena); // row 1 — beyond prev's height

        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();

        assert_eq!(diff.len(), 3);
        assert!(diff.iter().all(|(_, y, _)| *y == 0));
    }

    #[test]
    #[should_panic(expected = "buffers must have the same width")]
    fn mismatched_widths_panic() {
        let prev = Buffer::new(5, 1);
        let next = Buffer::new(10, 1);
        let _ = BufferDiff::new(&prev, &next);
    }

    #[test]
    fn size_hint_is_an_upper_bound() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["aaa"], &mut arena);
        let next = Buffer::from_lines(["bbb"], &mut arena);

        let iter = BufferDiff::new(&prev, &next);
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 0);
        assert_eq!(upper, Some(3));
        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn exhausted_iterator_stays_none() {
        let buf = Buffer::new(2, 1);
        let mut iter = BufferDiff::new(&buf, &buf);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn buffer_diff_method_matches_free_function() {
        let mut arena = Arena::new();
        let prev = Buffer::from_lines(["hello"], &mut arena);
        let next = Buffer::from_lines(["hallo"], &mut arena);

        let via_method: Vec<_> = Buffer::diff(&prev, &next).map(|(x, y, _)| (x, y)).collect();
        let via_ctor: Vec<_> = BufferDiff::new(&prev, &next)
            .map(|(x, y, _)| (x, y))
            .collect();
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

        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();
        assert_eq!(diff.len(), 1);
        assert_eq!((diff[0].0, diff[0].1), (0, 0));
    }
}
