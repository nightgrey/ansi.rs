use std::io::Write as _;
use ansi::escape;
use ansi::io::Write;
use ansi::sequences::*;
use std::io;
use geometry::{Bounded, Row};
use terminal::Capabilities;
use super::cursor::Cursor;
use crate::Cell;
use crate::{Buffer, Arena};

#[derive(Debug, Clone)]
pub struct Rasterizer {
    output: Vec<u8>,
    shadow: Buffer,
    pen: Cursor,
    capabilities: Capabilities,
    invalidated: bool,
    inline: Option<InlineState>,
}

impl Rasterizer {
    /// Create a new rasterizer with the given screen dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            output: Vec::with_capacity(width * height * 4),
            shadow: Buffer::new(width, height),
            pen: Cursor::new(),
            capabilities: Capabilities::from_env(),
            invalidated: true,
            inline: None,
        }
    }

    /// Create an inline rasterizer (renders in the normal scrollback region).
    pub fn inline(width: usize, height: usize) -> Self {
        Self {
            inline: Some(InlineState {
                owned_rows: 0,
                first_render: true,
            }),
            ..Self::new(width, height)
        }
    }

    pub fn for_buffer(buffer: &Buffer) -> Self {
        Self {
            output: Vec::with_capacity(buffer.width * buffer.height * 4),
            shadow: Buffer::new(buffer.width, buffer.height),
            pen: Cursor::new(),
            capabilities: Capabilities::default(),
            invalidated: true,
            inline: None,
        }
    }

    /// Set terminal capabilities. Chainable.
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Returns whether this rasterizer operates in inline mode.
    pub fn is_inline(&self) -> bool {
        self.inline.is_some()
    }

    /// Rasterize a buffer, diffing against the shadow frame.
    pub fn raster(&mut self, buffer: &Buffer, arena: &Arena) -> io::Result<()> {
        let (shadow, next) = (&mut self.shadow, buffer);

        if self.inline.is_some() {
            return self.raster_inline_impl(next, arena);
        }

        let width = next.width;
        let height = next.height;

        if self.capabilities.use_sync_output() {
            self.output.escape(SynchronizedOutput::Set)?;
        }

        // Handle dimension change or forced clear.
        if shadow.width != width || shadow.height != height || self.invalidated {
            self.output.escape(Home)?;
            self.output.escape(EraseDisplay)?;
            self.pen.reset();
            shadow.resize(width, height);
            shadow.clear();
            self.invalidated = false;
        }

        self.output.escape(TextCursorEnable::Reset)?;

        let cursor_mode = CursorMode::Absolute(self.capabilities);
        for y in 0..height {
            Self::row(
                &shadow[Row(y)],
                &next[Row(y)],
                arena,
                y,
                &mut self.output,
                &mut self.pen,
                cursor_mode,
                width,
            )?;
        }

        self.output.escape(SelectGraphicRendition::RESET)?;
        self.output.escape(TextCursorEnable::Set)?;
        if self.capabilities.use_sync_output() {
            self.output.escape(SynchronizedOutput::Reset)?;
        }

        // Swap prev ← new.
        self.shadow.copy_from_slice(buffer.as_ref());
        Ok(())
    }
    pub fn write(&mut self, out: &mut impl io::Write) -> io::Result<()> {
        out.write_all(&self.output)
    }
    /// Flush the accumulated output to a writer and clear the buffer.
    pub fn flush(&mut self, out: &mut impl io::Write) -> io::Result<()> {
        if !self.output.is_empty() {
            out.write_all(&self.output)?;
            self.output.clear();
        }
        out.flush()
    }

    pub fn clear(&mut self) {
        self.output.clear();
        self.shadow.clear();
        self.pen.reset();
        self.invalidated = true;
    }

    /// Mark the screen for a full clear on next render.
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Handle a terminal resize.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.shadow.resize(width, height);
        self.invalidated = true;
    }

    /// Enter alternate screen buffer.
    pub fn enter_alt_screen(&mut self) {
        self.output.escape(AlternateScreen::Set).unwrap();
        self.invalidated = true;
    }

    /// Exit alternate screen buffer.
    pub fn exit_alt_screen(&mut self) {
        escape!(self.output, SelectGraphicRendition::RESET, AlternateScreen::Reset, SelectGraphicRendition::RESET);
        self.pen.reset();
        self.invalidated = true;
    }

    /// Returns the last output as bytes.
    ///
    /// For testing and debugging.
    pub fn as_bytes(&self) -> &[u8] {
        &self.output
    }

    /// Returns the last output as a string.
    ///
    /// For testing and debugging.
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.output) }
    }

    /// Clear the output buffer without flushing.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Render an entire buffer to the writer, no diffing.
    pub fn once(buffer: &Buffer, arena: &Arena, out: &mut impl io::Write) -> io::Result<()> {
        let mut pen = Cursor::new();
        let mut output = Vec::with_capacity(buffer.width * buffer.height * 4);

        output.escape(TextCursorEnable::Reset)?;
        output.escape(Home)?;

        for y in 0..buffer.height {
            if y > 0 {
                output.push(b'\n');
                output.escape(CarriageReturn)?;
            }

            let row = &buffer[Row(y)];
            let last_content = (0..buffer.width)
                .rev()
                .find(|&x| !row[x].is_empty());

            if let Some(end) = last_content {
                for col in 0..=end {
                    let cell = &row[col];
                    pen.transition(&mut output, cell.style)?;
                    if cell.is_blank() {
                        output.push(b' ');
                    } else {
                        output.extend_from_slice(cell.as_bytes(arena));
                    }
                }
            }

            output.escape(EraseLineToEnd)?;
        }

        output.escape(SelectGraphicRendition::RESET)?;
        output.escape(TextCursorEnable::Set)?;

        out.write_all(&output)?;
        out.flush()
    }

    /// Render in inline mode (no alternate screen, relative cursor only).
    fn raster_inline_impl(&mut self, buffer: &Buffer, arena: &Arena) -> io::Result<()> {
        let (prev, next) = (&mut self.shadow, buffer);
        let width = next.width;
        let height = next.height;

        let inline = self.inline.as_mut().expect("inline state required");

        if self.capabilities.use_sync_output() {
            self.output.escape(SynchronizedOutput::Set)?;
        }

        self.output.escape(TextCursorEnable::Reset)?;

        if inline.first_render {
            // First render: emit each row with \n separators.
            // Only emit up to the last non-empty cell per row, then EL.
            inline.first_render = false;
            inline.owned_rows = height;
            prev.resize(width, height);

            for y in 0..height {
                if y > 0 {
                    self.output.push(b'\n');
                }

                let row = &buffer[Row(y)];


                // Find last non-empty cell in this row.
                let last_content = (0..width).rev().find(|&x| {
                    let cell = &row[x];
                    !cell.is_empty()
                });

                match last_content {
                    Some(end) => {
                        for col in 0..=end {
                            let cell = &row[col];
                            Self::render_cell(cell, &mut self.output, &mut self.pen, arena)?;
                        }
                    }
                    None => {
                        // Entire row is empty — nothing to emit before EL.
                    }
                }

                self.pen.reset_style(&mut self.output);
                self.output.escape(EraseLineToEnd)?;
            }

            self.pen.row = height - 1;
            self.pen.col = match (0..width)
                .rev()
                .find(|&x| !next[x].is_empty())
            {
                Some(end) => end + 1,
                None => 0,
            };
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
                self.output.escape(CursorUp(self.pen.row))?;
            }
            self.output.escape(CarriageReturn)?;
            self.pen.row = 0;
            self.pen.col = 0;

            // Resize prev if needed.
            if self.shadow.width != width || self.shadow.height != height {
                self.shadow.resize(width, height);
            }

            // Diff each row using relative movement.
            for y in 0..height {
                Self::row(
                    &self.shadow[y * width..(y) * width + width],
                    &next[y * width..(y) * width + width],
                    arena,
                    y,
                    &mut self.output,
                    &mut self.pen,
                    CursorMode::Relative,
                    width,
                );
            }

            // If we shrank, clear extra rows.
            if height < old_owned {
                for _ in height..old_owned {
                    self.pen
                        .move_to_relative(self.pen.row + 1, 0, &mut self.output);
                    self.output.escape(EraseLineToEnd)?;
                }
                // Move back to last row of content.
                if self.pen.row > height - 1 {
                    let up = self.pen.row - (height - 1);
                    self.output.escape(CursorUp(up))?;
                    self.pen.row = height - 1;
                }
                inline.owned_rows = height;
            }
        }

        self.pen.reset_style(&mut self.output);
        self.output.escape(TextCursorEnable::Set)?;

        if self.capabilities.sync_output {
            self.output.escape(SynchronizedOutput::Reset)?;
        }

        // Reuse the allocation when dimensions match.
        if self.shadow.width == width && self.shadow.height == height {
            self.shadow.copy_from_slice(next.as_ref());
        } else {
            self.shadow.clone_from(next);
        }

        Ok(())
    }

    /// Diff a single row, emitting only the changed cells.
    ///
    /// Uses left→right and right→left scanning to find the minimal dirty
    /// region, plus a trailing EL optimization when the tail goes empty.
    fn row(
        prev: &[Cell],
        next: &[Cell],
        arena: &Arena,
        y: usize,
        output: &mut Vec<u8>,
        cursor: &mut Cursor,
        cursor_mode: CursorMode,
        width: usize,
    ) -> io::Result<()> {
        // Scan left→right for first differing cell.
        let first = match (0..width).find(|&x| next[x] != prev[x]) {
            Some(col) => col,
            None => return Ok(()), // Entire line is identical.
        };

        // Scan right→left for last differing cell.
        let last = (0..width).rev().find(|&x| next[x] != prev[x]).unwrap_or(width - 1);

        // Within [first, last], find the last non-empty new cell. If the tail
        // of the diff range is all-empty new cells replacing old content, we
        // can use EL instead of emitting spaces.
        let last_content = (first..=last).rev().find(|&x| !next[x].is_empty());

        // Move cursor to start of changed region.
        match cursor_mode {
            CursorMode::Absolute(caps) => cursor.move_to(y, first, output, caps),
            CursorMode::Relative => cursor.move_to_relative(y, first, output),
        }

        match last_content {
            None => {

                // Entire diff range is now empty — just erase to end of line.
                cursor.reset_style(output);
                escape!(output, EraseLineToEnd)?;
            }
            Some(emit_end) => {
                // Emit changed cells from first through emit_end.
                let mut col = first;
                while col <= emit_end {
                    let cell = &next[col];
                    Self::render_cell(cell, output, cursor, arena)?;
                    let w = cell.width() as usize;
                    col += w;
                    cursor.col += w;
                }

                // Clear to end of line if trailing cells transitioned to empty.
                if emit_end < last {
                    cursor.reset_style(output);
                    escape!(output, EraseLineToEnd)?;
                }
            }
        }
        
        Ok(())
    }

    /// Write a single cell's content, updating the pen first.
    #[inline]
    fn render_cell(cell: &Cell, output: &mut Vec<u8>, cursor: &mut Cursor, arena: &Arena) -> io::Result<()> {
        cursor.transition(output, cell.style)?;

            output.extend_from_slice(cell.as_bytes(arena));
        Ok(())
    }
}

