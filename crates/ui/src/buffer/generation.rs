//! Procedural buffer generation for demos, tests, and benchmarking.
//!
//! The [`Gen`] enum describes a handful of deterministic patterns — solid
//! colour fills, chessboard patterns (with Unicode piece glyphs when the
//! board is exactly 8×8), diagonal lines, random-noise grids, and
//! user-supplied character matrices — that are rendered into a [`Buffer`]
//! by [`Buffer::from_gen`].
//!
//! Each variant produces a pure, deterministic result: even [`Gen::Random`]
//! uses a seed so repeated calls produce identical output.

use crate::{Buffer, Cell};
use ansi::{Attribute, Color, Style};

/// A procedural buffer pattern used for demos, testing, and benchmarking.
///
/// Each variant renders deterministically into a [`Buffer`] via
/// [`Buffer::from_gen`].
pub enum Gen {
    /// Every cell filled with a solid background colour.
    Solid(Color),

    /// Alternating light and dark squares.
    ///
    /// When the buffer is exactly 8×8, Unicode chess piece glyphs (♔ ♕ ♖ …)
    /// are placed in the standard starting position. For any other size, only
    /// the chessboard pattern is drawn.
    Chessboard,

    /// A user-supplied grid of `(character, style)` pairs.
    ///
    /// Each inner `Vec` is one row; characters with value `'\0'` are left
    /// as empty cells.
    Grid(Vec<Vec<(char, Style)>>),

    /// Two crossing diagonal lines, optionally with distinct foreground and
    /// background colours.
    Diagonals {
        /// Colour of the diagonal cells (defaults to the terminal default if `None`).
        foreground: Option<Color>,
        /// Colour of the off-diagonal cells.
        background: Option<Color>,
    },

    /// A deterministic pseudo-random noise grid seeded by the given `u64`.
    ///
    /// The same seed always produces the same pattern. Glyphs, styles, and
    /// background colours are sampled from a fixed palette.
    Random(u64),
}

