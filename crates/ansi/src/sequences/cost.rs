use super::*;
use crate::Escape;

/// Byte-length calculation for escape sequences.
///
/// This is the static dual of `Escape`—`cost()` must return the exact
/// number of bytes that `escape()` would write.
pub trait Cost: Escape {
    fn cost(&self) -> usize;
}

/// Width of the decimal representation of an unsigned integer.
///
/// Returns the number of ASCII bytes (characters) required to encode `n` in base 10.
/// This is always ≥ 1, since `0` is encoded as `"0"` (one byte).
///
/// Used to calculate exact byte lengths for CSI (Control Sequence Introducer)
/// parameters, which are transmitted as 1-indexed decimal ASCII strings.
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
        // \x1B [ digits `
        2 + decimal_width(self.0 + 1) + 1
    }
}

impl Cost for VerticalPositionAbsolute {
    fn cost(&self) -> usize {
        // \x1B [ digits d
        2 + decimal_width(self.0 + 1) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Write;

    macro_rules! assert_cost {
        (@$sequence:expr) => {
            let actual = $sequence.cost();
            let expected = {
                use crate::Escape;
                let mut buf = Vec::new();
                buf.escape($sequence).and_then(|_| Ok(buf.len())).unwrap()
            };
            let label = $sequence;
            assert_eq!(actual, expected, "Cost of {label:?} expected to be {expected}, but was {actual}");
        };
        ($sequences:expr) => {
            for seq in $sequences {
                assert_cost!(@seq);
            }
        };
    }

    // Representative parameter values covering edge cases:
    // 0 (n=0 no-op), 1 (short form), 9 (1 digit), 10 (2 digits), 100 (3 digits)
    const PARAMS: &[usize] = &[0, 1, 2, 5, 9, 10, 42, 99, 100, 999];

    #[test]
    fn cursor_position_cost() {
        assert_cost!(
            PARAMS
                .iter()
                .flat_map(|&r| PARAMS.iter().map(move |&c| CursorPosition(r, c)))
        );
    }

    #[test]
    fn cursor_up_cost() {
        assert_cost!(PARAMS.iter().map(|&n| CursorUp(n)));
    }

    #[test]
    fn cursor_down_cost() {
        assert_cost!(PARAMS.iter().map(|&n| CursorDown(n)));
    }

    #[test]
    fn cursor_forward_cost() {
        assert_cost!(PARAMS.iter().map(|&n| CursorForward(n)));
    }

    #[test]
    fn cursor_backward_cost() {
        assert_cost!(PARAMS.iter().map(|&n| CursorBackward(n)));
    }

    #[test]
    fn vertical_position_absolute_cost() {
        assert_cost!(PARAMS.iter().map(|&n| VerticalPositionAbsolute(n)));
    }

    #[test]
    fn horizontal_position_absolute_cost() {
        assert_cost!(PARAMS.iter().map(|&n| HorizontalPositionAbsolute(n)));
    }

    #[test]
    fn set_cursor_style_cost() {
        assert_cost!([
            SetCursorStyle::Default,
            SetCursorStyle::BlinkBlock,
            SetCursorStyle::SteadyBlock,
            SetCursorStyle::BlinkUnderline,
            SetCursorStyle::SteadyUnderline,
        ]);
    }
}
