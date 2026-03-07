//! Framebuffer painter with clip stack and invariant-safe drawing primitives.
//!
//! All drawing ops are clip-aware and preserve the wide-glyph invariant:
//! a width-2 cell always has a continuation (width-0) cell to its right.
//! Partial overwrites of wide glyphs clear the orphaned half, even if
//! the neighbor is outside the current clip (the one exception to clip
//! enforcement — required for correctness).

use derive_more::{Deref, DerefMut, Index, IndexMut};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use ansi::Style;
use geometry::{Point, Rect};
use geometry::prelude::*;
use crate::{Buffer, Grapheme};

// ---------------------------------------------------------------------------
// Painter
// ---------------------------------------------------------------------------

/// A clip-aware drawing context over a [`Buffer`].
///
/// Created via [`Painter::new`], which pushes the full framebuffer bounds as
/// the initial clip. All subsequent drawing is intersected with the current
/// clip rectangle.
#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
pub struct Painter<'a> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    buffer: &'a mut Buffer,
    stack: Vec<Rect>,
}

impl<'a> Painter<'a> {
    /// Begin painting on `buf`. The initial clip is the full buffer bounds.
    pub fn new(buf: &'a mut Buffer) -> Self {
        let bounds = Rect::bounds(0, 0, buf.width, buf.height);
        Self {
            buffer: buf,
            stack: vec![bounds],
        }
    }

    /// The buffer dimensions as a [`Rect`].
    #[inline]
    fn bounds(&self) -> Rect {
        Rect::bounds(0, 0, self.width, self.height)
    }

    /// Current effective clip rectangle.
    #[inline]
    pub fn current(&self) -> Rect {
        // SAFETY: clips is never empty — `new` pushes one, `pop` refuses to
        // remove the last.
        *self.stack.last().unwrap()
    }

    /// Push `rect` intersected with the current clip and buffer bounds.
    pub fn push(&mut self, rect: Rect) {
        let next = self.bounds().intersect(&rect).intersect(&self.current());
        self.stack.push(next);
    }

    /// Pop the most recent clip. Panics if called on the initial (root) clip.
    pub fn pop(&mut self) {
        assert!(self.stack.len() > 1, "cannot pop the root clip");
        self.stack.pop();
    }

