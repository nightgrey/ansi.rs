use crate::*;
use ansi::Color;
use tree::At;

pub fn chess_board(engine: &mut Engine) {
    /// Chess pieces in standard starting position (white perspective, rank 8 → rank 1).
    const CHESS_BOARD: [[&str; 8]; 8] = [
        ["♜", "♞", "♝", "♛", "♚", "♝", "♞", "♜"], // rank 8 (black pieces)
        ["♟", "♟", "♟", "♟", "♟", "♟", "♟", "♟"], // rank 7 (black pawns)
        [" ", " ", " ", " ", " ", " ", " ", " "], // rank 6
        [" ", " ", " ", " ", " ", " ", " ", " "], // rank 5
        [" ", " ", " ", " ", " ", " ", " ", " "], // rank 4
        [" ", " ", " ", " ", " ", " ", " ", " "], // rank 3
        ["♙", "♙", "♙", "♙", "♙", "♙", "♙", "♙"], // rank 2 (white pawns)
        ["♖", "♘", "♗", "♕", "♔", "♗", "♘", "♖"], // rank 1 (white pieces)
    ];

    let root = engine.root_mut();
    root.display = Display::Flex;
    root.flex_direction = FlexDirection::Column;
    root.align_items = AlignItems::Center.into();
    root.justify_content = JustifyContent::Center.into();

    // Board container: 8 rows stacked vertically
    let board = engine.insert_with(Element::Div(), |node| {
        node.display = Display::Flex;
        node.flex_direction = FlexDirection::Column;
    });

    // Build each rank (row)
    for (rank_idx, rank) in CHESS_BOARD.iter().enumerate() {
        let row = engine.insert_at_with(Element::Div(), At::Child(board), |node| {
            node.display = Display::Flex;
            node.flex_direction = FlexDirection::Row;
        });

        // Build each file (column) in this rank
        for (file_idx, &piece) in rank.iter().enumerate() {
            let light_square = (rank_idx + file_idx) % 2 == 0;
            let bg = if light_square {
                Color::Rgb(240, 217, 181) // light wood
            } else {
                Color::Rgb(181, 136, 99) // dark wood
            };

            let fg = if rank_idx < 2 {
                Color::Rgb(0, 0, 0) // black pieces → dark text
            } else if rank_idx > 5 {
                Color::Rgb(255, 255, 255) // white pieces → light text
            } else {
                Color::None // empty squares
            };

            engine.insert_at(
                Element::Span(piece)
                    .background(bg)
                    .color(fg)
                    .padding((1, 2))
                    .bold(),
                At::Child(row),
            );
        }
    }
}
