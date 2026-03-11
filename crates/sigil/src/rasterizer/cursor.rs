use ansi::{escape, sequences::*, Escape, Style,};
use ansi::io::Write;
use super::capabilities::Capabilities;

/// Tracks the logical cursor position and current style state.
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

    /// Emit the shortest cursor movement sequence to reach `(target_row, target_col)`.
    ///
    /// Evaluates up to 4 strategies and picks the one with the fewest bytes:
    /// 0. CUP (absolute)
    /// 1. Relative (CUU/CUD + CUF/CUB)
    /// 2. CR + relative vertical + CUF
    /// 3. VPA + CHA (if capabilities allow)
    pub fn move_to(
        &mut self,
        row: usize,
        col: usize,
        output: &mut Vec<u8>,
        caps: Capabilities,
    ) {
        if self.row == row && self.col == col {
            return;
        }

        let dr = row as isize - self.row as isize;
        let dc = col as isize - self.col as isize;

        // Strategy 0: CUP (always available)
        let cost_cup = CursorPosition(row, col).cost();

        // Strategy 1: Relative moves
        let vert_cost = CursorUp(dr.unsigned_abs()).cost();
        let horiz_cost = CursorForward(dc.unsigned_abs()).cost();
        let cost_relative = vert_cost + horiz_cost;

        // Strategy 2: CR + relative vertical + CUF
        // CR is 1 byte, then vertical move, then CUF to target_col
        let cost_cr = 1 + vert_cost + CursorForward(col).cost();

        // Strategy 3: VPA + CHA (requires capabilities)
        let cost_vpa_cha = if caps.contains(Capabilities::VPA | Capabilities::CHA) {
            let v = if dr != 0 { VerticalPositionAbsolute(row).cost() } else { 0 };
            let h = if dc != 0 || dr != 0 {
                HorizontalPositionAbsolute(col).cost()
            } else {
                0
            };
            v + h
        } else {
            usize::MAX
        };

        // Pick the cheapest strategy.
        let min = cost_cup.min(cost_relative).min(cost_cr).min(cost_vpa_cha);

        if min == cost_relative && cost_relative > 0 {
            // Relative moves
            if dr > 0 {
                output.escape(CursorDown(dr as usize)).unwrap();
            } else if dr < 0 {
                output.escape(CursorUp((-dr) as usize)).unwrap();
            }
            if dc > 0 {
                output.escape(CursorForward(dc as usize)).unwrap();
            } else if dc < 0 {
                output.escape(CursorBackward((-dc) as usize)).unwrap();
            }
        } else if min == cost_cr {
            output.escape(CarriageReturn).unwrap();
            if dr > 0 {
                output.escape(CursorDown(dr as usize)).unwrap();
            } else if dr < 0 {
                output.escape(CursorUp((-dr) as usize)).unwrap();
            }
            if col > 0 {
                output.escape(CursorForward(col)).unwrap();
            }
        } else if min == cost_vpa_cha {
            if dr != 0 {
                output.escape(VerticalPositionAbsolute(row)).unwrap();
            }
            if dc != 0 || dr != 0 {
                output.escape(HorizontalPositionAbsolute(col)).unwrap();
            }
        } else {
            output.escape(CursorPosition(row, col)).unwrap();
        }

        self.row = row;
        self.col = col;
    }

    /// Emit a relative-only cursor movement sequence (no CUP, VPA, CHA).
    ///
    /// Used in inline mode where the rasterizer doesn't know its absolute
    /// screen position. Evaluates two strategies:
    /// 1. Pure relative (CUU/CUD + CUF/CUB)
    /// 2. CR + vertical + CUF
    pub fn move_to_relative(
        &mut self,
        row: usize,
        col: usize,
        output: &mut Vec<u8>,
    ) {
        if self.row == row && self.col == col {
            return;
        }

        let dr = row as isize - self.row as isize;
        let dc = col as isize - self.col as isize;

        // Strategy 1: Pure relative
        let vert_cost = CursorUp(dr.unsigned_abs()).cost();
        let horiz_cost = CursorForward(dc.unsigned_abs()).cost();
        let cost_relative = vert_cost + horiz_cost;

        // Strategy 2: CR + vertical + CUF
        let cost_cr = 1 + vert_cost + CursorForward(col).cost();

        if cost_cr < cost_relative {
            output.escape(CarriageReturn).unwrap();
            if dr > 0 {
                output.escape(CursorDown(dr as usize)).unwrap();
            } else if dr < 0 {
                output.escape(CursorUp((-dr) as usize)).unwrap();
            }
            if col > 0 {
                output.escape(CursorForward(col)).unwrap();
            }
        } else if cost_relative > 0 {
            if dr > 0 {
                output.escape(CursorDown(dr as usize)).unwrap();
            } else if dr < 0 {
                output.escape(CursorUp((-dr) as usize)).unwrap();
            }
            if dc > 0 {
                output.escape(CursorForward(dc as usize)).unwrap();
            } else if dc < 0 {
                output.escape(CursorBackward((-dc) as usize)).unwrap();
            }
        }

        self.row = row;
        self.col = col;
    }

    /// Update the pen (SGR state) to match `target_style`, emitting only
    /// the diff. No-op if the style is already current.
    pub fn update_style(&mut self, output: &mut Vec<u8>, style: &Style) {
        if self.style == *style {
            return;
        }

        let diff = self.style.diff(*style);
        if !diff.is_empty() {
            diff.escape(output).ok();
        }

        self.style = *style;
    }

    /// Reset the pen to default, emitting SGR 0 only if the pen is dirty.
    pub fn reset_style(&mut self, buf: &mut Vec<u8>) {
        if !self.style.is_empty() {
            escape(buf, Reset);
            self.style = Style::EMPTY;
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_to_same_position_is_noop() {
        let mut cursor = Cursor::new();
        let mut buf = Vec::new();
        cursor.move_to(0, 0, &mut buf, Capabilities::DEFAULT);
        assert!(buf.is_empty());
    }

    #[test]
    fn move_to_picks_shortest_for_small_relative() {
        let mut cursor = Cursor::new();
        let mut buf = Vec::new();
        // Moving right by 1 should use CUF (3 bytes) not CUP (6+ bytes)
        cursor.move_to(0, 1, &mut buf, Capabilities::DEFAULT);
        assert_eq!(buf, b"\x1B[C");
    }

    #[test]
    fn move_to_uses_cr_for_column_zero() {
        let mut cursor = Cursor::new();
        cursor.row = 5;
        cursor.col = 10;
        let mut buf = Vec::new();
        // Same row, col 0 — CR (1 byte) is cheapest
        cursor.move_to(5, 0, &mut buf, Capabilities::DEFAULT);
        assert_eq!(buf, b"\r");
    }

    #[test]
    fn move_to_uses_cup_for_distant_positions() {
        let mut cursor = Cursor::new();
        let mut buf = Vec::new();
        cursor.move_to(50, 80, &mut buf, Capabilities::DEFAULT);
        let output = String::from_utf8_lossy(&buf);
        // Should use some form of absolute positioning
        assert!(output.contains('\x1B'));
        assert_eq!(cursor.row, 50);
        assert_eq!(cursor.col, 80);
    }

    #[test]
    fn pen_elision_no_sgr_for_same_style() {
        let mut cursor = Cursor::new();
        cursor.style = Style::new().bold();
        let mut buf = Vec::new();
        cursor.update_style(&mut buf, &Style::new().bold());
        assert!(buf.is_empty());
    }

    #[test]
    fn pen_reset_only_when_dirty() {
        let mut cursor = Cursor::new();
        let mut buf = Vec::new();
        // Clean pen — no output
        cursor.reset_style(&mut buf);
        assert!(buf.is_empty());

        // Dirty pen — emits SGR reset
        cursor.style = Style::new().bold();
        cursor.reset_style(&mut buf);
        assert_eq!(buf, b"\x1B[0m");
        assert!(cursor.style.is_empty());
    }
}
