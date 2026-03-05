use crate::{cost, decimal_width, sequence, Escape};
use derive_more::{Deref, DerefMut};

sequence!(
    /// [DECSCUSR] - Set Cursor Style
    ///
    /// Select the style of the cursor on the screen.
    ///
    /// ## Format
    ///
    /// **CSI** *Ps* **SP** q
    ///
    /// ## Parameters
    /// - `Ps` indicates the style of the cursor.
    ///
    /// This sequence causes the cursor to be displayed in a different style when the cursor is enabled.
    ///
    /// [`DECSCUSR`]: https://vt100.net/docs/vt510-rm/DECSCUSR.html
    #[derive(Default)]
    pub enum SetCursorStyle {
        #[default]
        /// Blink Block
        Default = 0,
        /// Blink Block
        BlinkBlock = 1,
        /// Steady Block
        SteadyBlock = 2,
        /// Blink Underline
        BlinkUnderline = 3,
        /// Steady Underline
        SteadyUnderline = 4,
    } => |this, w| {
        write!(w, "\x1B{}q", *this as usize)
    }
);



pub type DECSCUSR = SetCursorStyle;

sequence!(
    /// [CUP] - Cursor Position
    ///
    /// This control function moves the cursor to the specified line and column. The starting point for lines and columns depends on the setting of origin mode (DECOM). CUP applies only to the current page.
    ///
    /// ## Format
    ///
    /// **CSI** *PI* ; *Pc* **H**
    ///
    /// ## Parameters
    /// - `PI` is the number of the line to move to. If Pl is 0 or 1, then the cursor moves to line 1.
    ///
    /// - `Pc` is the number of the column to move to. If Pc is 0 or 1, then the cursor moves to column 1.
    ///
    /// This control function moves the cursor to the left by a specified number of columns.
    /// The cursor stops at the left border of the page.
    ///
    /// [`CUB`]: https://vt100.net/docs/vt510-rm/CUB.html
    pub struct CursorPosition(pub usize, pub usize) => |this, w| {
        write!(w, "\x1B{};{}H", this.0 + 1, this.1 + 1)
    }
);

pub type CUP = CursorPosition;

sequence!(
    /// [CUB] - Cursor Backward
    ///
    /// Moves the cursor to the left by a specified number of columns.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **D**
    ///
    /// ## Parameters
    /// - `Pn` is the number of columns to move the cursor to the left
    ///
    /// This control function moves the cursor to the left by a specified number of columns.
    /// The cursor stops at the left border of the page.
    ///
    /// [`CUB`]: https://vt100.net/docs/vt510-rm/CUB.html
    #[derive(Deref, DerefMut)]
    pub struct CursorBackward(pub usize) => |this, w| { write!(w, "\x1B{}D", this.0) }
);

pub type CUB = CursorBackward;



sequence!(
    /// [CUD] - Cursor Down
    ///
    /// Moves the cursor down a specified number of lines in the same column.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **B**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to move the cursor down
    ///
    /// The cursor stops at the bottom margin. If the cursor is already below the bottom margin,
    /// then the cursor stops at the bottom line.
    ///
    /// [`CUD`]: https://vt100.net/docs/vt510-rm/CUD.html
    #[derive(Deref, DerefMut)]
    pub struct CursorDown(pub usize) => |this, w| {
        write!(w, "\x1B{}B", this.0)
    }
);

pub type CUD = CursorDown;



sequence!(
    /// [CUF] - Cursor Forward
    ///
    /// Moves the cursor to the right by a specified number of columns.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **C**
    ///
    /// ## Parameters
    /// - `Pn` is the number of columns to move the cursor to the right
    ///
    /// The cursor stops at the right border of the page.
    ///
    /// [`CUF`]: https://vt100.net/docs/vt510-rm/CUF.html
    #[derive(Deref, DerefMut)]
    pub  struct CursorForward(pub usize) => |this, w| {
        write!(w, "\x1B{}C", this.0)
    }
);

pub type CUF = CursorForward;



