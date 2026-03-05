use crate::Escape;
use super::*;

/// Byte-length calculation for escape sequences.
///
/// This is the static dual of `Escape`—`cost()` must return the exact
/// number of bytes that `escape()` would write.
pub trait Cost {
    fn cost(&self) -> usize;
}

/// Width of the decimal representation of an unsigned integer.
///
/// Returns the number of ASCII bytes (characters) required to encode `n` in base 10.
/// This is always ≥ 1, since `0` is encoded as `"0"` (one byte).
///
/// Used to calculate exact byte lengths for CSI (Control Sequence Introducer)
/// parameters, which are transmitted as 1-indexed decimal ASCII strings.
///
/// # Examples
///
/// ```
/// assert_eq!(decimal_width(0), 1);    // "0"
/// assert_eq!(decimal_width(5), 1);    // "5"
/// assert_eq!(decimal_width(10), 2);   // "10"
/// assert_eq!(decimal_width(255), 3);  // "255"
/// assert_eq!(decimal_width(9999), 4); // "9999"
/// ```
#[inline]
pub const fn decimal_width(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut v = n;
    while v > 0 {
        count += 1;
        v /= 10;
    }
    count
}


/// Cost of a CSI sequence that moves the cursor relative to its current position.
#[inline(always)]
fn relative_cursor_cost(n: usize) -> usize {
    match n {
        0 => 0,
        1 => 3,
        n => 2 + decimal_width(n) + 1,
    }
}

impl Cost for SetCursorStyle {
    fn cost(&self) -> usize {
        2 + 1 + 1 + 1
    }
}

impl Cost for CursorPosition {
    fn cost(&self) -> usize {
        // CSI Pl ; Pc H  →  \x1B [ digits ; digits H
        2 + decimal_width(self.0 + 1) + 1 + decimal_width(self.1 + 1) + 1
    }
}

impl Cost for CursorBackward {
    fn cost(&self) -> usize {
        relative_cursor_cost(self.value())
    }
}

impl Cost for CursorDown {
    fn cost(&self) -> usize {
        relative_cursor_cost(self.value())
    }
}

impl Cost for CursorForward {
    fn cost(&self) -> usize {
        relative_cursor_cost(self.value())
    }
}

impl Cost for CursorUp {
    fn cost(&self) -> usize {
       relative_cursor_cost(self.value())
    }
}


impl Cost for HorizontalPositionAbsolute {
    fn cost(&self) -> usize {
        relative_cursor_cost(self.value())
    }
}

impl Cost for VerticalPositionAbsolute {
    fn cost(&self) -> usize {
        relative_cursor_cost(self.value())
    }
}
