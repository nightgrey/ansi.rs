use  super::{Buffer, Cell, Routine};
use ansi::{Attribute, Color};
use geometry::{Point, Resolve, Size};

/// A zero-allocation iterator over the differences between two buffers of the same width.
///
/// Yields `(x, y, &Cell)` tuples for each cell in `next` that differs from the corresponding cell
/// in `prev`. Handles multi-width characters (including VS16 emoji trailing cells) and
/// [`CellDiffOption`] directives.
#[derive(Debug)]
pub struct BufferDiff<'prev, 'next> {
    /// The next (current) buffer's cells.
    next: &'next [Cell],
    /// The previous buffer's cells.
    prev: &'prev [Cell],
    size: Size,
    /// Current position in the flat cell array.
    pos: usize,
    /// Tracks trailing cells that must be yielded after a wide character is processed.
    ///
    /// Set when a wide char was replaced by narrower content (force=true) or when a VS16 emoji
    /// needs its trailing column checked (force=false).
    trailing: Option<TrailingState>,
}

/// Tracks pending trailing-cell yields when a wide character is followed by narrower content.
#[derive(Debug)]
struct TrailingState {
    next_index: usize,
    end: usize,
    /// When `true`, all cells in the trailing range are emitted unconditionally: the previous
    /// wide character's style was visible on blank cells, so the terminal may show stale style
    /// there and every trailing cell must be refreshed.
    ///
    /// When `false` (VS16 path), only cells whose symbol changed are emitted, because the emoji
    /// visually covers its trailing column and style differences there are invisible.
    force: bool,
}

/// Bitmask of [`Attribute`] flags that are visually meaningful on a blank
/// (space) cell.
///
/// When a wide character with a visible-on-blank attribute (e.g. underline,
/// inverse) is replaced by narrower content, the trailing cells must be
/// force-emitted even if they appear unchanged in the buffer — the terminal
/// may still render the stale attribute there.
const VISIBLE_ON_BLANK: Attribute = Attribute::Inverse
    | Attribute::Underline
    | Attribute::Blink
    | Attribute::RapidBlink
    | Attribute::Strikethrough;

impl<'prev, 'next> BufferDiff<'prev, 'next> {
    /// Creates a new iterator over the differences between `prev` and `next` terminal cells.
    ///
    /// Heights may differ; the iterator uses the minimum of the two.
    ///
    /// # Panics
    ///
    /// Panics if the buffers have different `x`, `y`, or `width` values.
    pub(crate) fn new(prev: &'prev Buffer, next: &'next Buffer) -> Self {
        assert_eq!(
            prev.width(),
            next.width(),
            "buffer areas must have the same width: prev={:?}, next={:?}",
            prev.width(),
            next.width()
        );

        let height = prev.height().min(next.height());

        Self {
            next: &next,
            prev: &prev,
            size: Size {
                width: prev.width(),
                height,
            },
            pos: 0,
            trailing: None,
        }
    }

    /// Converts a flat cell index to `(x, y)` grid coordinates.
    ///
    /// Uses the stored width and origin offset (`area.x`, `area.y`).
    fn point_of(&self, index: usize) -> Point {
        (&self.size).resolve(index)
    }
}

impl<'next> Iterator for BufferDiff<'_, 'next> {
    type Item = (Point, &'next Cell);

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.next.len().min(self.prev.len());

        // First, yield any pending trailing cells.
        if let Some(TrailingState {
            next_index,
            end,
            force,
        }) = &mut self.trailing
        {
            while *next_index < *end {
                let j = *next_index;
                // Advance past this cell; if it is wide, also skip its own trailing column
                // so the main loop does not emit a spurious EMPTY write over it.
                let cell_width = self.next[j].column_width();
                *next_index += cell_width;
                *end = (*end).max(*next_index).min(len);

                if (*force || self.prev[j].grapheme() != self.next[j].grapheme())
                {
                    return Some((self.point_of(j), &self.next[j]));
                }
            }

            // Done with trailing cells; resume main loop past the wide character.
            self.pos = *end;
            self.trailing = None;
        }

        while self.pos < len {
            let i = self.pos;
            self.pos += 1;

            let current = &self.next[i];
            let previous = &self.prev[i];

            // If the current cell is multi-width, ensure the trailing cells are
            // explicitly cleared when they previously contained non-blank content.
            // Some terminals do not reliably clear the trailing cell(s) when printing
            // a wide grapheme, which can result in visual artifacts (e.g., leftover
            // characters). Emitting an explicit update for the trailing cells avoids
            // this.
            let cell_width = current.column_width();
            if current == previous {
                // Equal cells still need to account for multi-width skip.
                self.pos += cell_width.saturating_sub(1);
                continue;
            }

            let previous_width = previous.column_width();

            if cell_width > 1 {
                self.pos += cell_width.saturating_sub(1);
            } else if previous_width > cell_width
                && (previous.style.background != Color::None || previous.style.attributes.intersects(VISIBLE_ON_BLANK))
            {
                // The previous wide character's style is visible on blank cells, so the
                // terminal may still show it on the trailing columns even after the
                // character is replaced. Force-emit every cell in the trailing range to
                // refresh the terminal regardless of whether the buffer content changed.
                self.trailing = Some(TrailingState {
                    next_index: i + 1,
                    end: i + previous_width,
                    force: true,
                });
            } else {
                // single-width character, no position adjustment needed
            }

            return Some((self.point_of(i), &self.next[i]));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU16;
    use ansi::Style;
    use geometry::Rect;
    use super::*;
    use crate::buffer::Buffer;
    use crate::Graphemes;

    #[test]
    fn empty_buffers_yield_no_diffs() {
        let buf = Buffer::new(5, 1);
        let diff: Vec<_> = BufferDiff::new(&buf, &buf).collect();
        assert!(diff.is_empty());
    }

    #[test]
    fn identical_buffers_yield_no_diffs() {
        let buf = Buffer::from_lines(["hello"], &mut Graphemes::new());
        let diff: Vec<_> = BufferDiff::new(&buf, &buf).collect();
        assert!(diff.is_empty());
    }

    #[test]
    fn single_cell_change() {
        let graphemes = &mut Graphemes::new();
        let prev = Buffer::from_lines(["hello"], graphemes);
        let next = Buffer::from_lines(["hallo"], graphemes);
        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].0.x, 1); // x
        assert_eq!(diff[0].0.y, 0); // y
        assert_eq!(diff[0].1.as_str(graphemes), "a");
    }

    #[test]
    fn all_cells_changed() {
        let graphemes = &mut Graphemes::new();
        let prev = Buffer::from_lines(["aaa"], graphemes);
        let next = Buffer::from_lines(["bbb"], graphemes);
        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();
        assert_eq!(diff.len(), 3);
    }

    #[test]
    fn test() {
        let graphemes = &mut Graphemes::new();
        let prev = Buffer::from_procedural(Routine::Diagonals {
            foreground: Some(Color::Red),
            background: Some(Color::Blue),
        }, 10, 10);
        let next = Buffer::from_procedural(Routine::Diagonals {
            foreground: Some(Color::Red),
            background: Some(Color::Yellow),
        }, 10, 10);
        let diff: Vec<_> = BufferDiff::new(&prev, &next).collect();
        dbg!(diff);
    }
}
