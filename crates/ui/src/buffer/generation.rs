use crate::{Buffer, Cell};
use ansi::{Attribute, Color, Style};

pub enum Gen {
    Solid(Color),
    Chessboard,
    Grid(Vec<Vec<(char, Style)>>),
    Diagonals {
        foreground: Option<Color>,
        background: Option<Color>,
    },
    Random(u64),
}

impl Buffer {
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
                let backgrounds = [Color::Rgb(20, 20, 20), Color::Blue, Color::Index(238)];

                Buffer::from_fn(width, height, |row, col| {
                    // Using a deterministic pseudo-random generator seeded per cell
                    // This ensures from_fn remains pure and deterministic
                    let cell_seed = seed
                        .wrapping_mul(0x9E3779B97F4A7C15)
                        .wrapping_add(1)
                        .wrapping_mul(row as u64 + 1)
                        .wrapping_add(col as u64);

                    let mut state = cell_seed;
                    let mut next = || {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        state
                    };

                    // First random check determines if cell is empty
                    if next() % 100 < 100 {
                        let ch = glyphs[(next() as usize) % glyphs.len()] as char;
                        let style = palette[(next() as usize) % palette.len()];
                        Cell::new(ch).with_style(style)
                    } else if next() % 4 == 0 {
                        let bg = backgrounds[(next() as usize) % backgrounds.len()];
                        Cell::default().with_background(bg)
                    } else {
                        Cell::default() // Empty cell
                    }
                })
            }
        }
    }
}