sequence!(
    /// [CUU] - Cursor Up
    ///
    /// Moves the cursor up a specified number of lines in the same column.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **A**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to move the cursor up
    ///
    /// The cursor stops at the top margin. If the cursor is already above the top margin,
    /// then the cursor stops at the top line.
    ///
    /// [`CUU`]: https://vt100.net/docs/vt510-rm/CUU.html
    #[derive(Deref, DerefMut)]
    pub struct CursorUp(pub usize) => |this, w| {
        write!(w, "\x1B{}A", this.0)
    }
);

pub type CUU = CursorUp;

sequence!(
    /// [CPL] - Cursor Previous Line
    ///
    /// Move the cursor to the preceding line.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **F**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to move (default: 1).
    ///
    /// The active position is moved to the first character of the n-th preceding line.
    #[derive(Deref, DerefMut)]
    pub struct CursorPreviousLine(pub usize) => |this, w| {
        write!(w, "\x1B{}F", this.0)
    }
);

pub type CPL = CursorPreviousLine;

sequence!(
    /// [CNL] - Cursor Next Line
    ///
    /// Move the cursor to the next line.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **E**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to move (default: 1).
    ///
    /// The active position is moved to the first character of the n-th following line.
    #[derive(Deref, DerefMut)]
    pub struct CursorNextLine(pub usize) => |this, w| {
        write!(w, "\x1B{}E", this.0)
    }
);

pub type CNL = CursorNextLine;

sequence!(
    /// [CHT] - Cursor Horizontal Forward Tabulation
    ///
    /// Move the active position n tabs forward.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **I**
    ///
    /// ## Parameters
    /// - `Pn` is the number of tabs to move forward (default: 1).
    ///
    /// The active position is moved to the character position corresponding to the
    /// following n-th horizontal tabulation stop.
    #[derive(Deref, DerefMut)]
    pub struct CursorHorizontalForwardTabulation(pub usize) => |this, w| {
        write!(w, "\x1B{}I", this.0)
    }
);
pub type CHF = CursorHorizontalForwardTabulation;


sequence!(
    /// [CBT] - Cursor Backward Tabulation
    ///
    /// Move the active position n tabs backward.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **Z**
    ///
    /// ## Parameters
    /// - `Pn` is the number of tabs to move backward (default: 1).
    ///
    /// The active position is moved to the character position corresponding to the
    /// n-th preceding horizontal tabulation stop. If an attempt is made to move the
    /// active position past the first character position on the line, then the active
    /// position stays at column one.
    #[derive(Deref, DerefMut)]
    pub struct CursorBackwardTabulation(pub usize) => |this, w| {
        write!(w, "\x1B{}Z", this.0)
    }
);

type CBT = CursorBackwardTabulation;


sequence!(
    /// [CR] — Carriage Return
    ///
    /// Moves the cursor to the left margin on the current line.
    ///
    /// ## Format
    ///
    /// **CR** (0/13)
    ///
    /// ## Description
    ///
    /// This control character moves the cursor to the beginning of the current line
    /// (left margin). If New Line mode (LNM) is set, it also performs a line feed.
    pub struct CarriageReturn => |this, w| {
        write!(w, "\r")
    }
);

sequence!(
    /// [CHA] — Cursor Horizontal Absolute
    ///
    /// Move the active position to the n-th character of the active line.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **G**
    ///
    /// ## Parameters
    /// - `Pn` is the column number (0-indexed, converted to 1-indexed). Default: 0 (column 1).
    ///
    /// ## Description
    /// The active position is moved to the n-th character position of the active line.
    /// If an attempt is made to move the active position past the last position on the line,
    /// then the active position stops at the last position on the line.
    ///
    /// [`CHA`]: https://vt100.net/docs/vt510-rm/CHA.html
    pub struct CursorHorizontalAbsolute(pub usize) => |this, w| {
        write!(w, "\x1B[{}G", this.0 + 1)
    }
);

