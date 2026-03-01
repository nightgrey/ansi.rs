//! Custom assertions for testing.

use geometry::{Position, Rect};
use sigil::Buffer;

macro_rules! assert_rect {
    ($rect: tt) => {};
}

/// Assert that a rectangle is valid (min <= max in both dimensions).
///
/// # Panics
/// Panics if the rectangle is inverted.
pub fn assert_rect_valid(rect: &Rect) {
    assert!(
        rect.min.x <= rect.max.x,
        "Rectangle has inverted x: min.x ({}) > max.x ({})",
        rect.min.x,
        rect.max.x
    );
    assert!(
        rect.min.y <= rect.max.y,
        "Rectangle has inverted y: min.y ({}) > max.y ({})",
        rect.min.y,
        rect.max.y
    );
}

/// Assert that a rectangle has the expected dimensions.
pub fn assert_rect_size(rect: &Rect, width: usize, height: usize) {
    assert_eq!(
        rect.width(),
        width,
        "Expected width {}, got {}",
        width,
        rect.width()
    );
    assert_eq!(
        rect.height(),
        height,
        "Expected height {}, got {}",
        height,
        rect.height()
    );
}

/// Assert that a rectangle is at the expected position.
pub fn assert_rect_position(rect: &Rect, x: usize, y: usize) {
    assert_eq!(rect.x(), x, "Expected x {}, got {}", x, rect.x());
    assert_eq!(rect.y(), y, "Expected y {}, got {}", y, rect.y());
}

/// Extract text from a buffer at a specific position range.
///
/// Useful for comparing rendered text output.
pub fn buffer_text_at(buffer: &Buffer, row: usize, col_start: usize, col_end: usize) -> String {
    let mut result = String::new();
    for col in col_start..col_end {
        let pos = Position::new(row, col);
        if let Some(cell) = buffer.get(pos) {
            result.push_str(cell.as_str(&buffer.arena));
        }
    }
    result
}

/// Assert that a buffer contains the expected text at a specific row.
pub fn assert_buffer_text(buffer: &Buffer, row: usize, expected: &str) {
    let actual = buffer_text_at(buffer, row, 0, buffer.width);
    let trimmed = actual.trim_end();
    assert_eq!(
        trimmed, expected,
        "Expected '{}' at row {}, got '{}'",
        expected, row, trimmed
    );
}