impl Buffer {
    /// Render a [`Gen`] pattern into a buffer of the given dimensions.
    ///
    /// Each variant produces a deterministic result: even
    /// [`Gen::Random`] uses a fixed seed so repeated calls return
    /// identical output.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let buf = Buffer::from_gen(Gen::Solid(Color::Blue), 10, 5);
    /// // every cell has a blue background
    /// ```
    pub fn from_gen(kind: Gen, width: u16, height: u16) -> Self {
        match kind {
            Gen::Solid(color) => {
                Self::from_fn(width, height, |_, _| Cell::empty().with_background(color))
            }
            Gen::Chessboard => {
                // Predefined board for 8×8 – top row (index 0) is black pieces, bottom row (7) white
                const PIECES: [[Option<char>; 8]; 8] = [
                    // Row 0 (black back rank)
                    [
                        Some('♜'),
                        Some('♞'),
                        Some('♝'),
                        Some('♛'),
                        Some('♚'),
                        Some('♝'),
                        Some('♞'),
                        Some('♜'),
                    ],
                    // Row 1 (black pawns)
                    [
                        Some('♟'),
                        Some('♟'),
                        Some('♟'),
                        Some('♟'),
                        Some('♟'),
                        Some('♟'),
                        Some('♟'),
                        Some('♟'),
                    ],
                    // Rows 2‑5 (empty)
                    [None; 8],
                    [None; 8],
                    [None; 8],
                    [None; 8],
                    // Row 6 (white pawns)
                    [
                        Some('♙'),
                        Some('♙'),
                        Some('♙'),
                        Some('♙'),
                        Some('♙'),
                        Some('♙'),
                        Some('♙'),
                        Some('♙'),
                    ],
                    // Row 7 (white back rank)
                    [
                        Some('♖'),
                        Some('♘'),
                        Some('♗'),
                        Some('♕'),
                        Some('♔'),
                        Some('♗'),
                        Some('♘'),
                        Some('♖'),
                    ],
                ];

                const WHITE: Cell = Cell::default().with_background(Color::Rgb(245, 245, 220));
                const BLACK: Cell = Cell::default().with_background(Color::Rgb(50, 50, 50));
                Buffer::from_fn(width, height, |row, col| {
                    let is_light = (row + col) % 2 == 0;
                    if width == 8 && height == 8 && row < 8 && col < 8 {
                        // Place piece if present
                        if let Some(ch) = PIECES[row as usize][col as usize] {
                            let piece_color = if row < 2 {
                                // Black pieces – use foreground black (or dim white on light bg)
                                Color::Rgb(30, 30, 30)
                            } else {
                                // White pieces
                                Color::Rgb(240, 240, 240)
                            };
                            let mut cell =
                                Cell::new(ch).with_style(Style::None.foreground(piece_color));
                            // Apply square background
                            if is_light {
                                cell = cell.with_background(Color::Rgb(245, 245, 220));
                            } else {
                                cell = cell.with_background(Color::Rgb(50, 50, 50));
                            }
                            cell
                        } else {
                            // Empty square – just background
                            if is_light { WHITE } else { BLACK }
                        }
                    } else {
                        // Fallback for non‑8×8 boards: plain chessboard pattern
                        if is_light { WHITE } else { BLACK }
                    }
                })
            }
            Gen::Diagonals {
                foreground,
                background,
            } => Buffer::from_fn(width, height, |row, col| {
                if row == col || row == height - col - 1 {
                    Cell::default().with_background(foreground.unwrap_or_default())
                } else {
                    Cell::default().with_background(background.unwrap_or_default())
                }
            }),
            Gen::Grid(rows) => {
                let height = rows.len();
                let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
                let mut buf = Buffer::new(width as u16, height as u16);
                for (y, row) in rows.iter().enumerate() {
                    for (x, &(ch, style)) in row.iter().enumerate() {
                        if ch != '\0' {
                            buf[(y as u16, x as u16)] = Cell::new(ch).with_style(style);
                        }
                    }
                }
                buf
            }
            Gen::Random(seed) => {
                /// xorshift64 step. Pure function of the state, so callers stay deterministic.
                #[inline]
                fn xorshift_next(state: &mut u64) -> u64 {
                    *state ^= *state << 13;
                    *state ^= *state >> 7;
                    *state ^= *state << 17;
                    *state
                }

                /// Weighted index pick, no external `rand` dependency — same approach as
                /// the corpus generator's `WeightedIndex`, just hand-rolled against the
                /// existing xorshift state so `Gen::Random` stays self-contained.
                #[inline]
                fn weighted_pick(state: &mut u64, weights: &[u32], total: u32) -> usize {
                    let mut roll = (xorshift_next(state) % total as u64) as u32;
                    for (i, &w) in weights.iter().enumerate() {
                        if roll < w {
                            return i;
                        }
                        roll -= w;
                    }
                    weights.len() - 1 // unreachable unless `total` undercounts `weights`
                }

                // Zipfian-ish glyph table: space dominates, lowercase beats
                // uppercase beats digits beats symbols — mirrors how a real
                // screenful of text is distributed, instead of sampling all 22
                // glyphs uniformly.
                const GLYPHS: &[u8] = b" abcdefgABCDEFG0123.#@";
                const GLYPH_WEIGHTS: &[u32] = &[
                    500, // ' '
                    40, 30, 30, 28, 45, 25, 22, // a b c d e f g
                    18, 14, 14, 12, 16, 10, 10, // A B C D E F G
                    10, 10, 8, 8, // 0 1 2 3
                    12, 6, 4, // . # @
                ];
                debug_assert_eq!(GLYPHS.len(), GLYPH_WEIGHTS.len());
                let glyph_total: u32 = GLYPH_WEIGHTS.iter().sum();

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
                // mostly unstyled, decorations are the rare case — same intuition as
                // the glyph table
                const STYLE_WEIGHTS: &[u32] = &[200, 25, 25, 20, 15, 10];
                let style_total: u32 = STYLE_WEIGHTS.iter().sum();

                let backgrounds = [Color::Rgb(20, 20, 20), Color::Blue, Color::Index(238)];
                const BG_WEIGHTS: &[u32] = &[50, 30, 20];
                let bg_total: u32 = BG_WEIGHTS.iter().sum();

                // glyph cell / tinted-empty cell / plain empty cell
                const OUTCOME_WEIGHTS: &[u32] = &[70, 20, 10];
                let outcome_total: u32 = OUTCOME_WEIGHTS.iter().sum();

                Buffer::from_fn(width, height, |row, col| {
                    let mut state = seed
                        .wrapping_mul(0x9E3779B97F4A7C15)
                        .wrapping_add(1)
                        .wrapping_mul(row as u64 + 1)
                        .wrapping_add(col as u64);

                    match weighted_pick(&mut state, OUTCOME_WEIGHTS, outcome_total) {
                        0 => {
                            let gi = weighted_pick(&mut state, GLYPH_WEIGHTS, glyph_total);
                            let si = weighted_pick(&mut state, STYLE_WEIGHTS, style_total);
                            Cell::new(GLYPHS[gi] as char).with_style(palette[si])
                        }
                        1 => {
                            let bi = weighted_pick(&mut state, BG_WEIGHTS, bg_total);
                            Cell::default().with_background(backgrounds[bi])
                        }
                        _ => Cell::default(),
                    }
                })
            }
        }
    }
}