    /// Push `rect` as a clip, run `f`, then pop. Strictly scoped alternative
    /// to manual push/pop.
    pub fn push_and_pop(&mut self, rect: Rect, f: impl FnOnce(&mut Painter<'_>)) {
        self.push(rect);
        f(self);
        self.pop();
    }

    // -- Coordinate helpers -------------------------------------------------

    /// True when `(col, row)` is inside both the buffer and the current clip.
    #[inline]
    fn can_touch(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 {
            return false;
        }
        let (c, r) = (col as usize, row as usize);
        if c >= self.width || r >= self.height {
            return false;
        }
        let clip = self.current();
        c >= clip.left() && c < clip.right() && r >= clip.top() && r < clip.bottom()
    }

    /// True when *both* `col` and `col+1` can be touched (needed for wide writes).
    #[inline]
    fn can_touch_wide(&self, col: i32, row: i32) -> bool {
        self.can_touch(col, row) && self.can_touch(col + 1, row)
    }

    // -- Invariant-safe cell writes -----------------------------------------
    //
    // These are the *only* paths that mutate cells. Every public draw op
    // bottoms out here, ensuring the wide-glyph invariant is never violated.

    /// Overwrite `(col, row)` with a width-1 grapheme, repairing any wide
    /// glyph it overlaps.
    ///
    /// **Clip exception**: the paired half of a broken wide glyph is cleared
    /// even when that neighbor is outside the current clip.
    fn write_w1(
        &mut self,
        col: usize,
        row: usize,
        grapheme: Grapheme,
        style: Style,
    ) -> bool {
        if col >= self.width || row >= self.height {
            return false;
        }
        if !self.can_touch(col as i32, row as i32) {
            return false;
        }

        // If target is a continuation cell, its lead (col-1) is now orphaned.
        if self[(row, col)].is_continuation() {
            if col > 0 {
                self[(row, col - 1)].set_space(style);
            }
        }

        // If target is a wide lead, its continuation (col+1) is now orphaned.
        if self[(row, col)].is_wide() {
            if col + 1 < self.width {
                self[(row, col + 1)].set_space(style);
            }
        }

        // If *next* cell is a continuation whose lead we're about to erase,
        // clear it too.
        if col + 1 < self.width && self[(row, col + 1)].is_continuation() {
            self[(row, col + 1)].set_space(style);
        }

        self[(row, col)].set(grapheme, 1, style);
        true
    }

    /// Write a width-2 grapheme at `(col, row)` + `(col+1, row)`.
    ///
    /// Both cells must be touchable; returns `false` otherwise.
    fn write_w2(
        &mut self,
        col: usize,
        row: usize,
        grapheme: Grapheme,
        style: Style,
    ) -> bool {
        if !self.can_touch_wide(col as i32, row as i32) {
            return false;
        }

        // Clear both cells first (this repairs any overlapping wide glyphs).
        let space = Grapheme::SPACE;
        if !self.write_w1(col, row, space, style) {
            return false;
        }
        if !self.write_w1(col + 1, row, space, style) {
            return false;
        }

        // Now install the wide pair.
        self[(row, col)].set(grapheme, 2, style);
        self[(row, col + 1)].set_continuation(style);
        true
    }

    // -- Public drawing primitives ------------------------------------------

    /// Place a single grapheme at `(col, row)` with the given display `width`.
    ///
    /// **Replacement policy** (deterministic, never produces half-glyphs):
    /// - `width == 2` but only one cell fits → replaced with `U+FFFD` (width 1)
    /// - grapheme bytes exceed inline capacity and arena stash fails → `U+FFFD`
    pub fn put(
        &mut self,
        col: i32,
        row: i32,
        grapheme: Grapheme,
        width: u8,
        style: Style,
    ) {
        if col < 0 || row < 0 {
            return;
        }
        let (uc, ur) = (col as usize, row as usize);
        if uc >= self.width || ur >= self.height {
            return;
        }

        match width {
            1 => {
                self.write_w1(uc, ur, grapheme, style);
            }
            2 => {
                if !self.write_w2(uc, ur, grapheme, style) {
                    // Can't fit wide — deterministic replacement, never half-glyph.
                    self.write_w1(uc, ur, Grapheme::REPLACEMENT_CHARACTER, style);
                }
            }
            _ => {}
        }
    }

    /// Fill `rect` with spaces in the given style.
    pub fn fill(&mut self, rect: Rect, style: Style) {
        let effective = self.bounds()
            .intersect(&self.current())
            .intersect(&rect);
        if effective.is_empty() {
            return;
        }

        let space = Grapheme::SPACE;
        for row in effective.top()..effective.bottom() {
            for col in effective.left()..effective.right() {
                self.write_w1(col, row, space, style);
            }
        }
    }

    /// Draw UTF-8 `text` starting at `(col, row)`, advancing the cursor by
    /// each grapheme's display width.
    ///
    /// Cursor advance is **stable**: clipping affects what's drawn, not how
    /// far the cursor moves. This keeps layout deterministic.
    pub fn draw_text(&mut self, col: i32, row: i32, text: &str, style: Style) {
        if row < 0 || text.is_empty() {
            return;
        }
        if row as usize >= self.height {
            return;
        }

        let mut cx: i64 = col as i64;

        for (cluster, width) in text.graphemes(true).map(|g| (g, g.width())) {
            if width == 0 {
                continue;
            }

            let grapheme = Grapheme::encode(cluster, &mut self.arena);

            if width == 2 {
                // Wide: need both cells touchable, else replace.
                if cx >= 0 && cx + 1 <= i32::MAX as i64 {
                    let ix = cx as i32;
                    if self.can_touch_wide(ix, row) {
                        self.put(ix, row, grapheme, 2, style);
                    } else if self.can_touch(ix, row) {
                        // Lead visible, continuation clipped → replacement.
                        let repl = Grapheme::REPLACEMENT_CHARACTER;
                        self.put(ix, row, repl, 1, style);
                    }
                    // else: fully clipped, draw nothing.
                }
                cx += 2;
            } else {
                if cx >= 0 && cx <= i32::MAX as i64 {
                    self.put(cx as i32, row, grapheme, 1, style);
                }
                cx += 1;
            }

            if cx > i32::MAX as i64 {
                break;
            }
        }
    }

    /// Repeat `ch` horizontally for `len` cells starting at `(col, row)`.
    pub fn hline(&mut self, col: i32, row: i32, len: i32, ch: char, style: Style) {
        if len <= 0 {
            return;
        }
        let g = Grapheme::from_char(ch);
        for i in 0..len {
            self.put(col.saturating_add(i), row, g, 1, style);
        }
    }

    /// Repeat `ch` vertically for `len` cells starting at `(col, row)`.
    pub fn vline(&mut self, col: i32, row: i32, len: i32, ch: char, style: Style) {
        if len <= 0 {
            return;
        }
        let g = Grapheme::from_char(ch);
        for i in 0..len {
            self.put(col, row.saturating_add(i), g, 1, style);
        }
    }

    /// Draw an ASCII box outline using `+`, `-`, `|`.
    pub fn draw_box(&mut self, rect: Rect, style: Style) {
        if rect.is_empty() {
            return;
        }
        let (x, y, w, h) = (
            rect.left() as i32,
            rect.top() as i32,
            rect.width() as i32,
            rect.height() as i32,
        );

        if w == 1 && h == 1 {
            self.put(x, y, Grapheme::from_char('+'), 1, style);
            return;
        }

        let right = x + w - 1;
        let bottom = y + h - 1;

        // Corners
        let corner = Grapheme::from_char('+');
        self.put(x, y, corner, 1, style);
        self.put(right, y, corner, 1, style);
        self.put(x, bottom, corner, 1, style);
        self.put(right, bottom, corner, 1, style);

        // Horizontal edges (inner)
        if w > 2 {
            self.hline(x + 1, y, w - 2, '-', style);
            self.hline(x + 1, bottom, w - 2, '-', style);
        }

        // Vertical edges (inner)
        if h > 2 {
            self.vline(x, y + 1, h - 2, '|', style);
            self.vline(right, y + 1, h - 2, '|', style);
        }
    }

    /// Copy cells from `src` rect to `dst` rect with overlap-safe ordering.
    ///
    /// Wide glyphs that don't fully fit in the effective copy region are
    /// replaced with `U+FFFD`. Clip-aware for the destination.
    pub fn blit(&mut self, dst: Rect, src: Rect) {
        if dst.is_empty() || src.is_empty() {
            return;
        }

        let w = dst.width().min(src.width());
        let h = dst.height().min(src.height());
        if w == 0 || h == 0 {
            return;
        }

        // Determine iteration order for overlap safety (memmove semantics).
        let (y_range, x_range) = blit_order(dst, src, w, h);

        let clip = self.current();

        for oy in y_range.clone() {
            let sy = src.top() + oy;
            let dy = dst.top() + oy;

            for ox in x_range.clone() {
                let sx = src.left() + ox;
                let dx = dst.left() + ox;

                if !clip.contains(&Point::new(dx, dy)) {
                    continue;
                }
                if sx >= self.width || sy >= self.height {
                    continue;
                }

                let cell = self[(sy, sx)];

                // Continuations are installed by their lead; skip.
                if cell.is_continuation() {
                    continue;
                }

                if cell.is_wide() && ox + 1 >= w {
                    // Wide lead doesn't fully fit in copy region → replace.
                    let repl = Grapheme::REPLACEMENT_CHARACTER;
                    self.put(dx as i32, dy as i32, repl, 1, *cell.style());
                } else {
                    self.put(dx as i32, dy as i32, cell.grapheme(), cell.width(), *cell.style());
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Blit iteration order
// ---------------------------------------------------------------------------

/// Compute forward/reverse iteration ranges for overlap-safe copy.
fn blit_order(
    dst: Rect,
    src: Rect,
    w: usize,
    h: usize,
) -> (IterRange, IterRange) {
    let y_range = if dst.top() > src.top() {
        IterRange::reverse(h)
    } else {
        IterRange::forward(h)
    };

    let x_range = if dst.top() == src.top() && dst.left() > src.left() {
        IterRange::reverse(w)
    } else {
        IterRange::forward(w)
    };

    (y_range, x_range)
}

/// A clonable iterator that yields `0..n` either forward or reversed.
#[derive(Clone)]
enum IterRange {
    Forward(std::ops::Range<usize>),
    Reverse(std::iter::Rev<std::ops::Range<usize>>),
}

impl IterRange {
    fn forward(n: usize) -> Self {
        Self::Forward(0..n)
    }
    fn reverse(n: usize) -> Self {
        Self::Reverse((0..n).rev())
    }
}

impl Iterator for IterRange {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        match self {
            Self::Forward(r) => r.next(),
            Self::Reverse(r) => r.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn painter_put() {
        let mut buffer = Buffer::new(10, 22);
        let mut painter = Painter::new(&mut buffer);
        painter.draw_text(0, 0, "Hello World, we are testing!", Style::default());

        dbg!(&buffer.to_string());
    }
}