/// How the cursor should be positioned before emitting cells.
#[derive(Clone, Copy)]
enum CursorMode {
    /// Use absolute or optimized movement (fullscreen).
    Absolute(Capabilities),
    /// Use only relative movement (inline).
    Relative,
}

/// State for inline rendering.
#[derive(Debug, Clone, Copy)]
struct InlineState {
    /// Number of rows the rasterizer "owns" in the terminal.
    owned_rows: usize,
    /// Whether this is the first render call.
    first_render: bool,
}

#[cfg(test)]
mod tests {
    use crate::Arena;
    use ansi::{Color, Style};

    use super::*;

    // ── Fullscreen: Basic Rendering ─────────────────────────────────

    #[test]
    fn render_styled_cells_emits_sgr() {
        let style = Style::default().bold().foreground(Color::Rgb(255, 0, 0));

        let mut buffer = Buffer::from_chars(5, 1, &[(0, 0, 'H', style), (0, 1, 'i', style)]);

        let mut r = Rasterizer::new(5, 1);
        r.raster(&buffer, &Arena::new());

        let output = r.as_str();
        assert!(output.contains("\x1B["), "expected SGR sequence: {output}");
        assert!(output.contains('H'), "expected 'H': {output}");
        assert!(output.contains('i'), "expected 'i': {output}");
    }

