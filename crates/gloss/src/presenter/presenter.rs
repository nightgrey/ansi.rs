use super::Counting;
use crate::raster::Pen;
use crate::{Arena, Buffer, BufferDiff, Cell, TrackingBuffer};
use ansi::Escape;
use ansi::io::Write;
use ansi::sequences::*;
use ansi::{SGR, Style};
use geometry::{Point, Row};
use std::io::{self, BufWriter, Write as IoWrite};
use terminal::Capabilities;

use super::PresentStats;

/// Internal buffer size for the buffered writer.
const BUFFER_CAPACITY: usize = 64 * 1024;

/// Inline-mode bookkeeping.
///
/// Mirrors [`crate::raster::Rasterer`]'s inline state: the presenter claims
/// `height` rows of scrollback on the first frame, then moves the cursor
/// relative to that anchor on subsequent frames.
#[derive(Debug, Clone, Copy)]
struct InlineState {
    height: usize,
    is_first: bool,
}

/// State-tracked ANSI presenter.
///
/// The presenter consumes [`BufferDiff`] iterators directly (no intermediate
/// allocations) and emits the minimal byte stream needed to bring the terminal
/// from `prev` to `next`. Cursor and SGR state are tracked across runs via a
/// shared [`Pen`].
///
/// # Lifecycle
///
/// 1. Construct with [`Presenter::new`] (fullscreen) or
///    [`Presenter::inline`].
/// 2. Optionally attach capabilities via [`with_capabilities`](Self::with_capabilities).
/// 3. Call [`present`](Self::present) (or [`present_tracking`](Self::present_tracking))
///    once per frame.
/// 4. On resize, alt-screen toggle, or external terminal corruption, call
///    [`invalidate`](Self::invalidate) to force a full repaint next frame.
pub struct Presenter<W: IoWrite> {
    writer: Counting<BufWriter<W>>,
    pen: Pen,
    capabilities: Capabilities,
    invalidated: bool,
    inline: Option<InlineState>,
}

impl<W: IoWrite> Presenter<W> {
    /// Create a fullscreen presenter wrapping `writer`.
    pub fn new(writer: W) -> Self {
        Self {
            writer: Counting::new(BufWriter::with_capacity(BUFFER_CAPACITY, writer)),
            pen: Pen::new(),
            capabilities: Capabilities::default(),
            invalidated: true,
            inline: None,
        }
    }

    /// Create an inline presenter (renders in scrollback, not alt-screen).
    pub fn inline(writer: W) -> Self {
        Self {
            inline: Some(InlineState {
                height: 0,
                is_first: true,
            }),
            ..Self::new(writer)
        }
    }

    /// Attach a capability set (chainable).
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Returns the active capability set.
    #[inline]
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// Whether this presenter operates in inline mode.
    #[inline]
    pub fn is_inline(&self) -> bool {
        self.inline.is_some()
    }

    /// Force a full repaint on the next [`present`](Self::present) call.
    ///
    /// Use after resize, alt-screen toggle, or any external write that may
    /// have corrupted the tracked terminal state.
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Note a buffer resize. Forces a full repaint next frame.
    pub fn resize(&mut self, _width: usize, _height: usize) {
        self.invalidated = true;
        if let Some(inline) = self.inline.as_mut() {
            inline.is_first = true;
            inline.height = 0;
        }
    }

    /// Enter the alternate screen buffer.
    pub fn enter_alt_screen(&mut self) -> io::Result<()> {
        self.writer.escape(AlternateScreen::Set)?;
        self.invalidated = true;
        Ok(())
    }

    /// Exit the alternate screen buffer.
    pub fn exit_alt_screen(&mut self) -> io::Result<()> {
        ansi::escape!(
            self.writer,
            SGR::reset(),
            AlternateScreen::Reset,
            SGR::reset()
        )?;
        self.pen.clear();
        self.invalidated = true;
        Ok(())
    }

    /// Present a frame by diffing `next` against `prev`.
    pub fn present(
        &mut self,
        prev: &Buffer,
        next: &Buffer,
        arena: &Arena,
    ) -> io::Result<PresentStats> {
        let mut stats = PresentStats::default();
        self.writer.reset();

        let dims_changed = prev.width != next.width || prev.height != next.height;
        let force_full = self.invalidated || dims_changed;

        let emission = self
            .begin_frame(next.width, next.height, force_full)
            .and_then(|()| {
                if force_full {
                    self.emit_full(next, arena, &mut stats)
                } else if self.is_inline() {
                    self.emit_diff_inline(prev, next, arena, &mut stats)
                } else {
                    self.emit_diff(prev, next, arena, &mut stats)
                }
            });

        let cleanup = self.finish_frame();

        emission?;
        cleanup?;

        stats.bytes = self.writer.count();
        Ok(stats)
    }

