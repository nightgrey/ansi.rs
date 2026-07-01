//! Procedural buffer generation for demos, tests, and benchmarking.
//!
//! The [`Routine`] enum describes a handful of deterministic patterns — solid
//! colour fills, chessboard patterns (with Unicode piece glyphs when the
//! board is exactly 8×8), diagonal lines, random-noise grids, and
//! user-supplied character matrices — that are rendered into a [`Buffer`]
//! by [`Buffer::from_procedural`].
//!
//! Each variant produces a pure, deterministic result: even [`Routine::Random`]
//! uses a seed so repeated calls produce identical output.

use crate::{Buffer, Cell};
use ansi::{Attribute, Color, Style};

/// A procedural buffer pattern used for demos, testing, and benchmarking.
///
/// Each variant renders deterministically into a [`Buffer`] via
/// [`Buffer::from_procedural`].
pub enum Routine {
    /// Every cell filled with a solid background colour.
    Solid(Color),

    /// Alternating light and dark squares.
    ///
    /// When the buffer is exactly 8×8, Unicode chess piece glyphs (♔ ♕ ♖ …)
    /// are placed in the standard starting position. For any other size, only
    /// the chessboard pattern is drawn.
    Chessboard,
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
    /// Render a [`Routine`] pattern into a buffer of the given dimensions.
    ///
    /// Each variant produces a deterministic result: even
    /// [`Routine::Random`] uses a fixed seed so repeated calls return
    /// identical output.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let buf = Buffer::from_procedural(Routine::Solid(Color::Blue), 10, 5);
    /// ```
    pub fn from_procedural(routine: Routine, width: u16, height: u16) -> Self {
        match routine {
            Routine::Solid(color) => {
                Self::from_fn(width, height, |_, _| Cell::empty().with_background(color))
            }
            Routine::Chessboard => {
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
            Routine::Diagonals {
                foreground,
                background,
            } => Buffer::from_fn(width, height, |row, col| {
                if row == col || row == height - col - 1 {
                    Cell::default().with_background(foreground.unwrap_or_default())
                } else {
                    Cell::default().with_background(background.unwrap_or_default())
                }
            }),
            Routine::Random(seed) => {
                /// xorshift64 step. Pure function of the state, so callers stay deterministic.
                #[inline]
                fn xorshift_next(state: &mut u64) -> u64 {
                    *state ^= *state << 13;
                    *state ^= *state >> 7;
                    *state ^= *state << 17;
                    *state
                }

                /// Weighted index pick – no external `rand` dependency.
                #[inline]
                fn weighted_pick(state: &mut u64, weights: &[u32], total: u32) -> usize {
                    let mut roll = (xorshift_next(state) % total as u64) as u32;
                    for (i, &w) in weights.iter().enumerate() {
                        if roll < w {
                            return i;
                        }
                        roll -= w;
                    }
                    weights.len() - 1
                }

                /// Build a random foreground colour from the current state.
                #[inline]
                fn random_fg_color(state: &mut u64) -> Color {
                    match weighted_pick(state, &[40, 30, 20, 10], 100) {
                        0 => {
                            // Random RGB
                            let r = (xorshift_next(state) & 0xFF) as u8;
                            let g = (xorshift_next(state) & 0xFF) as u8;
                            let b = (xorshift_next(state) & 0xFF) as u8;
                            Color::Rgb(r, g, b)
                        }
                        1 => {
                            // Random 256-colour Ascii index
                            Color::Index((xorshift_next(state) % 256) as u8)
                        }
                        2 => {
                            // A few named colours for contrast
                            const NAMED: &[Color] = &[
                                Color::Red,
                                Color::Green,
                                Color::Yellow,
                                Color::Blue,
                                Color::Magenta,
                                Color::Cyan,
                                Color::BrightRed,
                                Color::BrightGreen,
                                Color::BrightYellow,
                                Color::BrightBlue,
                                Color::BrightMagenta,
                                Color::BrightCyan,
                            ];
                            let idx = (xorshift_next(state) % NAMED.len() as u64) as usize;
                            NAMED[idx]
                        }
                        _ => Color::None,
                    }
                }

                /// Build a random background colour similarly.
                #[inline]
                fn random_bg_color(state: &mut u64) -> Color {
                    // Same distribution but slightly different weights to
                    // favour darker backgrounds.
                    match weighted_pick(state, &[35, 35, 20, 10], 100) {
                        0 => {
                            let r = (xorshift_next(state) & 0xFF) as u8;
                            let g = (xorshift_next(state) & 0xFF) as u8;
                            let b = (xorshift_next(state) & 0xFF) as u8;
                            Color::Rgb(r, g, b)
                        }
                        1 => Color::Index((xorshift_next(state) % 256) as u8),
                        2 => {
                            const NAMED: &[Color] = &[
                                Color::Blue,
                                Color::Green,
                                Color::Cyan,
                                Color::Red,
                                Color::Magenta,
                                Color::Yellow,
                                Color::Black,
                                Color::Index(232),
                            ];
                            let idx = (xorshift_next(state) % NAMED.len() as u64) as usize;
                            NAMED[idx]
                        }
                        _ => Color::None,
                    }
                }

                /// Randomly attach text decorations to a style.
                #[inline]
                fn random_style(state: &mut u64, style: Style) -> Style {
                    let mut style = style;
                    // Order doesn't matter; we just try each flag independently.
                    if xorshift_next(state) % 10 < 2 {        // 20 % chance
                        style = style.with(Attribute::Bold);
                    }
                    if xorshift_next(state) % 10 < 2 {
                        style = style.with(Attribute::Faint);
                    }
                    if xorshift_next(state) % 20 < 2 {        // 10 % chance
                        style = style.with(Attribute::Italic);
                    }
                    if xorshift_next(state) % 50 < 1 {        // 2 % chance
                        style = style.with(Attribute::Underline);
                    }
                    if xorshift_next(state) % 100 < 1 {       // 1 % chance
                        style = style.with(Attribute::Strikethrough);
                    }
                    style
                }

                // Extended glyph table – ASCII + a few emoji (single codepoints).
                const GLYPHS: &[char] = &[
                    ' ', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
                    'A', 'B', 'C', 'D', 'E', 'F', 'G',
                    '0', '1', '2', '3',
                    '.', '#', '@',
                    '🚀', '🦀', '🎉', '🔥', '✨', '🌟', '🧪', '🔧', '💻', '🎨',
                ];
                const GLYPH_WEIGHTS: &[u32] = &[
                    500,                     // ' '
                    40, 30, 30, 28, 45, 25, 22, // a b c d e f g
                    18, 14, 14, 12, 16, 10, 10, // A B C D E F G
                    10, 10, 8, 8,            // 0 1 2 3
                    12, 6, 4,                // . # @
                    5, 5, 5, 5, 5, 5, 3, 3, 3, 3,
                ];
                debug_assert_eq!(GLYPHS.len(), GLYPH_WEIGHTS.len());
                let glyph_total: u32 = GLYPH_WEIGHTS.iter().sum();

                // Fraction of cells that are glyph / tinted empty / plain empty
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
                            // Glyph cell
                            let gi = weighted_pick(&mut state, GLYPH_WEIGHTS, glyph_total);
                            let fg = random_fg_color(&mut state);
                            let sty = random_style(&mut state, Style::None.foreground(fg));
                            Cell::new(GLYPHS[gi]).with_style(sty)
                        }
                        1 => {
                            // Tinted empty cell
                            let bg = random_bg_color(&mut state);
                            Cell::default().with_background(bg)
                        }
                        _ => Cell::default(),
                    }
                })
            }
        }
    }
}