    #[test]
    fn render_identical_buffer_produces_no_diff() {
        let style = Style::default().foreground(Color::Index(2));
        let buffer = Buffer::from_chars(
            3,
            1,
            &[(0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style)],
        );

        let mut r = Rasterizer::new(3, 1);
        r.raster(&buffer, &Arena::new());
        r.clear_output();

        r.raster(&buffer, &Arena::new());

        let output_str = r.as_str();
        assert!(
            !output_str.contains('A'),
            "should not re-emit 'A': {output_str}"
        );
        assert!(
            !output_str.contains('B'),
            "should not re-emit 'B': {output_str}"
        );
        assert!(
            !output_str.contains('C'),
            "should not re-emit 'C': {output_str}"
        );
    }

    #[test]
    fn render_single_cell_change_emits_only_that_cell() {
        let style = Style::default().foreground(Color::Index(3));
        let buf1 = Buffer::from_chars(
            3,
            1,
            &[(0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style)],
        );

        let mut r = Rasterizer::new(3, 1);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(
            3,
            1,
            &[(0, 0, 'A', style), (0, 1, 'X', style), (0, 2, 'C', style)],
        );

        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(output_str.contains('X'), "should emit 'X': {output_str}");
        assert!(
            !output_str.contains('A'),
            "should not re-emit 'A': {output_str}"
        );
        assert!(
            !output_str.contains('C'),
            "should not re-emit 'C': {output_str}"
        );
    }

