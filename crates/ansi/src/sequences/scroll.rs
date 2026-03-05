sequence!(
    /// [DECSTBM] — Set Top and Bottom Margins
    ///
    /// Sets the top and bottom margins for the current page.
    ///
    /// ## Format
    ///
    /// **CSI** *Pt* **;** *Pb* **r**
    ///
    /// ## Parameters
    /// - `Pt` is the line number for the top margin (0-indexed, converted to 1-indexed).
    /// - `Pb` is the line number for the bottom margin (0-indexed, converted to 1-indexed).
    ///
    /// ## Description
    /// This control function sets the top and bottom margins for the current page.
    /// You cannot perform scrolling outside the margins.
    /// DECSTBM moves the cursor to column 1, line 1 of the page.
    ///
    /// ## Notes
    /// - The value of the top margin must be less than the bottom margin.
    /// - The maximum size of the scrolling region is the page size.
    ///
    /// [`DECSTBM`]: https://vt100.net/docs/vt510-rm/DECSTBM.html
    pub struct SetMargins(pub usize, pub usize) => |this, w| {
        write!(w, "\x1B[{};{}r", this.0 + 1, this.1 + 1)
    }
);

sequence!(
    /// [DECSTBM] — Reset Top and Bottom Margins
    ///
    /// Resets the scrolling region to the full screen (page size).
    ///
    /// ## Format
    ///
    /// **CSI** **r**
    ///
    /// ## Description
    /// This control function resets the top and bottom margins to the page limits,
    /// allowing scrolling on the entire screen.
    ///
    /// [`DECSTBM`]: https://vt100.net/docs/vt510-rm/DECSTBM.html
    pub struct ResetMargins => |this, w| {
        write!(w, "\x1B[r")
    }
);

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