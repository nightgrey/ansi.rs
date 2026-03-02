use ansi::Style;

/// Tracks the logical cursor position and current SGR pen state.
#[derive(Clone, Debug)]
pub(crate) struct Cursor {
    pub row: usize,
    pub col: usize,
    pub style: Style,
}

impl Cursor {
    pub const fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            style: Style::EMPTY,
        }
    }

    /// Reset cursor to origin with empty pen.
    pub fn reset(&mut self) {
        self.row = 0;
        self.col = 0;
        self.style = Style::EMPTY;
    }

    /// Move logical cursor to (row, col).
    pub fn move_to(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
