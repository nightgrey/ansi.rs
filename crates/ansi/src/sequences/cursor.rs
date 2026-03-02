use crate::sequence;
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
    pub enum CursorStyle {
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
    } => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}q", *this as u16)
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
    pub enum Cursor {
        /// Makes the cursor visible.
        #[default]
        Visible = 1,
        /// Makes the cursor invisible.
        Invisible = 0,
    } => |this: &Self, w: &mut dyn std::io::Write| write!(
                w,
                "\x1B?25{}",
                match this {
                    Cursor::Visible => 'h',
                    Cursor::Invisible => 'l',
                }
    )
);

pub type DECTCEM = Cursor;

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
    pub struct CursorBackward(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}D", this.0)
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
    pub struct CursorDown(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}B", this.0)
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
    pub  struct CursorForward(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}C", this.0)
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
    pub struct CursorUp(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}A", this.0)
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
    pub struct CursorPreviousLine(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}F", this.0)
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
    pub struct CursorNextLine(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}E", this.0)
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
    pub struct CursorHorizontalForwardTabulation(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}I", this.0)
);
pub type CHF = CursorHorizontalForwardTabulation;


sequence!(
    /// [CHA] - Cursor Horizontal Absolute
    ///
    /// Move the active position to the n-th character of the active line.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **G**
    ///
    /// ## Parameters
    /// - `Pn` is the character position to move to (default: 1).
    ///
    /// The active position is moved to the n-th character position of the active line.
    #[derive(Deref, DerefMut)]
    pub struct CursorHorizontalAbsolute(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}G", this.0)
);

pub type CHA = CursorHorizontalAbsolute;

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
    pub struct CursorBackwardTabulation(pub u16) => |this: &Self, w: &mut dyn std::io::Write| write!(w, "\x1B{}Z", this.0)
);

type CBT = CursorBackwardTabulation;