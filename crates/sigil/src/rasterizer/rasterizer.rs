use std::io;
use ansi::escape;
use ansi::io::Write;
use ansi::sequences::*;
use grid::Row;

use crate::buffer::{Buffer, GraphemeArena};
use crate::Cell;
use super::capabilities::Capabilities;
use super::cursor::Cursor;

/// Whether the rasterizer operates in fullscreen or inline mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterizerMode {
    /// Traditional fullscreen mode using alternate screen buffer.
    Fullscreen,
    /// Inline/CLI mode: renders within the normal scrollback region.
    Inline,
}

/// State for inline rendering.
#[derive(Debug, Clone, Copy)]
struct InlineState {
    /// Number of rows the rasterizer "owns" in the terminal.
    owned_rows: usize,
    /// Whether this is the first render call.
    first_render: bool,
}

/// How the cursor should be positioned before emitting cells.
#[derive(Clone, Copy)]
enum CursorMode {
    /// Use absolute or optimized movement (fullscreen).
    Absolute(Capabilities),
    /// Use only relative movement (inline).
    Relative,
}

#[derive(Debug, Clone)]
pub struct Rasterizer {
    output: Vec<u8>,
    previous: Buffer,
    pen: Cursor,
    caps: Capabilities,
    is_fullscreen: bool,
    invalidated: bool,
    mode: RasterizerMode,
    inline: Option<InlineState>,
}