    /// Present a frame using per-row dirty bits from a [`TrackingBuffer`].
    ///
    /// Unmarked rows are assumed identical to `prev`. The caller is
    /// responsible for clearing the dirty bits after this call (e.g. via
    /// [`TrackingBuffer::unmark_all`]).
    pub fn present_tracking(
        &mut self,
        prev: &Buffer,
        next: &TrackingBuffer,
        arena: &Arena,
    ) -> io::Result<PresentStats> {
        let mut stats = PresentStats::default();
        self.writer.reset();

        let dims_changed = prev.width != next.width || prev.height != next.height;
        let force_full = self.invalidated || dims_changed;

        let emission = self
            .begin_frame(next.width, next.height, force_full)
            .and_then(|()| {
                if force_full {
                    self.emit_full(next.as_inner(), arena, &mut stats)
                } else {
                    self.emit_diff_dirty(prev, next, arena, &mut stats)
                }
            });

        let cleanup = self.finish_frame();
        emission?;
        cleanup?;

        stats.bytes = self.writer.count();
        Ok(stats)
    }

    /// Flush any buffered output to the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    /// Consume the presenter and return the underlying writer.
    pub fn into_inner(mut self) -> io::Result<W> {
        self.writer.flush()?;
        self.writer
            .into_inner()
            .into_inner()
            .map_err(|e| e.into_error())
    }

    // ────────────────────────────────────────────────────────────────────
    // Frame envelope
    // ────────────────────────────────────────────────────────────────────

    fn begin_frame(&mut self, _w: usize, _h: usize, force_full: bool) -> io::Result<()> {
        if self.capabilities.use_sync_output() {
            self.writer.escape(SynchronizedOutput::Set)?;
        }
        self.writer.escape(TextCursorEnable::Reset)?;

        if force_full && !self.is_inline() {
            self.writer.escape(Home)?;
            self.writer.escape(EraseDisplay)?;
            self.pen.clear();
            self.invalidated = false;
        }

        Ok(())
    }

    /// Always-run cleanup: reset SGR, restore cursor, close sync.
    ///
    /// Returns the first error encountered so the caller can prioritise it
    /// over an emission error.
    fn finish_frame(&mut self) -> io::Result<()> {
        let style = self.pen.clear_style(&mut self.writer);
        let cursor = self.writer.escape(TextCursorEnable::Set);
        let sync = if self.capabilities.use_sync_output() {
            self.writer.escape(SynchronizedOutput::Reset)
        } else {
            Ok(())
        };
        let flush = self.writer.flush();

        style.and(cursor).and(sync).and(flush)
    }

    // ────────────────────────────────────────────────────────────────────
    // Cursor placement (inline-aware)
    // ────────────────────────────────────────────────────────────────────

