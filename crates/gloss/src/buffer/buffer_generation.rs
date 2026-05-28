use ansi::Color;
use crate::{Arena, Buffer, Cell};
pub fn buffer_solid(width: usize, height: usize, color: Color) -> Buffer {
    Buffer::from_fn(width, height, |_, _| Cell::default().with_background(color))
}
/// Creates a buffer with a checkerboard pattern using two cells.
pub fn buffer_chessboard(width: usize, height: usize) -> Buffer {
    Buffer::from_fn(width, height, |row, col| {
        if (row + col) % 2 == 0 { Cell::default().with_background(Color::BrightWhite) } else { Cell::default().with_background(Color::Black) }
    })
}

/// Creates a buffer with a diagonal line of a given cell, rest with another.
/// Useful for testing insertion/deletion that shifts rows/columns.
pub fn buffer_diagonals(width: usize, height: usize) -> Buffer {
    Buffer::from_fn(width, height, |row, col| {
        if row == col || row == height - col - 1 {
            Cell::default().with_background(Color::BrightWhite)
        } else {
            Cell::default()
        }
    })
}