impl Rasterizer {
    /// Create a new rasterizer with the given screen dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            output: Vec::with_capacity(4096),
            previous: Buffer::new(width, height),
            pen: Cursor::new(),
            caps: Capabilities::default(),
            is_fullscreen: false,
            invalidated: true,
            mode: RasterizerMode::Fullscreen,
            inline: None,
        }
    }

    /// Create a rasterizer with explicit capabilities.
    pub fn with_capabilities(width: usize, height: usize, caps: Capabilities) -> Self {
        Self {
            caps,
            ..Self::new(width, height)
        }
    }

    /// Create a rasterizer in inline mode.
    pub fn inline(width: usize, height: usize) -> Self {
        Self {
            mode: RasterizerMode::Inline,
            inline: Some(InlineState {
                owned_rows: 0,
                first_render: true,
            }),
            ..Self::new(width, height)
        }
    }

    /// Create an inline rasterizer with explicit capabilities.
    pub fn inline_with_capabilities(width: usize, height: usize, caps: Capabilities) -> Self {
        Self {
            caps,
            ..Self::inline(width, height)
        }
    }

    /// Write a single cell's content, updating the pen first.
    #[inline]
    fn render_cell(cell: &Cell, output: &mut Vec<u8>, cursor: &mut Cursor, arena: &GraphemeArena) {
        cursor.update_style(output, cell.style());
        output.extend_from_slice(cell.as_bytes(arena));
    }

    /// Diff a single line between `next` (new) and `previous` (old), emitting only
    /// the changed cells. Uses left→right and right→left scanning to find the
    /// minimal dirty region, plus a trailing EL optimization.
    fn diff_line(
        previous: &Buffer,
        next: &Buffer,
        output: &mut Vec<u8>,
        cursor: &mut Cursor,
        cursor_mode: CursorMode,
        y: usize,
        width: usize,
        arena: &GraphemeArena,
    ) {
        let new_row: &[Cell] = &next[Row(y)];
        let old_row: &[Cell] = &previous[Row(y)];

        // Scan left→right for first differing cell.
        let first = match (0..width).find(|&x| new_row[x] != old_row[x]) {
            Some(col) => col,
            None => return, // Entire line is identical.
        };

        // Scan right→left for last differing cell.
        let last = (0..width)
            .rev()
            .find(|&x| new_row[x] != old_row[x])
            .unwrap();

        // Trailing EL optimization: within the diff range [first, last], find the
        // last cell that has non-empty new content. If the tail of the diff range
        // consists of empty new cells (replacing old content), we can use EL
        // instead of emitting spaces.
        let last_content_in_range = (first..=last)
            .rev()
            .find(|&x| !new_row[x].is_empty());

        // Move cursor to start of changed region.
        match cursor_mode {
            CursorMode::Absolute(caps) => cursor.to(y, first, output, caps),
            CursorMode::Relative => cursor.to_relative(y, first, output),
        }

        match last_content_in_range {
            None => {
                // Entire diff range is now empty — just erase to end of line.
                cursor.reset_style(output);
                escape!(output, EraseLineToEnd).unwrap();
            }
            Some(emit_end) => {
                let need_eol = emit_end < last;

                // Emit changed cells from first through emit_end.
                let mut col = first;
                while col <= emit_end {
                    let cell = &new_row[col];
                    Self::render_cell(cell, output, cursor, arena);
                    let w = cell.columns() as usize;
                    col += w;
                    cursor.col += w;
                }

                // Clear to end of line if trailing cells transitioned to empty.
                if need_eol {
                    cursor.reset_style(output);
                    escape!(output, EraseLineToEnd).unwrap();
                }
            }
        }
    }

    /// Render a buffer, diffing against the previous frame.
    pub fn render(&mut self, buffer: &Buffer, arena: &GraphemeArena) {
        if self.mode == RasterizerMode::Inline {
            return self.render_inline(buffer, arena);
        }
        let width = buffer.width;
        let height = buffer.height;

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Enable).unwrap();
        }

        // Handle dimension change or forced clear.
        if self.previous.width != width || self.previous.height != height || self.invalidated {
            escape!(&mut self.output, Home).unwrap();
            escape!(&mut self.output, EraseDisplay).unwrap();
            self.pen.reset();
            self.previous = Buffer::new(width, height);
            self.invalidated = false;
        }

        escape!(&mut self.output, TextCursor::Disable).unwrap();

        let cursor_mode = CursorMode::Absolute(self.caps);
        for y in 0..height {
            Self::diff_line(
                &self.previous,
                buffer,
                &mut self.output,
                &mut self.pen,
                cursor_mode,
                y,
                width,
                arena,
            );
        }

        self.pen.reset_style(&mut self.output);
        escape!(&mut self.output, TextCursor::Enable).unwrap();

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Disable).unwrap();
        }

        // Swap prev ← new.
        self.previous.copy_from_slice(buffer);
    }

    /// Flush the accumulated output to a writer and clear the buffer.
    pub fn flush(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        if !self.output.is_empty() {
            w.write_all(&self.output)?;
            self.output.clear();
        }
        w.flush()
    }

    /// Mark the screen for a full clear on next render.
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Handle a terminal resize.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.previous = Buffer::new(width, height);
        self.invalidated = true;
    }

    /// Enter alternate screen buffer.
    pub fn enter_alt_screen(&mut self) {
        escape!(&mut self.output, AlternateScreen::Enable).unwrap();
        self.is_fullscreen = true;
        self.invalidated = true;
    }

    /// Exit alternate screen buffer.
    pub fn exit_alt_screen(&mut self) {
        escape!(&mut self.output, Reset, AlternateScreen::Disable).unwrap();
        self.is_fullscreen = false;
        self.pen.reset();
        self.invalidated = true;
    }

    /// Access the internal output buffer (for testing).
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    /// Clear the output buffer without flushing.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    // ── Inline rendering ───────────────────────────────────────────

    /// Render in inline mode (no alternate screen, relative cursor only).
    fn render_inline(&mut self, buffer: &Buffer, arena: &GraphemeArena) {
        let width = buffer.width;
        let height = buffer.height;

        let inline = self.inline.as_mut().expect("inline state required");

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Enable).unwrap();
        }

        escape!(&mut self.output, TextCursor::Disable).unwrap();

        if inline.first_render {
            // First render: emit each row with \n separators and EL.
            inline.first_render = false;
            inline.owned_rows = height;
            self.previous = Buffer::new(width, height);

            for y in 0..height {
                if y > 0 {
                    self.output.push(b'\n');
                }
                let row = &buffer[Row(y)];
                for col in 0..width {
                    let cell = &row[col];
                    self.pen.update_style(&mut self.output, cell.style());
                    if cell.is_empty() {
                        self.output.push(b' ');
                    } else {
                        let s = cell.as_str(arena);
                        self.output.extend_from_slice(s.as_bytes());
                    }
                }
                self.pen.reset_style(&mut self.output);
                escape!(&mut self.output, EraseLineToEnd).unwrap();
            }

            self.pen.row = height - 1;
            self.pen.col = width;
        } else {
            // Subsequent renders: move up to the top of our owned region, diff rows.
            let old_owned = inline.owned_rows;

            if height > old_owned {
                // Growing: emit newlines to claim more rows.
                let extra = height - old_owned;
                for _ in 0..extra {
                    self.output.push(b'\n');
                }
                self.pen.row += extra;
                inline.owned_rows = height;
            }

            // Move to the top of our owned region.
            if self.pen.row > 0 {
                escape!(&mut self.output, CursorUp(self.pen.row)).unwrap();
            }
            escape!(&mut self.output, CarriageReturn).unwrap();
            self.pen.row = 0;
            self.pen.col = 0;

            // Resize prev if needed.
            if self.previous.width != width || self.previous.height != height {
                self.previous = Buffer::new(width, height);
            }

            // Diff each row using relative movement.
            for y in 0..height {
                Self::diff_line(
                    &self.previous,
                    buffer,
                    &mut self.output,
                    &mut self.pen,
                    CursorMode::Relative,
                    y,
                    width,
                    arena,
                );
            }

            // If we shrank, clear extra rows.
            if height < old_owned {
                for _ in height..old_owned {
                    self.pen.to_relative(self.pen.row + 1, 0, &mut self.output);
                    escape!(&mut self.output, EraseLineToEnd).unwrap();
                }
                // Move back to last row of content.
                if self.pen.row > height - 1 {
                    let up = self.pen.row - (height - 1);
                    escape!(&mut self.output, CursorUp(up)).unwrap();
                    self.pen.row = height - 1;
                }
                inline.owned_rows = height;
            }
        }

        self.pen.reset_style(&mut self.output);
        escape!(&mut self.output, TextCursor::Enable).unwrap();

        if self.caps.contains(Capabilities::SYNC_OUTPUT) {
            escape!(&mut self.output, SynchronizedOutput::Disable).unwrap();
        }

        self.previous = buffer.clone();
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Color, Style};
    use crate::buffer::GraphemeArena;

    use super::*;

    // ── Fullscreen: Basic Rendering ─────────────────────────────────

    #[test]
    fn render_styled_cells_emits_sgr() {
        let style = Style::new().bold().foreground(Color::Rgb(255, 0, 0));

        let buffer = Buffer::from_chars(5, 1, &[(0, 0, 'H', style), (0, 1, 'i', style)]);

        let mut r = Rasterizer::new(5, 1);
        r.render(&buffer, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(output.contains("\x1B["), "expected SGR sequence: {output}");
        assert!(output.contains('H'), "expected 'H': {output}");
        assert!(output.contains('i'), "expected 'i': {output}");
    }

    #[test]
    fn render_identical_buffer_produces_no_diff() {
        let style = Style::new().foreground(Color::Index(2));
        let buffer = Buffer::from_chars(
            3, 1,
            &[(0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style)],
        );

        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer, &GraphemeArena::new());
        r.clear_output();

        // Render same buffer again.
        r.render(&buffer, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(!output_str.contains('A'), "should not re-emit 'A': {output_str}");
        assert!(!output_str.contains('B'), "should not re-emit 'B': {output_str}");
        assert!(!output_str.contains('C'), "should not re-emit 'C': {output_str}");
    }

    #[test]
    fn render_single_cell_change_emits_only_that_cell() {
        let style = Style::new().foreground(Color::Index(3));
        let buf1 = Buffer::from_chars(
            3, 1,
            &[(0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style)],
        );

        let mut r = Rasterizer::new(3, 1);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(
            3, 1,
            &[(0, 0, 'A', style), (0, 1, 'X', style), (0, 2, 'C', style)],
        );

        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains('X'), "should emit 'X': {output_str}");
        assert!(!output_str.contains('A'), "should not re-emit 'A': {output_str}");
        assert!(!output_str.contains('C'), "should not re-emit 'C': {output_str}");
    }

    #[test]
    fn invalidate_forces_full_redraw() {
        let buffer = Buffer::from_chars(2, 1, &[(0, 0, 'Z', Style::EMPTY)]);

        let mut r = Rasterizer::new(2, 1);
        r.render(&buffer, &GraphemeArena::new());
        r.clear_output();

        r.invalidate();
        r.render(&buffer, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains("\x1B[2J"), "should contain ED2: {output_str}");
        assert!(output_str.contains('Z'), "should re-emit 'Z': {output_str}");
    }

    #[test]
    fn resize_forces_full_redraw() {
        let style = Style::EMPTY;
        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', style)]);

        let mut r = Rasterizer::new(3, 1);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        r.resize(5, 2);
        let buf2 = Buffer::from_chars(5, 2, &[(0, 0, 'B', style)]);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains("\x1B[2J"), "should contain ED2 after resize: {output_str}");
        assert!(output_str.contains('B'), "should emit 'B': {output_str}");
    }

    // ── Trailing EL ─────────────────────────────────────────────────

    #[test]
    fn trailing_el_optimization() {
        let style = Style::new().foreground(Color::Index(1));

        let buf1 = Buffer::from_chars(
            5, 1,
            &[
                (0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style),
                (0, 3, 'D', style), (0, 4, 'E', style),
            ],
        );

        let mut r = Rasterizer::new(5, 1);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        // Second frame: cells 2-4 become empty.
        let buf2 = Buffer::from_chars(5, 1, &[(0, 0, 'A', style), (0, 1, 'X', style)]);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains("\x1B[K"), "should contain EL: {output_str}");
    }

    #[test]
    fn trailing_el_entire_row_cleared() {
        let style = Style::new().foreground(Color::Index(1));
        let buf1 = Buffer::from_chars(
            3, 1,
            &[(0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style)],
        );

        let mut r = Rasterizer::new(3, 1);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        // All cells become empty.
        let buf2 = Buffer::new(3, 1);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains("\x1B[K"), "should contain EL when row cleared: {output_str}");
        // Should NOT re-emit any content characters.
        assert!(!output_str.contains('A'), "should not emit 'A': {output_str}");
    }

    #[test]
    fn no_trailing_el_when_content_extends_to_end() {
        let s1 = Style::new().foreground(Color::Index(1));
        let s2 = Style::new().foreground(Color::Index(2));
        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s1), (0, 1, 'B', s1), (0, 2, 'C', s1)]);

        let mut r = Rasterizer::new(3, 1);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        // Change all cells (none become empty).
        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'X', s2), (0, 1, 'Y', s2), (0, 2, 'Z', s2)]);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(!output_str.contains("\x1B[K"), "should not contain EL: {output_str}");
    }

    // ── Synchronized Output ──────────────────────────────────────────

    #[test]
    fn sync_output_wraps_render() {
        let caps = Capabilities::DEFAULT | Capabilities::SYNC_OUTPUT;
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::with_capabilities(3, 1, caps);
        r.render(&buffer, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(output.starts_with("\x1B[?2026h"), "should start with begin_sync: {output}");
        assert!(output.ends_with("\x1B[?2026l"), "should end with end_sync: {output}");
    }

    #[test]
    fn no_sync_without_cap() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(!output.contains("\x1B[?2026h"), "should not contain begin_sync: {output}");
        assert!(!output.contains("\x1B[?2026l"), "should not contain end_sync: {output}");
    }

    // ── Pen Elision ────────────────────────────────────────────────────

    #[test]
    fn pen_elision_no_redundant_sgr() {
        let style = Style::new().foreground(Color::Rgb(0, 255, 0));

        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', style), (0, 1, 'B', style)]);

        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        let sgr_count = output_str.matches("\x1B[38;2;").count();
        assert_eq!(sgr_count, 1, "should emit SGR only once: {output_str}");
    }

    #[test]
    fn style_change_across_frames_emits_new_sgr() {
        let s1 = Style::new().foreground(Color::Rgb(255, 0, 0));
        let s2 = Style::new().foreground(Color::Rgb(0, 0, 255));

        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s1)]);
        let mut r = Rasterizer::new(3, 1);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s2)]);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        // Should contain blue foreground SGR.
        assert!(output_str.contains("38;2;0;0;255"), "should emit new style: {output_str}");
    }

    // ── Cursor Visibility ──────────────────────────────────────────────

    #[test]
    fn render_hides_then_shows_cursor() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer, &GraphemeArena::new());

        let output = r.output();
        let output_str = String::from_utf8_lossy(output);

        let hide = "\x1B[?25l";
        let show = "\x1B[?25h";
        let hide_pos = output_str.find(hide);
        let show_pos = output_str.rfind(show);

        assert!(hide_pos.is_some(), "should contain hide cursor: {output_str}");
        assert!(show_pos.is_some(), "should contain show cursor: {output_str}");
        assert!(hide_pos.unwrap() < show_pos.unwrap(),
            "hide should come before show: {output_str}");
    }

    // ── Alt Screen ─────────────────────────────────────────────────────

    #[test]
    fn enter_exit_alt_screen_sequences() {
        let mut r = Rasterizer::new(3, 1);

        r.enter_alt_screen();
        let output = String::from_utf8_lossy(r.output());
        assert!(output.contains("\x1B[?1047h"), "should enter alt screen: {output}");
        assert!(r.is_fullscreen);
        assert!(r.invalidated);
        r.clear_output();

        r.exit_alt_screen();
        let output = String::from_utf8_lossy(r.output());
        assert!(output.contains("\x1B[?1047l"), "should exit alt screen: {output}");
        assert!(!r.is_fullscreen);
    }

    // ── Flush ──────────────────────────────────────────────────────────

    #[test]
    fn flush_writes_and_clears() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::new(3, 1);
        r.render(&buffer, &GraphemeArena::new());

        assert!(!r.output().is_empty(), "output should be non-empty before flush");

        let mut sink = Vec::new();
        r.flush(&mut sink).unwrap();

        assert!(!sink.is_empty(), "sink should receive output");
        assert!(r.output().is_empty(), "output should be empty after flush");
    }

    // ── Inline Mode ────────────────────────────────────────────────────

    #[test]
    fn inline_first_render_no_cup() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(5, 2, &[
            (0, 0, 'h', style), (0, 1, 'i', style),
            (1, 0, 'l', style), (1, 1, 'o', style),
        ]);

        let mut r = Rasterizer::inline(5, 2);
        r.render(&buffer, &GraphemeArena::new());

        let output = r.output();
        let has_cup = output.windows(2).enumerate().any(|(i, w)| {
            w == b"\x1B[" && {
                let rest = &output[i + 2..];
                rest.iter().position(|&b| b == b'H').map_or(false, |h_pos| {
                    rest[..h_pos].contains(&b';') && rest[..h_pos].iter().all(|b| b.is_ascii_digit() || *b == b';')
                })
            }
        });
        let output_str = String::from_utf8_lossy(output);
        assert!(!has_cup, "should not contain CUP: {output_str}");
        assert!(output_str.contains('h'), "should contain 'h': {output_str}");
        assert!(output_str.contains('l'), "should contain 'l': {output_str}");
    }

    #[test]
    fn inline_second_render_starts_with_cuu() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(5, 3, &[
            (0, 0, 'a', style),
            (1, 0, 'b', style),
            (2, 0, 'c', style),
        ]);

        let mut r = Rasterizer::inline(5, 3);
        r.render(&buffer, &GraphemeArena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(5, 3, &[
            (0, 0, 'a', style),
            (1, 0, 'X', style),
            (2, 0, 'c', style),
        ]);

        r.render(&buf2, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(output.contains("\x1B[") && output.contains('A'),
            "should contain CUU: {output}");
    }

    #[test]
    fn inline_no_alt_screen_sequences() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'z', style)]);

        let mut r = Rasterizer::inline(3, 1);
        r.render(&buffer, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(!output.contains("\x1B[?1049h"), "should not enter alt screen: {output}");
        assert!(!output.contains("\x1B[?1049l"), "should not exit alt screen: {output}");
    }

    #[test]
    fn inline_no_ed_on_first_render() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(3, 2, &[
            (0, 0, 'x', style),
            (1, 0, 'y', style),
        ]);

        let mut r = Rasterizer::inline(3, 2);
        r.render(&buffer, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(!output.contains("\x1B[2J"), "should not contain ED2: {output}");
        assert!(!output.contains("\x1B[H"), "should not contain home: {output}");
    }

    #[test]
    fn inline_diff_only_changed_cells() {
        let style = Style::EMPTY;
        let buf1 = Buffer::from_chars(5, 2, &[
            (0, 0, 'a', style), (0, 1, 'b', style),
            (1, 0, 'c', style), (1, 1, 'd', style),
        ]);

        let mut r = Rasterizer::inline(5, 2);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        // Only change cell (1,0).
        let buf2 = Buffer::from_chars(5, 2, &[
            (0, 0, 'a', style), (0, 1, 'b', style),
            (1, 0, 'X', style), (1, 1, 'd', style),
        ]);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(output_str.contains('X'), "should emit changed cell: {output_str}");
        assert!(!output_str.contains('a'), "should not re-emit 'a': {output_str}");
        assert!(!output_str.contains('b'), "should not re-emit 'b': {output_str}");
        assert!(!output_str.contains('d'), "should not re-emit 'd': {output_str}");
    }

    #[test]
    fn inline_identical_second_render_no_content() {
        let style = Style::EMPTY;
        let buffer = Buffer::from_chars(5, 2, &[
            (0, 0, 'a', style), (0, 1, 'b', style),
            (1, 0, 'c', style), (1, 1, 'd', style),
        ]);

        let mut r = Rasterizer::inline(5, 2);
        r.render(&buffer, &GraphemeArena::new());
        r.clear_output();

        // Same buffer again.
        r.render(&buffer, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        assert!(!output_str.contains('a'), "should not re-emit 'a': {output_str}");
        assert!(!output_str.contains('c'), "should not re-emit 'c': {output_str}");
    }

    #[test]
    fn inline_grow_claims_new_rows() {
        let style = Style::EMPTY;
        let buf1 = Buffer::from_chars(3, 2, &[
            (0, 0, 'a', style),
            (1, 0, 'b', style),
        ]);

        let mut r = Rasterizer::inline(3, 2);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        // Grow to 3 rows.
        let buf2 = Buffer::from_chars(3, 3, &[
            (0, 0, 'a', style),
            (1, 0, 'b', style),
            (2, 0, 'c', style),
        ]);
        r.render(&buf2, &GraphemeArena::new());

        let output = r.output();
        // Should contain a newline for the new row.
        assert!(output.contains(&b'\n'), "should emit newline for growth");
        let output_str = String::from_utf8_lossy(output);
        assert!(output_str.contains('c'), "should emit new row content: {output_str}");
    }

    #[test]
    fn inline_shrink_clears_orphan_rows() {
        let style = Style::EMPTY;
        let buf1 = Buffer::from_chars(3, 3, &[
            (0, 0, 'a', style),
            (1, 0, 'b', style),
            (2, 0, 'c', style),
        ]);

        let mut r = Rasterizer::inline(3, 3);
        r.render(&buf1, &GraphemeArena::new());
        r.clear_output();

        // Shrink to 1 row.
        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'a', style)]);
        r.render(&buf2, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        // Should contain EL sequences for the orphaned rows.
        assert!(output_str.contains("\x1B[K"), "should clear orphan rows: {output_str}");
    }

    #[test]
    fn inline_hides_then_shows_cursor() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::inline(3, 1);
        r.render(&buffer, &GraphemeArena::new());

        let output_str = String::from_utf8_lossy(r.output());
        let hide = "\x1B[?25l";
        let show = "\x1B[?25h";
        assert!(output_str.contains(hide), "should hide cursor: {output_str}");
        assert!(output_str.contains(show), "should show cursor: {output_str}");
        let hide_pos = output_str.find(hide).unwrap();
        let show_pos = output_str.rfind(show).unwrap();
        assert!(hide_pos < show_pos, "hide should come before show: {output_str}");
    }

    #[test]
    fn inline_sync_output() {
        let caps = Capabilities::DEFAULT | Capabilities::SYNC_OUTPUT;
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::EMPTY)]);
        let mut r = Rasterizer::inline_with_capabilities(3, 1, caps);
        r.render(&buffer, &GraphemeArena::new());

        let output = String::from_utf8_lossy(r.output());
        assert!(output.starts_with("\x1B[?2026h"), "should start with begin_sync: {output}");
        assert!(output.ends_with("\x1B[?2026l"), "should end with end_sync: {output}");
    }
}
