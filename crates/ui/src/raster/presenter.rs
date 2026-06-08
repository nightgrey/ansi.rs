use utils::Counting;
use crate::raster::Pen;
use crate::{Arena, Buffer, BufferDiff, Cells, Cell, Run, TrackingBuffer};
use ansi::WriteEscape;
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
/// 3. Call [`present`](Self::present) (or [`present_tracking`](Self::present_tracked))
///    once per frame.
/// 4. On resize, alt-screen toggle, or external terminal corruption, call
///    [`invalidate`](Self::invalidate) to force a full repaint next frame.
#[derive(Debug)]
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
        self.writer.write_escape(AlternateScreen::Set)?;
        self.invalidated = true;
        Ok(())
    }

    /// Exit the alternate screen buffer.
    pub fn exit_alt_screen(&mut self) -> io::Result<()> {
        ansi::escape!(
            &mut self.writer,
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
    pub fn present_tracked(
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
    
    /// Clear the presenter's internal state, resetting the pen and invalidating
    /// the next frame.
    pub fn clear(&mut self) {
        self.writer.reset();
        self.pen.clear();
        self.invalidated = true;
        if let Some(inline) = self.inline.as_mut() {
            inline.is_first = true;
            inline.height = 0;
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Frame envelope
    // ────────────────────────────────────────────────────────────────────

    fn begin_frame(&mut self, _w: usize, _h: usize, force_full: bool) -> io::Result<()> {
        if self.capabilities.use_sync_output() {
            self.writer.write_escape(SynchronizedOutput::Set)?;
        }
        self.writer.write_escape(TextCursorEnable::Reset)?;

        if force_full {
            // Fullscreen homes + erases the screen; inline mode claims/rewinds
            // its own rows inside `emit_full`, so it must not touch the screen
            // here. Either way the full paint discharges the invalidation, so
            // clear the flag for both modes — otherwise inline would re-take
            // the full-paint path on every frame and never diff.
            if !self.is_inline() {
                self.writer.write_escape(Home)?;
                self.writer.write_escape(EraseDisplay)?;
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
        let cursor = self.writer.write_escape(TextCursorEnable::Set);
        let sync = if self.capabilities.use_sync_output() {
            self.writer.write_escape(SynchronizedOutput::Reset)
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

        // Fullscreen full-paint: the screen was just homed + erased, so blank
        // rows need no clearing and can be skipped. Write every row with
        // content *or* styling (a background is paintable even on an empty
        // cell) and EL its tail to drop any leftover content past the last.
        for y in 0..height {
            let row = &next[Row(y)];
            let Some(end) = Cells(row).last() else {
                continue;
            };
            self.move_pen(y as u16, 0)?;
            for col in 0..=end {
                emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                stats.cells += 1;
            }
            stats.runs += 1;
            self.pen.reset(&mut self.writer)?;
            self.writer.write_escape(EraseLineToEnd)?;
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
            if let Some(end) = Cells(row).last() {
                for col in 0..=end {
                    emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                    stats.cells += 1;
                }
                stats.runs += 1;
            }
            self.pen.reset(&mut self.writer)?;
            self.writer.write_escape(EraseLineToEnd)?;
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
            if let Some(end) = Cells(row).last() {
                for col in 0..=end {
                    emit_cell(&row[col], &mut self.writer, &mut self.pen, arena)?;
                    stats.cells += 1;
                }
                stats.runs += 1;
            }
            self.pen.reset(&mut self.writer)?;
            self.writer.write_escape(EraseLineToEnd)?;
        }

        // Track the final pen position so the next inline frame can CUU+CR
        // back to the top of the claimed region.
        self.pen.row = (height - 1) as u16;
        let last_row = &next[Row(height - 1)];
        self.pen.col = Cells(last_row).last().map_or(0, |end| end as u16 + 1);

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
        self.emit_run_loop(prev, next, arena, stats)
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
        // The run loop's cursor moves are mode-aware via `move_pen`, so the same
        // path serves fullscreen and inline.
        self.emit_run_loop(prev, next, arena, stats)?;
        self.inline_shrink(prev_height, next.height)
    }

    /// Walk the changed runs of `next` vs `prev`, emitting each one. Shared by
    /// the fullscreen and inline diff paths.
    fn emit_run_loop(
        &mut self,
        prev: &Buffer,
        next: &Buffer,
        arena: &Arena,
        stats: &mut PresenterStats,
    ) -> io::Result<()> {
        let mut runs = BufferDiff::runs(prev, next).peekable();
        while let Some(run) = runs.next() {
            self.move_pen(run.y, run.x)?;

            if self.emit_run(next, &run, arena, stats)? {
                // The run ended with an EL that cleared to the row's end, so any
                // remaining runs on this row are blank and already cleared.
                while runs.peek().is_some_and(|r| r.y == run.y) {
                    runs.next();
                }
                continue;
            }

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

    /// Emit one run's cells. If the run ends in blanks that extend to the end of
    /// the row, replace those trailing spaces with a single [`EraseLineToEnd`]
    /// (cheaper, and it clears to the row's end) and return `true` to signal the
    /// row is finished. Otherwise emit every cell and return `false`.
    fn emit_run(
        &mut self,
        next: &Buffer,
        run: &Run,
        arena: &Arena,
        stats: &mut PresenterStats,
    ) -> io::Result<bool> {
        let el_col = self.trailing_el_col(next, run);
        for change in run.iter() {
            if el_col.is_some_and(|col| change.x >= col) {
                break; // trailing blanks handled by the EL below
            }
            emit_cell(change.cell, &mut self.writer, &mut self.pen, arena)?;
            stats.cells += 1;
        }
        stats.runs += 1;

        if el_col.is_some() {
            // Reset SGR first so the erase clears to the default background
            // rather than smearing the pen's current colour (BCE).
            self.pen.reset(&mut self.writer)?;
            self.writer.write_escape(EraseLineToEnd)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Column at which an [`EraseLineToEnd`] should replace a run's trailing
    /// blank cells, or `None` if that is unsafe or not worth it.
    ///
    /// Safe only when every cell from the run's end to the row's end is already
    /// blank (so EL cannot wipe content that must stay), and worthwhile only
    /// when the blank suffix is longer than the escape itself.
    fn trailing_el_col(&self, next: &Buffer, run: &Run) -> Option<u16> {
        /// Byte length of `\x1B[K`.
        const EL_COST: u16 = 3;

        let width = next.width as u16;
        let y = run.y;
        let run_end = run.x + run.as_ref().len() as u16;

        // Cells past the run must already be blank for EL to be safe — they are
        // unchanged (== prev), so a blank here means the screen is blank there.
        for x in run_end..width {
            if !(next[Point { x, y }]).is_empty() {
                return None;
            }
        }

        // Walk back over the run's own trailing blank columns.
        let mut from = run_end;
        while from > run.x && (next[Point { x: from - 1, y }]).is_empty() {
            from -= 1;
        }

        (run_end - from > EL_COST).then_some(from)
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
            self.writer.write_escape(CursorUp(self.pen.row))?;
        }
        self.writer.write_escape(CarriageReturn)?;
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
            self.writer.write_escape(EraseLineToEnd)?;
        }
        if self.pen.row > (height - 1) as u16 {
            let up = self.pen.row - (height - 1) as u16;
            self.writer.write_escape(CursorUp(up))?;
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
        pen.col += cell.advance() as u16;
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
    fn fullscreen_diff_clears_row_with_erase_line() {
        let arena = Arena::new();
        let full: Vec<_> = (0..20).map(|c| (0usize, c, 'x', Style::None)).collect();
        let buf = Buffer::from_chars(20, 1, &full);
        let mut h = Harness::new(20, 1);
        h.present(&buf, &arena);

        // Clear the whole row: should collapse to one EL, not 20 spaces.
        let cleared = Buffer::new(20, 1);
        let out = h.frame_str(&cleared, &arena);
        assert!(out.contains("\x1B[K"), "should erase line: {out:?}");
        assert!(
            out.matches(' ').count() < 4,
            "trailing clear should not spam spaces: {out:?}"
        );
    }

    #[test]
    fn fullscreen_diff_shrink_uses_erase_line_for_tail() {
        let arena = Arena::new();
        let full: Vec<_> = (0..20).map(|c| (0usize, c, 'x', Style::None)).collect();
        let buf = Buffer::from_chars(20, 1, &full);
        let mut h = Harness::new(20, 1);
        h.present(&buf, &arena);

        // Keep the first two cells, clear the rest to the row end.
        let shrunk = Buffer::from_chars(20, 1, &[(0, 0, 'x', Style::None), (0, 1, 'x', Style::None)]);
        let out = h.frame_str(&shrunk, &arena);
        assert!(out.contains("\x1B[K"), "should EL the cleared tail: {out:?}");
        assert!(
            out.matches(' ').count() < 4,
            "should not emit a space per cleared cell: {out:?}"
        );
    }

    #[test]
    fn fullscreen_diff_no_erase_line_when_tail_has_content() {
        let arena = Arena::new();
        let buf = Buffer::from_chars(
            5,
            1,
            &[(0, 0, 'a', Style::None), (0, 1, 'b', Style::None), (0, 4, 'Z', Style::None)],
        );
        let mut h = Harness::new(5, 1);
        h.present(&buf, &arena);

        // Clear b (col 1) but col 4 'Z' stays: a blank gap mid-row must not be
        // turned into an EL (that would wipe 'Z').
        let next = Buffer::from_chars(5, 1, &[(0, 0, 'a', Style::None), (0, 4, 'Z', Style::None)]);
        let out = h.frame_str(&next, &arena);
        assert!(!out.contains("\x1B[K"), "must not EL across surviving content: {out:?}");
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

        presenter.present_tracked(&prev, &next, &arena).unwrap();
        let all = presenter.get_writer().get_ref();
        let frame = String::from_utf8_lossy(&all[mark..]).into_owned();

        assert!(frame.contains('X'), "should emit changed cell: {frame:?}");
        assert!(!frame.contains('a'), "should not touch row 0: {frame:?}");
    }


    #[cfg(test)]
    mod emulator {
        // Roundtrip correctness oracle for the [`Presenter`].
        //
        // Instead of asserting that the presenter emits particular escape *strings*,
        // these tests assert that the bytes it emits, when fed through `ansi`'s VTE
        // parser into a virtual terminal grid, reconstruct the source buffer. That
        // validates the presenter's actual contract — "bring the terminal from `prev`
        // to `next`" — at the level of *where glyphs and styles land*, which substring
        // checks cannot do.
        //
        // ## Pipeline
        //
        // ```text
        // Buffer ──Presenter──▶ bytes ──ansi::Parser──▶ Term grid ──assert==──▶ Buffer
        // ```
        //
        // The [`Emulator`] is persisted across frames (it is a real terminal model), so a
        // diff frame is correct iff applying it to the prior screen yields `next`.
        //
        // ## Modelling assumptions
        //
        // - **LF (`\n`) is a newline** (cursor down + column 0). The presenter's inline
        //   scrollback-claim relies on this (ONLCR); modelling LF as pure line-feed
        //   would make inline reconstruction wrong, so we mirror the presenter's own
        //   assumption.
        // - **An empty cell with a style is a styled space, not a blank.** A cell with
        //   a background but no glyph is paintable on every path (`Cell::is_blank` is
        //   the "truly nothing" predicate). We do *not* rely on "erase with background
        //   colour" (BCE): the generators never put a style on a cell that the
        //   presenter would clear with `EL` rather than overwrite.

        use super::Presenter;
        use crate::{Arena, Buffer, Cell, TrackingBuffer};
        use ansi::parser::{Handler, ByteStr, Params, Parser};
        use ansi::{Attribute, Color, Style};
        use std::io::Cursor;
        use unicode_width::UnicodeWidthChar;

        // ─────────────────────────────────────────────────────────────────────────
        // Virtual terminal
        // ─────────────────────────────────────────────────────────────────────────

        /// One reconstructed grid cell, normalised for comparison.
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Glyph {
            /// Unwritten, or a default space — visually blank.
            Blank,
            /// Tail cell of a wide glyph.
            Continuation,
            /// A printed base glyph.
            Char(char),
        }

        #[derive(Clone, Copy)]
        struct EmulatorCell {
            glyph: Glyph,
            style: Style,
        }

        impl EmulatorCell {
            const BLANK: Self = Self {
                glyph: Glyph::Blank,
                style: Style::None,
            };
        }

        /// Minimal VTE handler: tracks a cursor, a pen style, and a grid that grows
        /// downward on demand (for inline scrollback).
        struct Emulator {
            width: usize,
            rows: usize,
            cells: Vec<EmulatorCell>,
            cx: usize,
            cy: usize,
            style: Style,
        }

        impl Emulator {
            fn new(width: usize) -> Self {
                Self {
                    width,
                    rows: 0,
                    cells: Vec::new(),
                    cx: 0,
                    cy: 0,
                    style: Style::None,
                }
            }

            fn ensure_row(&mut self, y: usize) {
                while self.rows <= y {
                    self.cells.extend(std::iter::repeat_n(EmulatorCell::BLANK, self.width));
                    self.rows += 1;
                }
            }

            fn get(&self, x: usize, y: usize) -> EmulatorCell {
                if y >= self.rows || x >= self.width {
                    return EmulatorCell::BLANK;
                }
                self.cells[y * self.width + x]
            }

            fn clear_all(&mut self) {
                for c in &mut self.cells {
                    *c = EmulatorCell::BLANK;
                }
            }

            /// Erase from the cursor to the end of the current row (`CSI K`).
            fn erase_line_to_end(&mut self) {
                if self.cy >= self.rows {
                    return;
                }
                for x in self.cx..self.width {
                    self.cells[self.cy * self.width + x] = EmulatorCell::BLANK;
                }
            }

            /// Flat list of SGR parameters (the presenter emits one value per `;`
            /// group, so concatenating groups recovers the parameter sequence).
            fn apply_sgr(&mut self, p: &[u16]) {
                if p.is_empty() {
                    self.style = Style::None; // `CSI m` == `CSI 0 m`
                    return;
                }
                let mut i = 0;
                while i < p.len() {
                    match p[i] {
                        0 => self.style = Style::None,
                        1 => self.style.attributes.insert(Attribute::Bold),
                        2 => self.style.attributes.insert(Attribute::Faint),
                        3 => self.style.attributes.insert(Attribute::Italic),
                        4 => self.style.attributes.insert(Attribute::Underline),
                        22 => self
                            .style
                            .attributes
                            .remove(Attribute::Bold | Attribute::Faint),
                        23 => self.style.attributes.remove(Attribute::Italic),
                        24 => self.style.attributes.remove(
                            Attribute::Underline | Attribute::UnderlineDouble | Attribute::UnderlineCurly,
                        ),
                        30..=37 => self.style.foreground = basic_color(p[i] - 30),
                        39 => self.style.foreground = Color::None,
                        40..=47 => self.style.background = basic_color(p[i] - 40),
                        49 => self.style.background = Color::None,
                        90..=97 => self.style.foreground = bright_color(p[i] - 90),
                        100..=107 => self.style.background = bright_color(p[i] - 100),
                        38 => {
                            self.style.foreground = read_extended_color(p, &mut i);
                        }
                        48 => {
                            self.style.background = read_extended_color(p, &mut i);
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
        }

        /// Decode `38;5;n` / `38;2;r;g;b` (or `48;…`), advancing `i` past the consumed
        /// values. `i` points at the `38`/`48` introducer on entry.
        fn read_extended_color(p: &[u16], i: &mut usize) -> Color {
            match p.get(*i + 1).copied() {
                Some(5) => {
                    let idx = p.get(*i + 2).copied().unwrap_or(0) as u8;
                    *i += 2;
                    Color::Index(idx)
                }
                Some(2) => {
                    let r = p.get(*i + 2).copied().unwrap_or(0) as u8;
                    let g = p.get(*i + 3).copied().unwrap_or(0) as u8;
                    let b = p.get(*i + 4).copied().unwrap_or(0) as u8;
                    *i += 4;
                    Color::Rgb(r, g, b)
                }
                _ => Color::None,
            }
        }

        fn basic_color(n: u16) -> Color {
            match n {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                _ => Color::White,
            }
        }

        fn bright_color(n: u16) -> Color {
            match n {
                0 => Color::BrightBlack,
                1 => Color::BrightRed,
                2 => Color::BrightGreen,
                3 => Color::BrightYellow,
                4 => Color::BrightBlue,
                5 => Color::BrightMagenta,
                6 => Color::BrightCyan,
                _ => Color::BrightWhite,
            }
        }

        /// Read a CSI numeric arg, treating absent or `0` as `default` (ANSI cursor
        /// ops use a default of 1).
        fn arg(p: &[u16], i: usize, default: u16) -> u16 {
            match p.get(i).copied() {
                None | Some(0) => default,
                Some(v) => v,
            }
        }

        impl Handler for Emulator {
            fn printable(&mut self, c: char) {
                let w = c.width().unwrap_or(0);
                if w == 0 {
                    return; // zero-width (combining) — our generators don't produce these
                }
                self.ensure_row(self.cy);
                if self.cx < self.width {
                    let base = self.cy * self.width + self.cx;
                    self.cells[base] = EmulatorCell {
                        glyph: Glyph::Char(c),
                        style: self.style,
                    };
                    for k in 1..w {
                        if self.cx + k < self.width {
                            self.cells[base + k] = EmulatorCell {
                                glyph: Glyph::Continuation,
                                style: self.style,
                            };
                        }
                    }
                }
                self.cx += w;
            }

            fn execute(&mut self, byte: u8) {
                match byte {
                    0x0A => {
                        // LF → newline (see module docs).
                        self.cy += 1;
                        self.cx = 0;
                        self.ensure_row(self.cy);
                    }
                    0x0D => self.cx = 0,
                    0x08 => self.cx = self.cx.saturating_sub(1),
                    _ => {}
                }
            }

            fn csi(&mut self, params: Params<'_>, _intermediates: &ByteStr, final_char: char) {
                let p: &[u16] = params.as_ref();
                match final_char {
                    'A' => self.cy = self.cy.saturating_sub(arg(p, 0, 1) as usize),
                    'B' | 'e' => {
                        self.cy += arg(p, 0, 1) as usize;
                        self.ensure_row(self.cy);
                    }
                    'C' | 'a' => self.cx += arg(p, 0, 1) as usize,
                    'D' => self.cx = self.cx.saturating_sub(arg(p, 0, 1) as usize),
                    'G' | '`' => self.cx = (arg(p, 0, 1) - 1) as usize,
                    'd' => self.cy = (arg(p, 0, 1) - 1) as usize,
                    'H' | 'f' => {
                        self.cy = (arg(p, 0, 1) - 1) as usize;
                        self.cx = (arg(p, 1, 1) - 1) as usize;
                        self.ensure_row(self.cy);
                    }
                    'J' => match p.first().copied().unwrap_or(0) {
                        2 | 3 => {
                            self.clear_all();
                            // ED does not move the cursor.
                        }
                        _ => {
                            // Erase from cursor to end of display.
                            self.erase_line_to_end();
                            for y in (self.cy + 1)..self.rows {
                                for x in 0..self.width {
                                    self.cells[y * self.width + x] = EmulatorCell::BLANK;
                                }
                            }
                        }
                    },
                    'K' => match p.first().copied().unwrap_or(0) {
                        1 => {
                            if self.cy < self.rows {
                                for x in 0..=self.cx.min(self.width.saturating_sub(1)) {
                                    self.cells[self.cy * self.width + x] = EmulatorCell::BLANK;
                                }
                            }
                        }
                        2 => {
                            if self.cy < self.rows {
                                for x in 0..self.width {
                                    self.cells[self.cy * self.width + x] = EmulatorCell::BLANK;
                                }
                            }
                        }
                        _ => self.erase_line_to_end(),
                    },
                    'm' => self.apply_sgr(p),
                    // Modes (`?…h/l`), cursor visibility, sync output, etc. — irrelevant
                    // to the reconstructed grid.
                    _ => {}
                }
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Comparison
        // ─────────────────────────────────────────────────────────────────────────

        /// Canonical, comparison-ready view of a cell. Empty cells and unstyled spaces
        /// both canonicalise to `Blank`, so the diff's "write a space" and the full
        /// paint's "skip / EL" agree.
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Canon {
            Blank,
            Continuation,
            Glyph(char, Style),
        }

        fn canon_term(c: EmulatorCell) -> Canon {
            match c.glyph {
                Glyph::Blank => Canon::Blank,
                Glyph::Continuation => Canon::Continuation,
                Glyph::Char(' ') if c.style == Style::None => Canon::Blank,
                Glyph::Char(ch) => Canon::Glyph(ch, c.style),
            }
        }

        fn canon_cell(cell: &Cell, arena: &Arena) -> Canon {
            if cell.is_continuation() {
                return Canon::Continuation;
            }
            if cell.is_empty() {
                // Empty *and* unstyled — nothing to paint.
                return Canon::Blank;
            }
            if cell.is_space() {
                // Empty but styled (e.g. a background) — painted as a styled space.
                return Canon::Glyph(' ', *cell.style());
            }
            let s = cell.as_str(arena);
            let ch = s.chars().next().unwrap_or(' ');
            if ch == ' ' && cell.style().is_none() {
                return Canon::Blank;
            }
            Canon::Glyph(ch, *cell.style())
        }

        fn describe(c: Canon) -> String {
            match c {
                Canon::Blank => "·".into(),
                Canon::Continuation => "▸".into(),
                Canon::Glyph(ch, _) => ch.to_string(),
            }
        }

        /// A self-tracking driver: owns `prev`, the presenter, the parser, and the
        /// virtual terminal, and slices out the bytes emitted per frame.
        struct Roundtrip {
            presenter: Presenter<Cursor<Vec<u8>>>,
            parser: Parser,
            term: Emulator,
            prev: Buffer,
            mark: usize,
        }

        impl Roundtrip {
            fn fullscreen(width: usize, height: usize) -> Self {
                Self {
                    presenter: Presenter::new(Cursor::new(Vec::new())),
                    parser: Parser::default(),
                    term: Emulator::new(width),
                    prev: Buffer::new(width, height),
                    mark: 0,
                }
            }

            fn inline(width: usize, height: usize) -> Self {
                Self {
                    presenter: Presenter::inline(Cursor::new(Vec::new())),
                    parser: Parser::default(),
                    term: Emulator::new(width),
                    prev: Buffer::new(width, height),
                    mark: 0,
                }
            }

            fn invalidate(&mut self) {
                self.presenter.invalidate();
            }

            /// Present `next`, feed the frame's bytes through the parser, then assert
            /// the reconstructed grid matches `next`.
            fn frame(&mut self, next: &Buffer, arena: &Arena) {
                assert_eq!(self.term.width, next.width, "width must be stable");
                if self.prev.width != next.width || self.prev.height != next.height {
                    self.prev.resize(next.width, next.height);
                    self.prev.clear();
                }
                self.presenter.present(&self.prev, next, arena).unwrap();
                self.prev.copy_from_slice(next.as_ref());

                let all = self.presenter.get_writer().get_ref();
                let bytes = all[self.mark..].to_vec();
                self.mark = all.len();
                let parser = &mut self.parser;
                parser.advance(&mut self.term, &bytes);

                assert_grid(&self.term, next, arena, &bytes);
            }
        }

        /// Assert the reconstructed terminal grid matches `next` cell-for-cell.
        fn assert_grid(term: &Emulator, next: &Buffer, arena: &Arena, bytes: &[u8]) {
            for y in 0..next.height {
                for x in 0..next.width {
                    let cell = &next[geometry::Point {
                        x: x as u16,
                        y: y as u16,
                    }];
                    let expected = canon_cell(cell, arena);
                    let actual = canon_term(term.get(x, y));
                    if expected != actual {
                        let w = next.width;
                        let h = next.height;
                        panic!(
                            "mismatch at ({x},{y}): expected {} got {}\n\
                     expected grid:\n{}\nreconstructed grid:\n{}\nbytes: {:?}",
                            describe(expected),
                            describe(actual),
                            dump(w, h, |y2, x2| canon_cell(
                                &next[geometry::Point {
                                    x: x2 as u16,
                                    y: y2 as u16,
                                }],
                                arena
                            )),
                            dump(w, h, |y2, x2| canon_term(term.get(x2, y2))),
                            String::from_utf8_lossy(bytes),
                        );
                    }
                }
            }
        }

        /// Render a `height × width` grid to text via `cell(y, x) -> Canon`.
        fn dump(width: usize, height: usize, cell: impl Fn(usize, usize) -> Canon) -> String {
            let mut s = String::new();
            for y in 0..height {
                for x in 0..width {
                    s.push_str(&describe(cell(y, x)));
                }
                s.push('\n');
            }
            s
        }

        /// Roundtrip driver for [`Presenter::present_tracked`].
        ///
        /// Mirrors the real dirty-tracking contract: the caller writes changes into a
        /// [`TrackingBuffer`] (whose `IndexMut` marks the touched row), presents, then
        /// `unmark_all`s. We mark exactly the rows we change, so the test exercises the
        /// presenter — not a mis-marking caller.
        struct TrackingEmulator {
            presenter: Presenter<Cursor<Vec<u8>>>,
            parser: Parser,
            term: Emulator,
            prev: Buffer,
            tracking: TrackingBuffer,
            mark: usize,
        }

        impl TrackingEmulator {
            fn new(width: usize, height: usize) -> Self {
                Self {
                    presenter: Presenter::new(Cursor::new(Vec::new())),
                    parser: Parser::default(),
                    term: Emulator::new(width),
                    prev: Buffer::new(width, height),
                    tracking: TrackingBuffer::new_unmarked(width, height),
                    mark: 0,
                }
            }

            fn invalidate(&mut self) {
                self.presenter.invalidate();
            }

            fn frame(&mut self, next: &Buffer, arena: &Arena) {
                assert_eq!(self.term.width, next.width, "width must be stable");

                // Apply `next` into the tracking buffer, marking only the rows that
                // actually change (writing via `Point` marks that row).
                for y in 0..next.height {
                    for x in 0..next.width {
                        let p = geometry::Point {
                            x: x as u16,
                            y: y as u16,
                        };
                        let cell = next[p];
                        if self.tracking[p] != cell {
                            self.tracking[p] = cell;
                        }
                    }
                }

                self.presenter
                    .present_tracked(&self.prev, &self.tracking, arena)
                    .unwrap();

                self.prev.copy_from_slice(self.tracking.as_inner().as_ref());
                self.tracking.unmark_all();

                let all = self.presenter.get_writer().get_ref();
                let bytes = all[self.mark..].to_vec();
                self.mark = all.len();
                self.parser.advance(&mut self.term, &bytes);

                assert_grid(&self.term, next, arena, &bytes);
            }
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Buffer builders (visible glyphs only; empty cells stay unstyled)
        // ─────────────────────────────────────────────────────────────────────────

        /// Build a buffer from rows of `(char, Style)`; `'\0'` means an empty cell.
        fn grid(rows: &[Vec<(char, Style)>]) -> Buffer {
            let height = rows.len();
            let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            let mut buf = Buffer::new(width, height);
            for (y, row) in rows.iter().enumerate() {
                for (x, &(ch, style)) in row.iter().enumerate() {
                    if ch != '\0' {
                        buf[(y, x)] = Cell::inline(ch).with_style(style);
                    }
                }
            }
            buf
        }

        /// Deterministic pseudo-random buffer of ASCII glyphs and a few styles. `fill`
        /// is the fraction (0..=100) of cells that are non-empty.
        fn pseudo_random(width: usize, height: usize, seed: u64, fill: u64) -> Buffer {
            let palette = [
                Style::None,
                Style::None.foreground(Color::Red),
                Style::None.foreground(Color::Rgb(10, 200, 30)),
                Style::None.background(Color::Blue),
                Style::None.with(Attribute::Bold),
                Style::None
                    .foreground(Color::BrightCyan)
                    .with(Attribute::Italic),
            ];
            let glyphs = b"abcdefgABCDEFG0123 .#@";
            let mut state = seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1);
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state
            };
            let backgrounds = [Color::Rgb(20, 20, 20), Color::Blue, Color::Index(238)];
            let mut buf = Buffer::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    if next() % 100 < fill {
                        let ch = glyphs[(next() as usize) % glyphs.len()] as char;
                        let style = palette[(next() as usize) % palette.len()];
                        buf[(y, x)] = Cell::inline(ch).with_style(style);
                    } else if next() % 4 == 0 {
                        // An empty cell carrying only a background — must paint as a
                        // styled space on every path.
                        let bg = backgrounds[(next() as usize) % backgrounds.len()];
                        buf[(y, x)] = Cell::default().with_background(bg);
                    }
                }
            }
            buf
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Tests
        // ─────────────────────────────────────────────────────────────────────────

        #[test]
        fn fullscreen_single_frame_styles_and_gaps() {
            let arena = Arena::new();
            let s_red = Style::None.foreground(Color::Red);
            let s_bg = Style::None.background(Color::Rgb(0, 0, 128));
            let buf = grid(&[
                vec![('H', s_red), ('i', s_red), ('\0', Style::None), ('!', Style::None)],
                vec![('\0', Style::None); 4],
                vec![(' ', s_bg), (' ', s_bg), ('x', Style::None), ('\0', Style::None)],
            ]);
            let mut rt = Roundtrip::fullscreen(4, 3);
            rt.frame(&buf, &arena);
        }

        #[test]
        fn fullscreen_diff_sequence() {
            let arena = Arena::new();
            let a = grid(&[
                vec![('a', Style::None), ('b', Style::None), ('c', Style::None)],
                vec![('d', Style::None), ('e', Style::None), ('f', Style::None)],
            ]);
            let b = grid(&[
                vec![('a', Style::None), ('X', Style::None), ('c', Style::None)],
                vec![('d', Style::None), ('e', Style::None), ('f', Style::None)],
            ]);
            let c = grid(&[
                vec![('a', Style::None), ('X', Style::None), ('\0', Style::None)],
                vec![('\0', Style::None), ('Y', Style::None), ('f', Style::None)],
            ]);
            let mut rt = Roundtrip::fullscreen(3, 2);
            rt.frame(&a, &arena);
            rt.frame(&b, &arena);
            rt.frame(&c, &arena);
            rt.frame(&a, &arena);
        }

        #[test]
        fn fullscreen_invalidate_then_diff() {
            let arena = Arena::new();
            let a = pseudo_random(12, 5, 1, 60);
            let b = pseudo_random(12, 5, 2, 60);
            let mut rt = Roundtrip::fullscreen(12, 5);
            rt.frame(&a, &arena);
            rt.invalidate();
            rt.frame(&a, &arena);
            rt.frame(&b, &arena);
        }

        #[test]
        fn fullscreen_property_random_sequences() {
            let arena = Arena::new();
            for seed in 0..40u64 {
                let mut rt = Roundtrip::fullscreen(16, 8);
                let mut s = seed;
                for _ in 0..6 {
                    let fill = 30 + (s % 60);
                    let buf = pseudo_random(16, 8, s, fill);
                    rt.frame(&buf, &arena);
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                }
            }
        }

        #[test]
        fn fullscreen_wide_glyphs() {
            let arena = Arena::new();
            // '中' is width 2; place it so a continuation cell follows.
            let mut a = Buffer::new(6, 2);
            a[(0, 0)] = Cell::inline('中');
            a[(0, 1)] = Cell::CONTINUATION;
            a[(0, 3)] = Cell::inline('x');
            a[(1, 0)] = Cell::inline('世');
            a[(1, 1)] = Cell::CONTINUATION;

            let mut rt = Roundtrip::fullscreen(6, 2);
            rt.frame(&a, &arena);

            // Change the narrow tail; the wide glyph must stay intact.
            let mut b = a.clone();
            b[(0, 3)] = Cell::inline('Z');
            rt.frame(&b, &arena);
        }

        #[test]
        fn inline_single_and_diff() {
            let arena = Arena::new();
            let a = grid(&[
                vec![('a', Style::None), ('b', Style::None), ('\0', Style::None)],
                vec![('c', Style::None), ('d', Style::None), ('\0', Style::None)],
            ]);
            let b = grid(&[
                vec![('a', Style::None), ('b', Style::None), ('\0', Style::None)],
                vec![('c', Style::None), ('Z', Style::None), ('\0', Style::None)],
            ]);
            let mut rt = Roundtrip::inline(3, 2);
            rt.frame(&a, &arena);
            rt.frame(&b, &arena);
            rt.frame(&a, &arena);
        }

        #[test]
        fn inline_grow_and_shrink() {
            let arena = Arena::new();
            let two = grid(&[
                vec![('a', Style::None)],
                vec![('b', Style::None)],
            ]);
            let four = grid(&[
                vec![('a', Style::None)],
                vec![('b', Style::None)],
                vec![('c', Style::None)],
                vec![('d', Style::None)],
            ]);
            let one = grid(&[vec![('a', Style::None)]]);

            let mut rt = Roundtrip::inline(1, 2);
            rt.frame(&two, &arena);
            rt.frame(&four, &arena);
            rt.frame(&one, &arena);
            rt.frame(&four, &arena);
        }

        #[test]
        fn inline_invalidate_repaints_in_place() {
            let arena = Arena::new();
            let a = grid(&[
                vec![('a', Style::None), ('b', Style::None)],
                vec![('c', Style::None), ('d', Style::None)],
            ]);
            // After invalidate, a frame that empties row 1 must clear it on screen.
            let b = grid(&[
                vec![('a', Style::None), ('b', Style::None)],
                vec![('\0', Style::None), ('\0', Style::None)],
            ]);
            let mut rt = Roundtrip::inline(2, 2);
            rt.frame(&a, &arena);
            rt.invalidate();
            rt.frame(&b, &arena);
        }

        #[test]
        fn inline_property_random_sequences() {
            let arena = Arena::new();
            for seed in 0..30u64 {
                let mut rt = Roundtrip::inline(10, 4);
                let mut s = seed.wrapping_add(7);
                for _ in 0..5 {
                    let fill = 40 + (s % 50);
                    let buf = pseudo_random(10, 4, s, fill);
                    rt.frame(&buf, &arena);
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                }
            }
        }

        // ── present_tracking (dirty-row) ────────────────────────────────────────

        #[test]
        fn tracking_single_and_diff() {
            let arena = Arena::new();
            let a = grid(&[
                vec![('a', Style::None), ('b', Style::None), ('c', Style::None)],
                vec![('d', Style::None), ('e', Style::None), ('f', Style::None)],
            ]);
            // Only row 1 changes — the unmarked row 0 must be left untouched on screen.
            let b = grid(&[
                vec![('a', Style::None), ('b', Style::None), ('c', Style::None)],
                vec![('d', Style::None), ('X', Style::None), ('f', Style::None)],
            ]);
            let mut rt = TrackingEmulator::new(3, 2);
            rt.frame(&a, &arena);
            rt.frame(&b, &arena);
            rt.frame(&a, &arena);
        }

        #[test]
        fn tracking_clears_and_backgrounds() {
            let arena = Arena::new();
            let s_bg = Style::None.background(Color::Rgb(30, 30, 30));
            let a = grid(&[
                vec![('x', Style::None), ('y', Style::None), ('z', Style::None)],
                vec![(' ', s_bg), (' ', s_bg), (' ', s_bg)],
            ]);
            // Clear row 0 to blank; recolour row 1.
            let b = grid(&[
                vec![('\0', Style::None), ('\0', Style::None), ('\0', Style::None)],
                vec![('p', Style::None), ('\0', Style::None), ('q', Style::None)],
            ]);
            let mut rt = TrackingEmulator::new(3, 2);
            rt.frame(&a, &arena);
            rt.frame(&b, &arena);
        }

        #[test]
        fn tracking_invalidate_forces_full_repaint() {
            let arena = Arena::new();
            let a = pseudo_random(12, 5, 3, 60);
            let mut rt = TrackingEmulator::new(12, 5);
            rt.frame(&a, &arena);
            // Invalidate: even with no rows marked, the next frame must full-repaint.
            rt.invalidate();
            rt.frame(&a, &arena);
        }

        #[test]
        fn tracking_property_random_sequences() {
            let arena = Arena::new();
            for seed in 0..40u64 {
                let mut rt = TrackingEmulator::new(16, 6);
                let mut s = seed.wrapping_add(11);
                for _ in 0..6 {
                    let fill = 30 + (s % 60);
                    let buf = pseudo_random(16, 6, s, fill);
                    rt.frame(&buf, &arena);
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                }
            }
        }

        /// Empty-but-styled cells (a background with no glyph, as produced by
        /// `buffer_solid`/`buffer_chessboard`) must paint as styled spaces on *every*
        /// path — full paint, inline claim, and diff. Earlier the full-paint scan used
        /// `is_empty()` (glyph-only) and dropped these; it now uses `is_blank()`
        /// (glyph *and* style), so a background survives a full repaint.
        #[test]
        fn empty_background_cells_paint_on_all_paths() {
            let arena = Arena::new();
            let grey = Color::Rgb(40, 40, 40);
            let solid = Buffer::from_fn(4, 2, |_, _| Cell::default().with_background(grey));

            // Full paint (first frame).
            let mut rt = Roundtrip::fullscreen(4, 2);
            rt.frame(&solid, &arena);

            // Diff: recolour one cell, leave the rest as background.
            let mut next = solid.clone();
            next[(1, 2)] = Cell::default().with_background(Color::Red);
            rt.frame(&next, &arena);

            // Inline claim + inline diff.
            let mut inl = Roundtrip::inline(4, 2);
            inl.frame(&solid, &arena);
            inl.frame(&next, &arena);
        }
    }
}