    #[test]
    fn invalidate_forces_full_redraw() {
        let buffer = Buffer::from_chars(2, 1, &[(0, 0, 'Z', Style::None)]);

        let mut r = Rasterizer::new(2, 1);
        r.raster(&buffer, &Arena::new());
        r.clear_output();

        r.invalidate();
        r.raster(&buffer, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains("\x1B[2J"),
            "should contain ED2: {output_str}"
        );
        assert!(output_str.contains('Z'), "should re-emit 'Z': {output_str}");
    }

    #[test]
    fn resize_forces_full_redraw() {
        let style = Style::None;
        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', style)]);

        let mut r = Rasterizer::new(3, 1);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        r.resize(5, 2);
        let buf2 = Buffer::from_chars(5, 2, &[(0, 0, 'B', style)]);
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains("\x1B[2J"),
            "should contain ED2 after resize: {output_str}"
        );
        assert!(output_str.contains('B'), "should emit 'B': {output_str}");
    }

    // ── Trailing EL ─────────────────────────────────────────────────

    #[test]
    fn trailing_el_optimization() {
        let style = Style::default().foreground(Color::Index(1));

        let buf1 = Buffer::from_chars(
            5,
            1,
            &[
                (0, 0, 'A', style),
                (0, 1, 'B', style),
                (0, 2, 'C', style),
                (0, 3, 'D', style),
                (0, 4, 'E', style),
            ],
        );

        let mut r = Rasterizer::new(5, 1);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(5, 1, &[(0, 0, 'A', style), (0, 1, 'X', style)]);
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains("\x1B[K"),
            "should contain EL: {output_str}"
        );
    }

    #[test]
    fn trailing_el_entire_row_cleared() {
        let style = Style::default().foreground(Color::Index(1));
        let buf1 = Buffer::from_chars(
            3,
            1,
            &[(0, 0, 'A', style), (0, 1, 'B', style), (0, 2, 'C', style)],
        );

        let mut r = Rasterizer::new(3, 1);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::new(3, 1);
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains("\x1B[K"),
            "should contain EL when row cleared: {output_str}"
        );
        assert!(
            !output_str.contains('A'),
            "should not emit 'A': {output_str}"
        );
    }

    #[test]
    fn no_trailing_el_when_content_extends_to_end() {
        let s1 = Style::default().foreground(Color::Index(1));
        let s2 = Style::default().foreground(Color::Index(2));
        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s1), (0, 1, 'B', s1), (0, 2, 'C', s1)]);

        let mut r = Rasterizer::new(3, 1);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'X', s2), (0, 1, 'Y', s2), (0, 2, 'Z', s2)]);
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            !output_str.contains("\x1B[K"),
            "should not contain EL: {output_str}"
        );
    }

    // ── Synchronized Output ──────────────────────────────────────────

    #[test]
    fn sync_output_wraps_render() {
        let caps = Capabilities::builder().sync_output(true).build();
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut r = Rasterizer::new(3, 1).with_capabilities(caps);
        r.raster(&buffer, &Arena::new());

        let output = r.as_str();
        assert!(
            output.starts_with("\x1B[?2026h"),
            "should start with begin_sync: {output}"
        );
        assert!(
            output.ends_with("\x1B[?2026l"),
            "should end with end_sync: {output}"
        );
    }

    #[test]
    fn no_sync_without_cap() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut r = Rasterizer::new(3, 1);
        r.raster(&buffer, &Arena::new());

        let output = r.as_str();
        assert!(
            !output.contains("\x1B[?2026h"),
            "should not contain begin_sync: {output}"
        );
        assert!(
            !output.contains("\x1B[?2026l"),
            "should not contain end_sync: {output}"
        );
    }

    // ── Pen Elision ────────────────────────────────────────────────────

    #[test]
    fn pen_elision_no_redundant_sgr() {
        let style = Style::default().foreground(Color::Rgb(0, 255, 0));

        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', style), (0, 1, 'B', style)]);

        let mut r = Rasterizer::new(3, 1);
        r.raster(&buffer, &Arena::new());

        let output_str = r.as_str();
        let sgr_count = output_str.matches("\x1B[38;2;").count();
        assert_eq!(sgr_count, 1, "should emit SGR only once: {output_str}");
    }

    #[test]
    fn style_change_across_frames_emits_new_sgr() {
        let s1 = Style::default().foreground(Color::Rgb(255, 0, 0));
        let s2 = Style::default().foreground(Color::Rgb(0, 0, 255));

        let buf1 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s1)]);
        let mut r = Rasterizer::new(3, 1);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'A', s2)]);
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains("38;2;0;0;255"),
            "should emit new style: {output_str}"
        );
    }

    // ── Cursor Visibility ──────────────────────────────────────────────

    #[test]
    fn render_hides_then_shows_cursor() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut r = Rasterizer::new(3, 1);
        r.raster(&buffer, &Arena::new());

        let output_str = r.as_str();

        let hide = "\x1B[?25l";
        let show = "\x1B[?25h";
        let hide_pos = output_str.find(hide);
        let show_pos = output_str.rfind(show);

        assert!(
            hide_pos.is_some(),
            "should contain hide cursor: {output_str}"
        );
        assert!(
            show_pos.is_some(),
            "should contain show cursor: {output_str}"
        );
        assert!(
            hide_pos.unwrap() < show_pos.unwrap(),
            "hide should come before show: {output_str}"
        );
    }

    // ── Alt Screen ─────────────────────────────────────────────────────

    #[test]
    fn enter_exit_alt_screen_sequences() {
        let mut r = Rasterizer::new(3, 1);

        r.enter_alt_screen();
        let output = r.as_str();
        assert!(
            output.contains("\x1B[?1047h"),
            "should enter alt screen: {output}"
        );
        assert!(r.invalidated);
        r.clear_output();

        r.exit_alt_screen();
        let output = r.as_str();
        assert!(
            output.contains("\x1B[?1047l"),
            "should exit alt screen: {output}"
        );
    }

    // ── Flush ──────────────────────────────────────────────────────────

    #[test]
    fn flush_writes_and_clears() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut r = Rasterizer::new(3, 1);
        r.raster(&buffer, &Arena::new());

        assert!(
            !r.as_bytes().is_empty(),
            "output should be non-empty before flush"
        );

        let mut sink = Vec::new();
        r.flush(&mut sink).unwrap();

        assert!(!sink.is_empty(), "sink should receive output");
        assert!(r.as_bytes().is_empty(), "output should be empty after flush");
    }

    // ── Constructor API ────────────────────────────────────────────────

    #[test]
    fn is_inline_reflects_mode() {
        let fs = Rasterizer::new(3, 1);
        assert!(!fs.is_inline());

        let il = Rasterizer::inline(3, 1);
        assert!(il.is_inline());
    }

    #[test]
    fn with_capabilities_chainable() {
        let caps = Capabilities::builder().sync_output(true).build();
        let r = Rasterizer::new(3, 1).with_capabilities(caps);
        assert!(r.capabilities.sync_output);
    }

    #[test]
    fn inline_with_capabilities_chainable() {
        let caps = Capabilities::builder().sync_output(true).build();
        let r = Rasterizer::inline(3, 1).with_capabilities(caps);
        assert!(r.is_inline());
        assert!(r.capabilities.sync_output);
    }

    // ── Inline Mode ────────────────────────────────────────────────────

    #[test]
    fn inline_first_render_no_cup() {
        let style = Style::None;
        let buffer = Buffer::from_chars(
            5,
            2,
            &[
                (0, 0, 'h', style),
                (0, 1, 'i', style),
                (1, 0, 'l', style),
                (1, 1, 'o', style),
            ],
        );

        let mut r = Rasterizer::inline(5, 2);
        r.raster(&buffer, &Arena::new());

        let output = r.as_bytes();
        let has_cup = output.windows(2).enumerate().any(|(i, w)| {
            w == b"\x1B[" && {
                let rest = &output[i + 2..];
                rest.iter().position(|&b| b == b'H').map_or(false, |h_pos| {
                    rest[..h_pos].contains(&b';')
                        && rest[..h_pos]
                        .iter()
                        .all(|b| b.is_ascii_digit() || *b == b';')
                })
            }
        });
        let output_str = String::from_utf8_lossy(output);
        assert!(!has_cup, "should not contain CUP: {output_str}");
        assert!(output_str.contains('h'), "should contain 'h': {output_str}");
        assert!(output_str.contains('l'), "should contain 'l': {output_str}");
    }

    #[test]
    fn inline_first_render_skips_trailing_empty_cells() {
        let style = Style::None;
        // Only first 2 of 10 columns have content.
        let buffer = Buffer::from_chars(10, 1, &[(0, 0, 'a', style), (0, 1, 'b', style)]);

        let mut r = Rasterizer::inline(10, 1);
        r.raster(&buffer, &Arena::new());

        let output = r.as_bytes();
        // Count space characters — should NOT have 8 trailing spaces.
        let space_count = output.iter().filter(|&&b| b == b' ').count();
        assert!(
            space_count < 3,
            "should not emit trailing spaces for empty cells, got {space_count}"
        );
    }

    #[test]
    fn inline_second_render_starts_with_cuu() {
        let style = Style::None;
        let buffer = Buffer::from_chars(
            5,
            3,
            &[(0, 0, 'a', style), (1, 0, 'b', style), (2, 0, 'c', style)],
        );

        let mut r = Rasterizer::inline(5, 3);
        r.raster(&buffer, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(
            5,
            3,
            &[(0, 0, 'a', style), (1, 0, 'X', style), (2, 0, 'c', style)],
        );

        r.raster(&buf2, &Arena::new());

        let output = r.as_str();
        assert!(
            output.contains("\x1B[") && output.contains('A'),
            "should contain CUU: {output}"
        );
    }

    #[test]
    fn inline_no_alt_screen_sequences() {
        let style = Style::None;
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'z', style)]);

        let mut r = Rasterizer::inline(3, 1);
        r.raster(&buffer, &Arena::new());

        let output = r.as_str();
        assert!(
            !output.contains("\x1B[?1049h"),
            "should not enter alt screen: {output}"
        );
        assert!(
            !output.contains("\x1B[?1049l"),
            "should not exit alt screen: {output}"
        );
    }

    #[test]
    fn inline_no_ed_on_first_render() {
        let style = Style::None;
        let buffer = Buffer::from_chars(3, 2, &[(0, 0, 'x', style), (1, 0, 'y', style)]);

        let mut r = Rasterizer::inline(3, 2);
        r.raster(&buffer, &Arena::new());

        let output = r.as_str();
        assert!(
            !output.contains("\x1B[2J"),
            "should not contain ED2: {output}"
        );
        assert!(
            !output.contains("\x1B[H"),
            "should not contain home: {output}"
        );
    }

    #[test]
    fn inline_diff_only_changed_cells() {
        let style = Style::None;
        let buf1 = Buffer::from_chars(
            5,
            2,
            &[
                (0, 0, 'a', style),
                (0, 1, 'b', style),
                (1, 0, 'c', style),
                (1, 1, 'd', style),
            ],
        );

        let mut r = Rasterizer::inline(5, 2);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(
            5,
            2,
            &[
                (0, 0, 'a', style),
                (0, 1, 'b', style),
                (1, 0, 'X', style),
                (1, 1, 'd', style),
            ],
        );
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains('X'),
            "should emit changed cell: {output_str}"
        );
        assert!(
            !output_str.contains('a'),
            "should not re-emit 'a': {output_str}"
        );
        assert!(
            !output_str.contains('b'),
            "should not re-emit 'b': {output_str}"
        );
        assert!(
            !output_str.contains('d'),
            "should not re-emit 'd': {output_str}"
        );
    }

    #[test]
    fn inline_identical_second_render_no_content() {
        let style = Style::None;
        let buffer = Buffer::from_chars(
            5,
            2,
            &[
                (0, 0, 'a', style),
                (0, 1, 'b', style),
                (1, 0, 'c', style),
                (1, 1, 'd', style),
            ],
        );

        let mut r = Rasterizer::inline(5, 2);
        r.raster(&buffer, &Arena::new());
        r.clear_output();

        r.raster(&buffer, &Arena::new());

        let output_str = r.as_str();
        assert!(
            !output_str.contains('a'),
            "should not re-emit 'a': {output_str}"
        );
        assert!(
            !output_str.contains('c'),
            "should not re-emit 'c': {output_str}"
        );
    }

    #[test]
    fn inline_grow_claims_new_rows() {
        let style = Style::None;
        let buf1 = Buffer::from_chars(3, 2, &[(0, 0, 'a', style), (1, 0, 'b', style)]);

        let mut r = Rasterizer::inline(3, 2);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(
            3,
            3,
            &[(0, 0, 'a', style), (1, 0, 'b', style), (2, 0, 'c', style)],
        );
        r.raster(&buf2, &Arena::new());

        let output = r.as_bytes();
        assert!(output.contains(&b'\n'), "should emit newline for growth");
        let output_str = String::from_utf8_lossy(output);
        assert!(
            output_str.contains('c'),
            "should emit new row content: {output_str}"
        );
    }

    #[test]
    fn inline_shrink_clears_orphan_rows() {
        let style = Style::None;
        let buf1 = Buffer::from_chars(
            3,
            3,
            &[(0, 0, 'a', style), (1, 0, 'b', style), (2, 0, 'c', style)],
        );

        let mut r = Rasterizer::inline(3, 3);
        r.raster(&buf1, &Arena::new());
        r.clear_output();

        let buf2 = Buffer::from_chars(3, 1, &[(0, 0, 'a', style)]);
        r.raster(&buf2, &Arena::new());

        let output_str = r.as_str();
        assert!(
            output_str.contains("\x1B[K"),
            "should clear orphan rows: {output_str}"
        );
    }

    #[test]
    fn inline_hides_then_shows_cursor() {
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut r = Rasterizer::inline(3, 1);
        r.raster(&buffer, &Arena::new());

        let output_str = r.as_str();
        let hide = "\x1B[?25l";
        let show = "\x1B[?25h";
        assert!(
            output_str.contains(hide),
            "should hide cursor: {output_str}"
        );
        assert!(
            output_str.contains(show),
            "should show cursor: {output_str}"
        );
        let hide_pos = output_str.find(hide).unwrap();
        let show_pos = output_str.rfind(show).unwrap();
        assert!(
            hide_pos < show_pos,
            "hide should come before show: {output_str}"
        );
    }

    #[test]
    fn inline_sync_output() {
        let caps = Capabilities::builder().sync_output(true).build();
        let buffer = Buffer::from_chars(3, 1, &[(0, 0, 'A', Style::None)]);
        let mut r = Rasterizer::inline(3, 1).with_capabilities(caps);
        r.raster(&buffer, &Arena::new());

        let output = r.as_str();
        assert!(
            output.starts_with("\x1B[?2026h"),
            "should start with begin_sync: {output}"
        );
        assert!(
            output.ends_with("\x1B[?2026l"),
            "should end with end_sync: {output}"
        );
    }
}
