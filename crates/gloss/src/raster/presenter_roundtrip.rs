//! Roundtrip correctness oracle for the [`Presenter`].
//!
//! Instead of asserting that the presenter emits particular escape *strings*,
//! these tests assert that the bytes it emits, when fed through `ansi`'s VTE
//! parser into a virtual terminal grid, reconstruct the source buffer. That
//! validates the presenter's actual contract — "bring the terminal from `prev`
//! to `next`" — at the level of *where glyphs and styles land*, which substring
//! checks cannot do.
//!
//! ## Pipeline
//!
//! ```text
//! Buffer ──Presenter──▶ bytes ──ansi::Parser──▶ Term grid ──assert==──▶ Buffer
//! ```
//!
//! The [`Term`] is persisted across frames (it is a real terminal model), so a
//! diff frame is correct iff applying it to the prior screen yields `next`.
//!
//! ## Modelling assumptions
//!
//! - **LF (`\n`) is a newline** (cursor down + column 0). The presenter's inline
//!   scrollback-claim relies on this (ONLCR); modelling LF as pure line-feed
//!   would make inline reconstruction wrong, so we mirror the presenter's own
//!   assumption.
//! - **An empty cell with a style is a styled space, not a blank.** A cell with
//!   a background but no glyph is paintable on every path (`Cell::is_blank` is
//!   the "truly nothing" predicate). We do *not* rely on "erase with background
//!   colour" (BCE): the generators never put a style on a cell that the
//!   presenter would clear with `EL` rather than overwrite.

use super::Presenter;
use crate::{Arena, Buffer, Cell};
use ansi::parser::{Handler, Inter, Params, Parser};
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
struct TCell {
    glyph: Glyph,
    style: Style,
}

impl TCell {
    const BLANK: Self = Self {
        glyph: Glyph::Blank,
        style: Style::None,
    };
}

/// Minimal VTE handler: tracks a cursor, a pen style, and a grid that grows
/// downward on demand (for inline scrollback).
struct Term {
    width: usize,
    rows: usize,
    cells: Vec<TCell>,
    cx: usize,
    cy: usize,
    style: Style,
}

impl Term {
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
            self.cells.extend(std::iter::repeat_n(TCell::BLANK, self.width));
            self.rows += 1;
        }
    }

    fn get(&self, x: usize, y: usize) -> TCell {
        if y >= self.rows || x >= self.width {
            return TCell::BLANK;
        }
        self.cells[y * self.width + x]
    }

    fn clear_all(&mut self) {
        for c in &mut self.cells {
            *c = TCell::BLANK;
        }
    }

    /// Erase from the cursor to the end of the current row (`CSI K`).
    fn erase_line_to_end(&mut self) {
        if self.cy >= self.rows {
            return;
        }
        for x in self.cx..self.width {
            self.cells[self.cy * self.width + x] = TCell::BLANK;
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

impl Handler for Term {
    fn print(&mut self, c: char) {
        let w = c.width().unwrap_or(0);
        if w == 0 {
            return; // zero-width (combining) — our generators don't produce these
        }
        self.ensure_row(self.cy);
        if self.cx < self.width {
            let base = self.cy * self.width + self.cx;
            self.cells[base] = TCell {
                glyph: Glyph::Char(c),
                style: self.style,
            };
            for k in 1..w {
                if self.cx + k < self.width {
                    self.cells[base + k] = TCell {
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

    fn csi(&mut self, params: Params<'_>, _intermediates: &Inter, final_char: char) {
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
                            self.cells[y * self.width + x] = TCell::BLANK;
                        }
                    }
                }
            },
            'K' => match p.first().copied().unwrap_or(0) {
                1 => {
                    if self.cy < self.rows {
                        for x in 0..=self.cx.min(self.width.saturating_sub(1)) {
                            self.cells[self.cy * self.width + x] = TCell::BLANK;
                        }
                    }
                }
                2 => {
                    if self.cy < self.rows {
                        for x in 0..self.width {
                            self.cells[self.cy * self.width + x] = TCell::BLANK;
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

fn canon_term(c: TCell) -> Canon {
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
    term: Term,
    prev: Buffer,
    mark: usize,
}

impl Roundtrip {
    fn fullscreen(width: usize, height: usize) -> Self {
        Self {
            presenter: Presenter::new(Cursor::new(Vec::new())),
            parser: Parser::default(),
            term: Term::new(width),
            prev: Buffer::new(width, height),
            mark: 0,
        }
    }

    fn inline(width: usize, height: usize) -> Self {
        Self {
            presenter: Presenter::inline(Cursor::new(Vec::new())),
            parser: Parser::default(),
            term: Term::new(width),
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

        self.assert_matches(next, arena, &bytes);
    }

    fn assert_matches(&self, next: &Buffer, arena: &Arena, bytes: &[u8]) {
        let w = next.width;
        let h = next.height;
        for y in 0..h {
            for x in 0..w {
                let cell = &next[geometry::Point {
                    x: x as u16,
                    y: y as u16,
                }];
                let expected = canon_cell(cell, arena);
                let actual = canon_term(self.term.get(x, y));
                if expected != actual {
                    panic!(
                        "mismatch at ({x},{y}): expected {} got {}\n\
                         expected grid:\n{}\nreconstructed grid:\n{}\nbytes: {:?}",
                        describe(expected),
                        describe(actual),
                        self.dump_expected(next, arena),
                        self.dump_actual(h),
                        String::from_utf8_lossy(bytes),
                    );
                }
            }
        }
    }

    fn dump_expected(&self, next: &Buffer, arena: &Arena) -> String {
        let mut s = String::new();
        for y in 0..next.height {
            for x in 0..next.width {
                let cell = &next[geometry::Point {
                    x: x as u16,
                    y: y as u16,
                }];
                s.push_str(&describe(canon_cell(cell, arena)));
            }
            s.push('\n');
        }
        s
    }

    fn dump_actual(&self, h: usize) -> String {
        let mut s = String::new();
        for y in 0..h {
            for x in 0..self.term.width {
                s.push_str(&describe(canon_term(self.term.get(x, y))));
            }
            s.push('\n');
        }
        s
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
