use super::Counting;
use crate::raster::Pen;
use crate::{Arena, Buffer, BufferDiff, Cell, TrackingBuffer};
use ansi::EscapeWrite;
use ansi::sequences::*;
use ansi::{SGR, Style};
use geometry::{Point, Row};
use std::io::{self, BufWriter, Write};
use terminal::Capabilities;

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
pub struct Presenter<W: Write> {
    writer: Counting<BufWriter<W>>,
    pen: Pen,
    capabilities: Capabilities,
    invalidated: bool,
    inline: Option<InlineState>,
}

impl<W: Write> Presenter<W> {
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

    pub fn get_writer(&self) -> &W {
       self.writer.as_ref().get_ref()
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
    ) -> io::Result<PresenterStats> {
        let mut stats = PresenterStats::default();
        self.writer.reset();

        let dims_changed = prev.width != next.width || prev.height != next.height;
        let force_full = self.invalidated || dims_changed;

        let emission = self
            .begin_frame(next.width, next.height, force_full)
            .and_then(|()| {
                if self.is_inline() {
                    if force_full {
                        self.emit_inline_full(next, arena, &mut stats)
                    } else {
                        self.emit_diff_inline(prev, next, arena, &mut stats)
                    }
                } else if force_full {
                    self.emit_full(next, arena, &mut stats)
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
    ) -> io::Result<PresenterStats> {
        let mut stats = PresenterStats::default();
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

        if force_full {
            // Fullscreen homes + erases the screen; inline mode claims/rewinds
            // its own rows inside `emit_full`, so it must not touch the screen
            // here. Either way the full paint discharges the invalidation, so
            // clear the flag for both modes — otherwise inline would re-take
            // the full-paint path on every frame and never diff.
            if !self.is_inline() {
                self.writer.escape(Home)?;
                self.writer.escape(EraseDisplay)?;
                self.pen.clear();
            }
            self.invalidated = false;
        }

        Ok(())
    }

    /// Always-run cleanup: reset SGR, restore cursor, close sync.
    ///
    /// Returns the first error encountered so the caller can prioritise it
    /// over an emission error.
    fn finish_frame(&mut self) -> io::Result<()> {
        let style = self.pen.reset(&mut self.writer);
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
            self.pen.relative_position(row, col, &mut self.writer)
        } else {
            self.pen.position(row, col, &mut self.writer)
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Full-paint path
    // ────────────────────────────────────────────────────────────────────

    fn emit_full(
        &mut self,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresenterStats,
    ) -> io::Result<()> {
        let width = next.width;
        let height = next.height;
        if width == 0 || height == 0 {
            return Ok(());
        }

        // Fullscreen full-paint: the screen was just homed + erased, so empty
        // rows need no clearing and can be skipped. Write every non-empty row
        // and EL its tail to drop any leftover content past the last cell.
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
            self.pen.reset(&mut self.writer)?;
            self.writer.escape(EraseLineToEnd)?;
        }
        Ok(())
    }

    // ────────────────────────────────────────────────────────────────────
    // Inline full-paint path
    // ────────────────────────────────────────────────────────────────────

    /// Full repaint in inline mode.
    ///
    /// On the very first frame this claims `height` rows of scrollback. On any
    /// later forced repaint (resize, [`invalidate`](Self::invalidate), external
    /// corruption) the rows already exist, so we rewind to the anchor and
    /// rewrite every row in place. Unlike [`emit_full`](Self::emit_full) this
    /// must EL-clear *every* row — there is no `EraseDisplay` in inline mode,
    /// so skipping empty rows would leave the prior frame's content behind.
    fn emit_inline_full(
        &mut self,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresenterStats,
    ) -> io::Result<()> {
        let width = next.width;
        let height = next.height;
        if width == 0 || height == 0 {
            return Ok(());
        }

        if self.inline.as_ref().is_some_and(|i| i.is_first) {
            return self.inline_claim(next, arena, stats);
        }

        let prev_height = self.inline_grow(height)?;
        self.inline_rewind()?;

        for y in 0..height {
            self.move_pen(y as u16, 0)?;
            let row = &next[Row(y)];
            if let Some(end) = (0..width).rev().find(|&x| !row[x].is_empty()) {
                for col in 0..=end {
                    emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                    stats.cells += 1;
                }
                stats.runs += 1;
            }
            self.pen.reset(&mut self.writer)?;
            self.writer.escape(EraseLineToEnd)?;
        }

        self.inline_shrink(prev_height, height)
    }

    /// First inline render: claim `height` rows of scrollback with `\n`
    /// separators and record the anchor so later frames can rewind to it.
    fn inline_claim(
        &mut self,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresenterStats,
    ) -> io::Result<()> {
        let width = next.width;
        let height = next.height;

        if let Some(inline) = self.inline.as_mut() {
            inline.is_first = false;
            inline.height = height;
        }

        for y in 0..height {
            if y > 0 {
                self.writer.write_all(b"\n")?;
            }
            let row = &next[Row(y)];
            if let Some(end) = (0..width).rev().find(|&x| !row[x].is_empty()) {
                for col in 0..=end {
                    emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                    stats.cells += 1;
                }
                stats.runs += 1;
            }
            self.pen.reset(&mut self.writer)?;
            self.writer.escape(EraseLineToEnd)?;
        }

        // Track the final pen position so the next inline frame can CUU+CR
        // back to the top of the claimed region.
        self.pen.row = (height - 1) as u16;
        let last_row = &next[Row(height - 1)];
        self.pen.col = (0..width)
            .rev()
            .find(|&x| !last_row[x].is_empty())
            .map_or(0, |end| end as u16 + 1);

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
        stats: &mut PresenterStats,
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
        stats: &mut PresenterStats,
    ) -> io::Result<()> {
        let prev_height = self.inline_grow(next.height)?;
        self.inline_rewind()?;

        // Run the same diff path as fullscreen, but with relative cursor moves.
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

        self.inline_shrink(prev_height, next.height)
    }

    // ────────────────────────────────────────────────────────────────────
    // Inline scaffolding (shared by the diff and full-repaint paths)
    // ────────────────────────────────────────────────────────────────────

    /// Claim extra scrollback rows when the frame grew, returning the height
    /// the claimed region had *before* this frame (the basis for shrink).
    fn inline_grow(&mut self, height: usize) -> io::Result<usize> {
        let prev_height = self.inline.as_ref().expect("inline state").height;
        if height > prev_height {
            let extra = height - prev_height;
            for _ in 0..extra {
                self.writer.write_all(b"\n")?;
            }
            self.pen.row += extra as u16;
            self.inline.as_mut().expect("inline state").height = height;
        }
        Ok(prev_height)
    }

    /// Rewind the pen to the top-left of the claimed region (CUU + CR).
    fn inline_rewind(&mut self) -> io::Result<()> {
        if self.pen.row > 0 {
            self.writer.escape(CursorUp(self.pen.row))?;
        }
        self.writer.escape(CarriageReturn)?;
        self.pen.origin();
        Ok(())
    }

    /// Clear orphan rows when the frame shrank and pull the pen back up to the
    /// new last row.
    fn inline_shrink(&mut self, prev_height: usize, height: usize) -> io::Result<()> {
        if height >= prev_height {
            return Ok(());
        }
        for _ in height..prev_height {
            self.pen
                .relative_position(self.pen.row + 1, 0, &mut self.writer)?;
            self.writer.escape(EraseLineToEnd)?;
        }
        if self.pen.row > (height - 1) as u16 {
            let up = self.pen.row - (height - 1) as u16;
            self.writer.escape(CursorUp(up))?;
            self.pen.row = (height - 1) as u16;
        }
        self.inline.as_mut().expect("inline state").height = height;
        Ok(())
    }

    fn emit_diff_dirty(
        &mut self,
        prev: &Buffer,
        next: &TrackingBuffer,
        arena: &Arena,
        stats: &mut PresenterStats,
    ) -> io::Result<()> {
        // ByDirty yields one Change per base cell. Coalesce same-row adjacent
        // changes into one logical run by relying on `pen.move_to`'s built-in
        // "already there" no-op and counting runs only when we actually move.
        let mut last: Option<(u16, u16)> = None;
        for change in next.diff(prev) {
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

/// Per-frame counters returned by [`Presenter::present`].
///
/// All fields are populated regardless of which path produced the frame
/// (full-paint, diff, dirty). `bytes` is read from the byte-counting writer
/// after flush, so it reflects what was actually emitted.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PresenterStats {
    /// Number of base cells emitted to the terminal.
    pub cells: usize,
    /// Number of distinct emission spans (a span ends with a cursor move).
    pub runs: usize,
    /// Bytes written for this frame, after flush.
    pub bytes: u64,
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
fn emit_cell<W: Write>(cell: &Cell, w: &mut W, pen: &mut Pen, arena: &Arena) -> io::Result<()> {
    if cell.is_continuation() {
        return Ok(());
    }
    pen.style(cell.style, w)?;
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
    use ansi::{Color, Style};
    use std::io::Cursor;

    /// Test wrapper that owns the previous frame and exposes the bytes emitted
    /// by the most recent `present` call. Mirrors the `Shadowed` helper used in
    /// the rasterer tests so the two implementations can be checked against the
    /// same expectations.
    struct Harness {
        inner: Presenter<Cursor<Vec<u8>>>,
        prev: Buffer,
        mark: usize,
    }

    impl Harness {
        fn new(width: usize, height: usize) -> Self {
            Self {
                inner: Presenter::new(Cursor::new(Vec::new())),
                prev: Buffer::new(width, height),
                mark: 0,
            }
        }

        fn inline(width: usize, height: usize) -> Self {
            Self {
                inner: Presenter::inline(Cursor::new(Vec::new())),
                prev: Buffer::new(width, height),
                mark: 0,
            }
        }

        fn with_capabilities(mut self, caps: Capabilities) -> Self {
            self.inner = self.inner.with_capabilities(caps);
            self
        }

        /// Present `next`, returning just the bytes emitted for this frame.
        fn present(&mut self, next: &Buffer, arena: &Arena) -> (PresenterStats, Vec<u8>) {
            if self.prev.width != next.width || self.prev.height != next.height {
                self.prev.resize(next.width, next.height);
                self.prev.clear();
            }
            let stats = self.inner.present(&self.prev, next, arena).unwrap();
            self.prev.copy_from_slice(next.as_ref());

            let all = self.inner.get_writer().get_ref();
            let frame = all[self.mark..].to_vec();
            self.mark = all.len();
            (stats, frame)
        }

        fn frame_str(&mut self, next: &Buffer, arena: &Arena) -> String {
            let (_, bytes) = self.present(next, arena);
            String::from_utf8_lossy(&bytes).into_owned()
        }

        fn invalidate(&mut self) {
            self.inner.invalidate();
        }
    }

    // ── Fullscreen ──────────────────────────────────────────────────────

    #[test]
    fn fullscreen_first_frame_clears_then_paints() {
        let buf = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut h = Harness::new(3, 1);
        let out = h.frame_str(&buf, &Arena::new());
        assert!(out.contains("\x1B[2J"), "first frame should ED2: {out:?}");
        assert!(out.contains('A'), "should paint 'A': {out:?}");
    }

    #[test]
    fn fullscreen_identical_second_frame_emits_no_content() {
        let buf = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None), (0, 1, 'B', Style::None)]);
        let mut h = Harness::new(3, 1);
        h.present(&buf, &Arena::new());
        let out = h.frame_str(&buf, &Arena::new());
        assert!(!out.contains('A'), "should not re-emit 'A': {out:?}");
        assert!(!out.contains('B'), "should not re-emit 'B': {out:?}");
    }

    #[test]
    fn fullscreen_diff_emits_only_changed_cell() {
        let arena = Arena::new();
        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None), (0, 1, 'B', Style::None), (0, 2, 'C', Style::None)]);
        let mut h = Harness::new(3, 1);
        h.present(&buf1, &arena);

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None), (0, 1, 'X', Style::None), (0, 2, 'C', Style::None)]);
        let out = h.frame_str(&buf2, &arena);
        assert!(out.contains('X'), "should emit 'X': {out:?}");
        assert!(!out.contains('A'), "should not re-emit 'A': {out:?}");
        assert!(!out.contains('C'), "should not re-emit 'C': {out:?}");
    }

    #[test]
    fn fullscreen_invalidate_forces_full_redraw() {
        let buf = Buffer::from_chars(2, 1, &[(0, 0, 'Z', Style::None)]);
        let mut h = Harness::new(2, 1);
        h.present(&buf, &Arena::new());

        h.invalidate();
        let out = h.frame_str(&buf, &Arena::new());
        assert!(out.contains("\x1B[2J"), "invalidate should ED2: {out:?}");
        assert!(out.contains('Z'), "should re-emit 'Z': {out:?}");
    }

    #[test]
    fn fullscreen_sync_output_wraps_frame() {
        let caps = Capabilities::builder().sync_output(true).build();
        let buf = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut h = Harness::new(3, 1).with_capabilities(caps);
        let out = h.frame_str(&buf, &Arena::new());
        assert!(out.starts_with("\x1B[?2026h"), "begin sync: {out:?}");
        assert!(out.ends_with("\x1B[?2026l"), "end sync: {out:?}");
    }

    #[test]
    fn fullscreen_hides_then_shows_cursor() {
        let buf = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut h = Harness::new(3, 1);
        let out = h.frame_str(&buf, &Arena::new());
        let hide = out.find("\x1B[?25l");
        let show = out.rfind("\x1B[?25h");
        assert!(hide.is_some() && show.is_some(), "hide/show present: {out:?}");
        assert!(hide < show, "hide before show: {out:?}");
    }

    // ── Inline ──────────────────────────────────────────────────────────

    #[test]
    fn inline_first_frame_has_no_ed_or_home() {
        let buf = Buffer::from_chars(3, 2, &[(0, 0, 'x', Style::None), (1, 0, 'y', Style::None)]);
        let mut h = Harness::inline(3, 2);
        let out = h.frame_str(&buf, &Arena::new());
        assert!(!out.contains("\x1B[2J"), "no ED2 inline: {out:?}");
        assert!(!out.contains("\x1B[H"), "no Home inline: {out:?}");
        assert!(out.contains('x') && out.contains('y'), "paints content: {out:?}");
    }

    #[test]
    fn inline_identical_second_frame_emits_no_content() {
        let buf = Buffer::from_chars(
            5,
            2,
            &[(0, 0, 'a', Style::None), (0, 1, 'b', Style::None), (1, 0, 'c', Style::None), (1, 1, 'd', Style::None)],
        );
        let mut h = Harness::inline(5, 2);
        h.present(&buf, &Arena::new());
        let out = h.frame_str(&buf, &Arena::new());
        assert!(!out.contains('a'), "should not re-emit 'a': {out:?}");
        assert!(!out.contains('c'), "should not re-emit 'c': {out:?}");
    }

    #[test]
    fn inline_diff_emits_only_changed_cell() {
        let buf1 = Buffer::from_chars(
            5,
            2,
            &[(0, 0, 'a', Style::None), (0, 1, 'b', Style::None), (1, 0, 'c', Style::None), (1, 1, 'd', Style::None)],
        );
        let mut h = Harness::inline(5, 2);
        h.present(&buf1, &Arena::new());

        let buf2 = Buffer::from_chars(
            5,
            2,
            &[(0, 0, 'a', Style::None), (0, 1, 'b', Style::None), (1, 0, 'X', Style::None), (1, 1, 'd', Style::None)],
        );
        let out = h.frame_str(&buf2, &Arena::new());
        assert!(out.contains('X'), "should emit changed cell: {out:?}");
        assert!(!out.contains('a'), "should not re-emit 'a': {out:?}");
        assert!(!out.contains('d'), "should not re-emit 'd': {out:?}");
    }

    #[test]
    fn inline_second_frame_rewinds_with_cuu() {
        let buf = Buffer::from_chars(
            5,
            3,
            &[(0, 0, 'a', Style::None), (1, 0, 'b', Style::None), (2, 0, 'c', Style::None)],
        );
        let mut h = Harness::inline(5, 3);
        h.present(&buf, &Arena::new());

        let buf2 = Buffer::from_chars(
            5,
            3,
            &[(0, 0, 'a', Style::None), (1, 0, 'X', Style::None), (2, 0, 'c', Style::None)],
        );
        let out = h.frame_str(&buf2, &Arena::new());
        assert!(out.contains("\x1B[2A") || out.contains("\x1B[A"), "should CUU to rewind: {out:?}");
    }

    #[test]
    fn inline_grow_claims_new_rows_with_newline() {
        let buf1 = Buffer::from_chars(3, 2, &[(0, 0, 'a', Style::None), (1, 0, 'b', Style::None)]);
        let mut h = Harness::inline(3, 2);
        h.present(&buf1, &Arena::new());

        let buf2 = Buffer::from_chars(
            3,
            3,
            &[(0, 0, 'a', Style::None), (1, 0, 'b', Style::None), (2, 0, 'c', Style::None)],
        );
        let out = h.frame_str(&buf2, &Arena::new());
        assert!(out.contains('\n'), "should emit newline to claim a row: {out:?}");
        assert!(out.contains('c'), "should paint the new row: {out:?}");
    }

    #[test]
    fn inline_shrink_clears_orphan_rows() {
        let buf1 = Buffer::from_chars(
            3,
            3,
            &[(0, 0, 'a', Style::None), (1, 0, 'b', Style::None), (2, 0, 'c', Style::None)],
        );
        let mut h = Harness::inline(3, 3);
        h.present(&buf1, &Arena::new());

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'a', Style::None)]);
        let out = h.frame_str(&buf2, &Arena::new());
        assert!(out.contains("\x1B[K"), "should EL orphan rows: {out:?}");
    }

    #[test]
    fn inline_invalidate_repaints_in_place() {
        // After invalidate (no resize) the inline region already exists on
        // screen, so the repaint must rewind (CUU) rather than re-claim with
        // newlines, and it must re-emit the content.
        let buf = Buffer::from_chars(
            5,
            2,
            &[(0, 0, 'a', Style::None), (1, 0, 'b', Style::None)],
        );
        let mut h = Harness::inline(5, 2);
        h.present(&buf, &Arena::new());

        h.invalidate();
        let out = h.frame_str(&buf, &Arena::new());
        assert!(out.contains("\x1B[A"), "should CUU to rewind, not re-claim: {out:?}");
        assert!(!out.contains('\n'), "must not claim new rows on repaint: {out:?}");
        assert!(out.contains('a') && out.contains('b'), "should repaint content: {out:?}");
    }

    #[test]
    fn inline_invalidate_clears_now_empty_row() {
        // Row 1 had content; after invalidate the new frame leaves it empty.
        // The fullscreen full-paint skips empty rows (EraseDisplay handles
        // them), but inline has no EraseDisplay, so the row must be EL-cleared.
        let buf1 = Buffer::from_chars(
            5,
            2,
            &[(0, 0, 'a', Style::None), (1, 0, 'b', Style::None)],
        );
        let mut h = Harness::inline(5, 2);
        h.present(&buf1, &Arena::new());

        let buf2 = Buffer::from_chars(5, 2, &[(0, 0, 'a', Style::None)]);
        h.invalidate();
        let out = h.frame_str(&buf2, &Arena::new());
        assert!(!out.contains('b'), "stale 'b' should not be re-emitted: {out:?}");
        // Two EL: one per row, including the now-empty row 1.
        assert_eq!(out.matches("\x1B[K").count(), 2, "every row must be EL-cleared: {out:?}");
    }

    // ── Style tracking across frames ────────────────────────────────────

    #[test]
    fn fullscreen_style_change_across_frames_emits_new_sgr() {
        let s1 = Style::default().foreground(Color::Rgb(255, 0, 0));
        let s2 = Style::default().foreground(Color::Rgb(0, 0, 255));
        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s1)]);
        let mut h = Harness::new(3, 1);
        h.present(&buf1, &Arena::new());

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s2)]);
        let out = h.frame_str(&buf2, &Arena::new());
        assert!(out.contains("38;2;0;0;255"), "should emit new fg: {out:?}");
    }

    // ── Tracking buffer (dirty-row) path ────────────────────────────────

    #[test]
    fn tracking_diff_emits_only_marked_changes() {
        use crate::TrackingBuffer;
        let mut arena = Arena::new();

        let prev = Buffer::from_chars(
            3,
            2,
            &[(0, 0, 'a', Style::None), (0, 1, 'b', Style::None), (1, 0, 'c', Style::None)],
        );

        // First frame: full paint to sync state.
        let mut presenter = Presenter::new(Cursor::new(Vec::new()));
        let blank = Buffer::new(3, 2);
        presenter.present(&blank, &prev, &arena).unwrap();
        let mark = presenter.get_writer().get_ref().len();

        // Change one cell on row 1, mark only row 1 dirty.
        let mut next = TrackingBuffer::from(prev.clone());
        next.unmark_all();
        next.set_line(Point { x: 0, y: 1 }, "X", &mut arena);

        presenter.present_tracking(&prev, &next, &arena).unwrap();
        let all = presenter.get_writer().get_ref();
        let frame = String::from_utf8_lossy(&all[mark..]).into_owned();

        assert!(frame.contains('X'), "should emit changed cell: {frame:?}");
        assert!(!frame.contains('a'), "should not touch row 0: {frame:?}");
    }
}