sequence!(
    /// [VPA] — Vertical Position Absolute
    ///
    /// Move cursor to line Pn.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **d**
    ///
    /// ## Parameters
    /// - `Pn` is the line number (0-indexed, converted to 1-indexed). Default: 0 (line 1).
    ///
    /// ## Description
    /// VPA causes the active position to be moved to the corresponding horizontal
    /// position at vertical position Pn. If an attempt is made to move the active
    /// position below the last line, then the active position stops on the last line.
    ///
    /// [`VPA`]: https://vt100.net/docs/vt510-rm/VPA.html
    pub struct VerticalPositionAbsolute(pub usize) => |this, w| {
        write!(w, "\x1B[{}d", this.0 + 1)
    }
);

sequence!(
    /// [HPA] — Horizontal Position Absolute
    ///
    /// Move the active position to the n-th horizontal position of the active line.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **`**
    ///
    /// ## Parameters
    /// - `Pn` is the column number (0-indexed, converted to 1-indexed). Default: 0 (column 1).
    ///
    /// ## Description
    /// HPA causes the active position to be moved to the n-th horizontal position of
    /// the active line. If an attempt is made to move the active position past the last
    /// position on the line, then the active position stops at the last position on the line.
    ///
    /// [`HPA`]: https://vt100.net/docs/vt510-rm/HPA.html
    pub struct HorizontalPositionAbsolute(pub usize) => |this, w| {
        write!(w, "\x1B[{}`", this.0 + 1)
    }
);


sequence!(
    /// [DECTCEM] - Text Cursor Enable Mode
    ///
    /// This control function makes the cursor visible or invisible.
    ///
    /// ## Format
    ///
    /// **CSI** **?** **25** **h** (Set)
    ///
    /// **CSI** **?** **25** **l** (Reset)
    ///
    /// ## Description
    ///
    /// Controls the visibility of the text cursor.
    ///
    /// - **Set**: Makes the cursor visible.
    /// - **Reset**: Makes the cursor invisible.
    ///
    /// Default: Visible.
    ///
    /// [`DECTCEM`]: https://vt100.net/docs/vt510-rm/DECTCEM.html
    #[derive(Default)]
    pub enum CursorMode {
        /// Makes the cursor visible.
        #[default]
        Visible = 1,
        /// Makes the cursor invisible.
        Invisible = 0,
    } => |this, w| {
        write!(
            w,
            "\x1B?25{}",
            match this {
                CursorMode::Visible => 'h',
                CursorMode::Invisible => 'l',
            }
        )
    }
);

pub type DECTCEM = CursorMode;


sequence!(
    /// [DECSC] — Save Cursor
    ///
    /// Saves the current cursor state.
    ///
    /// ## Format
    ///
    /// **ESC** 7
    ///
    /// ## Description
    /// Saves the following items in the terminal's memory:
    /// - Cursor position
    /// - Character attributes set by the SGR command
    /// - Character sets (G0, G1, G2, or G3) currently in GL and GR
    /// - Wrap flag (autowrap or no autowrap)
    /// - State of origin mode (DECOM)
    /// - Selective erase attribute
    /// - Any single shift 2 (SS2) or single shift 3 (SS3) functions sent
    ///
    /// [`DECSC`]: https://vt100.net/docs/vt510-rm/DECSC.html
    pub struct SaveCursor => |this, w| {
        write!(w, "\x1B7")
    }
);


sequence!(
    /// [DECRC] — Restore Cursor
    ///
    /// Restores the cursor state saved by DECSC.
    ///
    /// ## Format
    ///
    /// **ESC** 8
    ///
    /// ## Description
    /// Restores the terminal to the state saved by the save cursor (DECSC) function.
    ///
    /// If nothing was saved by DECSC, then DECRC performs the following actions:
    /// - Moves the cursor to the home position (upper left of screen).
    /// - Resets origin mode (DECOM).
    /// - Turns all character attributes off (normal setting).
    /// - Maps the ASCII character set into GL, and the DEC Supplemental Graphic set into GR.
    ///
    /// [`DECRC`]: https://vt100.net/docs/vt510-rm/DECRC.html
    pub struct RestoreCursor => |this, w| {
        write!(w, "\x1B8")
    }
);