    /// Move the pen to (row, col) using whichever cursor strategy fits the
    /// presenter mode (absolute for fullscreen, relative for inline).
    #[inline]
    fn move_pen(&mut self, row: u16, col: u16) -> io::Result<()> {
        if self.is_inline() {
            self.pen.move_to_relative(row, col, &mut self.writer)
        } else {
            self.pen.move_to(row, col, &mut self.writer)
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Full-paint path
    // ────────────────────────────────────────────────────────────────────

    fn emit_full(
        &mut self,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresentStats,
    ) -> io::Result<()> {
        let width = next.width;
        let height = next.height;
        if width == 0 || height == 0 {
            return Ok(());
        }

        if let Some(inline) = self.inline.as_mut()
            && inline.is_first {
                // First inline render: claim scrollback rows with \n separators.
                inline.is_first = false;
                inline.height = height;

                for y in 0..height {
                    if y > 0 {
                        self.writer.write_all(b"\n")?;
                    }
                    let row = &next[Row(y)];
                    let last = (0..width).rev().find(|&x| !row[x].is_empty());
                    if let Some(end) = last {
                        for col in 0..=end {
                            emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                            stats.cells += 1;
                        }
                        stats.runs += 1;
                    }
                    self.pen.clear_style(&mut self.writer)?;
                    self.writer.escape(EraseLineToEnd)?;
                }

                // Track final pen position so the next inline frame can do
                // a CUU+CR back to the top.
                self.pen.row = (height - 1) as u16;
                let last_row = &next[Row(height - 1)];
                self.pen.col = (0..width)
                    .rev()
                    .find(|&x| !last_row[x].is_empty())
                    .map_or(0, |end| end as u16 + 1);

                return Ok(());
            }

        // Fullscreen full-paint: write every non-empty row, EL its tail.
        for y in 0..height {
            let row = &next[Row(y)];
            let Some(end) = (0..width).rev().find(|&x| !row[x].is_empty()) else {
                continue;
            };
            self.move_pen(y as u16, 0)?;
            for col in 0..=end {
                emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                stats.cells += 1;
            }
            stats.runs += 1;
            self.pen.clear_style(&mut self.writer)?;
            self.writer.escape(EraseLineToEnd)?;
        }
        Ok(())
    }

    // ────────────────────────────────────────────────────────────────────
    // Diff paths
    // ────────────────────────────────────────────────────────────────────

    fn emit_diff(
        &mut self,
        prev: &Buffer,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresentStats,
    ) -> io::Result<()> {
        let mut runs = BufferDiff::runs(prev, next).peekable();
        while let Some(run) = runs.next() {
            self.move_pen(run.y, run.x)?;
            for change in run.iter() {
                emit_cell(change.cell, &mut self.writer, &mut self.pen, arena)?;
                stats.cells += 1;
            }
            stats.runs += 1;

            // Local peek-bridge: if the next run is close on the same row,
            // bleeding through the unchanged cells can be cheaper than a
            // cursor move. BufferDiff::runs already merges adjacent changes,
            // so this only fires across true gaps.
            if let Some(next_run) = runs.peek()
                && next_run.y == run.y
                && let Some(gap_start) = self.pen_col_after()
                && next_run.x > gap_start
            {
                self.maybe_bridge(next, arena, run.y, gap_start, next_run.x)?;
            }
        }
        Ok(())
    }

    fn emit_diff_inline(
        &mut self,
        prev: &Buffer,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresentStats,
    ) -> io::Result<()> {
        let inline = self.inline.as_mut().expect("inline state");
        let prev_height = inline.height;
        let height = next.height;

        // Grow: emit \n to claim new scrollback rows.
        if height > prev_height {
            let extra = height - prev_height;
            for _ in 0..extra {
                self.writer.write_all(b"\n")?;
            }
            self.pen.row += extra as u16;
            inline.height = height;
        }

        // Rewind to the top of the claimed region.
        if self.pen.row > 0 {
            self.writer.escape(CursorUp(self.pen.row))?;
        }
        self.writer.escape(CarriageReturn)?;
        self.pen.clear_position();

        // Now run the same diff path with relative cursor moves.
        let mut runs = BufferDiff::runs(prev, next).peekable();
        while let Some(run) = runs.next() {
            self.move_pen(run.y, run.x)?;
            for change in run.iter() {
                emit_cell(change.cell, &mut self.writer, &mut self.pen, arena)?;
                stats.cells += 1;
            }
            stats.runs += 1;

            if let Some(next_run) = runs.peek()
                && next_run.y == run.y
                && let Some(gap_start) = self.pen_col_after()
                && next_run.x > gap_start
            {
                self.maybe_bridge(next, arena, run.y, gap_start, next_run.x)?;
            }
        }

        // Shrink: clear any orphan rows below the new last row.
        let inline = self.inline.as_mut().unwrap();
        if height < prev_height {
            for _ in height..prev_height {
                self.pen
                    .move_to_relative(self.pen.row + 1, 0, &mut self.writer)?;
                self.writer.escape(EraseLineToEnd)?;
            }
            if self.pen.row > (height - 1) as u16 {
                let up = self.pen.row - (height - 1) as u16;
                self.writer.escape(CursorUp(up))?;
                self.pen.row = (height - 1) as u16;
            }
            inline.height = height;
        }

        Ok(())
    }

    fn emit_diff_dirty(
        &mut self,
        prev: &Buffer,
        next: &TrackingBuffer,
        arena: &Arena,
        stats: &mut PresentStats,
    ) -> io::Result<()> {
        let mut iter = next.diff(prev).peekable();
        // ByDirty yields one Change per base cell. Coalesce same-row adjacent
        // changes into one logical run by relying on `pen.move_to`'s built-in
        // "already there" no-op and counting runs only when we actually move.
        let mut last: Option<(u16, u16)> = None;
        for change in iter {
            let starts_run = match last {
                Some((py, px)) => change.y != py || change.x != px,
                None => true,
            };
            if starts_run {
                self.move_pen(change.y, change.x)?;
                stats.runs += 1;
            }
            emit_cell(change.cell, &mut self.writer, &mut self.pen, arena)?;
            stats.cells += 1;
            last = Some((change.y, self.pen.col));
        }
        Ok(())
    }

    /// Logical column the pen sits at if we're confident in its row, else `None`.
    #[inline]
    fn pen_col_after(&self) -> Option<u16> {
        Some(self.pen.col)
    }

    /// Decide whether to bleed through a gap of `[from, to)` cells on row `y`
    /// instead of moving the cursor. Emits the bleed if cheaper.
    fn maybe_bridge(
        &mut self,
        next: &Buffer,
        arena: &Arena,
        y: u16,
        from: u16,
        to: u16,
    ) -> io::Result<()> {
        // Cap the scan: large gaps are never worth bleeding.
        const MAX_GAP: u16 = 8;
        let gap = to.saturating_sub(from);
        if gap == 0 || gap > MAX_GAP {
            return Ok(());
        }

        let move_cost = cheapest_move_cost(self.pen.col, to);
        let bleed_cost = bridge_cost(next, arena, y, from, to, self.pen.style);

        if bleed_cost < move_cost {
            for x in from..to {
                let cell = &next[Point { x, y }];
                emit_cell(cell, &mut self.writer, &mut self.pen, arena)?;
            }
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────
// Free helpers
// ────────────────────────────────────────────────────────────────────────

/// Emit a single cell: SGR transition + grapheme bytes + advance pen.
///
/// Empty cells are written as a single space (to clear the position and
/// advance the terminal cursor). Continuation cells are skipped — emitting a
/// wide base already advanced the terminal cursor by the full width.
#[inline]
fn emit_cell<W: IoWrite>(cell: &Cell, w: &mut W, pen: &mut Pen, arena: &Arena) -> io::Result<()> {
    if cell.is_continuation() {
        return Ok(());
    }
    pen.transition(cell.style, w)?;
    let bytes = cell.as_bytes(arena);
    if bytes.is_empty() {
        // EMPTY: emit a space to keep the grid aligned.
        w.write_all(b" ")?;
        pen.col += 1;
    } else {
        w.write_all(bytes)?;
        pen.col += cell.width().max(1) as u16;
    }
    Ok(())
}

/// Cheapest cursor move cost on the same row (CHA vs CUF/CUB).
///
/// This intentionally ignores cross-row moves: callers only invoke this when
/// peeking at a same-row neighbour, so a CUP path is never on the table.
#[inline]
fn cheapest_move_cost(from_col: u16, to_col: u16) -> usize {
    let cha = CursorHorizontalAbsolute(to_col).cost();
    if to_col >= from_col {
        cha.min(CursorForward(to_col - from_col).cost())
    } else {
        cha.min(CursorBackward(from_col - to_col).cost())
    }
}

/// Estimated byte cost of bleeding through `[from, to)` on row `y`.
///
/// Conservative: a real SGR transition is usually several bytes, but the
/// exact size depends on the from/to styles. The fixed 8-byte estimate is
/// tuned so the bridge predicate stays conservative on styled gaps.
fn bridge_cost(
    next: &Buffer,
    arena: &Arena,
    y: u16,
    from: u16,
    to: u16,
    pen_style: Style,
) -> usize {
    let mut total = 0usize;
    let mut style = pen_style;
    for x in from..to {
        let cell = &next[Point { x, y }];
        if cell.is_continuation() {
            continue;
        }
        if cell.style != style {
            total += 8;
            style = cell.style;
        }
        let bytes = cell.as_bytes(arena);
        total += if bytes.is_empty() { 1 } else { bytes.len() };
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer_generation::{buffer_chessboard, buffer_diagonals, buffer_solid};
    use ansi::Color;
    use std::io::Cursor;

    #[test]
    fn test() {
        let writer = Cursor::new(Vec::new());
        let mut presenter = Presenter::new(writer);

        let prev = buffer_solid(100, 100, Color::BrightWhite);
        dbg!(prev);

        let next = buffer_diagonals(100, 100);
        let arena = Arena::new();
        dbg!(next);
    }
}
