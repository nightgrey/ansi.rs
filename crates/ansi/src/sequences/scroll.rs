sequence!(
    /// [SU] — Scroll Up (Pan Down)
    ///
    /// Scrolls the user window down a specified number of lines in page memory.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **S**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to scroll. Default: 1.
    ///
    /// ## Description
    /// This control function moves the user window down a specified number of lines in
    /// page memory. Pn new lines appear at the bottom of the display. Pn old lines
    /// disappear at the top of the display. You cannot pan past the bottom margin of
    /// the current page.
    ///
    /// [`SU`]: https://vt100.net/docs/vt510-rm/SU.html
    pub struct ScrollUp(pub usize) => |this, w| {
        if this.0 == 1 {
            write!(w, "\x1B[S")
        } else if this.0 > 1 {
            write!(w, "\x1B[{}S", this.0)
        } else {
            Ok(())
        }
    }
);

sequence!(
    /// [SD] — Scroll Down (Pan Up)
    ///
    /// Scrolls the user window up a specified number of lines in page memory.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **T**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to scroll. Default: 1.
    ///
    /// ## Description
    /// This control function moves the user window up a specified number of lines in
    /// page memory. Pn new lines appear at the top of the display. Pn old lines
    /// disappear at the bottom of the display. You cannot pan past the top margin of
    /// the current page.
    ///
    /// [`SD`]: https://vt100.net/docs/vt510-rm/SD.html
    pub struct ScrollDown(pub usize) => |this, w| {
        if this.0 == 1 {
            write!(w, "\x1B[T")
        } else if this.0 > 1 {
            write!(w, "\x1B[{}T", this.0)
        } else {
            Ok(())
        }
    }
);

sequence!(
    /// [IL] — Insert Line
    ///
    /// Inserts `Pn` blank lines at the cursor position. Existing lines are pushed down.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **L**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to insert. Default: 1.
    ///
    /// [`IL`]: https://vt100.net/docs/vt510-rm/IL.html
    pub struct InsertLine(pub usize) => |this, w| {
        if this.0 == 1 {
            write!(w, "\x1B[L")
        } else if this.0 > 1 {
            write!(w, "\x1B[{}L", this.0)
        } else {
            Ok(())
        }
    }
);

sequence!(
    /// [DL] — Delete Line
    ///
    /// Deletes `Pn` lines at the cursor position. Lines below are pulled up.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **M**
    ///
    /// ## Parameters
    /// - `Pn` is the number of lines to delete. Default: 1.
    ///
    /// [`DL`]: https://vt100.net/docs/vt510-rm/DL.html
    pub struct DeleteLine(pub usize) => |this, w| {
        if this.0 == 1 {
            write!(w, "\x1B[M")
        } else if this.0 > 1 {
            write!(w, "\x1B[{}M", this.0)
        } else {
            Ok(())
        }
    }
);

sequence!(
    /// [RI] — Reverse Index
    ///
    /// Moves the cursor up one line in the same column. If the cursor is at the
    /// top margin, the page scrolls down.
    ///
    /// ## Format
    ///
    /// **ESC** **M**
    ///
    /// [`RI`]: https://vt100.net/docs/vt510-rm/RI.html
    pub struct ReverseIndex => |this, w| {
        write!(w, "\x1BM")
    }
);

sequence!(
    /// [CUP] — Cursor Position (Home)
    ///
    /// Moves the cursor to the home position (1,1).
    ///
    /// ## Format
    ///
    /// **CSI** **H** (or **CSI** 1 **;** 1 **H**)
    ///
    /// ## Description
    /// This is a convenience sequence that moves the cursor to the upper-left corner
    /// of the screen (line 1, column 1), which is the home position.
    ///
    /// [`CUP`]: https://vt100.net/docs/vt510-rm/CUP.html
    pub struct Home => |this, w| {
        write!(w, "\x1B[H")
    }
);