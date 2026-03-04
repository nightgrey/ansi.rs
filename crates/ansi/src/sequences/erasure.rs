sequence!(
    /// [EL] — Erase in Line (to end)
    ///
    /// Erases characters from the cursor through the end of the line.
    ///
    /// ## Format
    ///
    /// **CSI** **K** (or **CSI** 0 **K**)
    ///
    /// ## Description
    /// This control function erases characters on the line that has the cursor.
    /// EL clears all character attributes from erased character positions.
    /// EL works inside or outside the scrolling margins.
    ///
    /// [`EL`]: https://vt100.net/docs/vt510-rm/EL.html
    pub struct EraseLineToEnd => |_this: &Self, w: &mut dyn std::io::Write| {
        write!(w, "\x1B[K")
    }
);

sequence!(
    /// [EL] — Erase in Line (to start)
    ///
    /// Erases characters from the beginning of the line through the cursor.
    ///
    /// ## Format
    ///
    /// **CSI** 1 **K**
    ///
    /// ## Description
    /// This control function erases characters from the beginning of the line through
    /// the cursor position. EL clears all character attributes from erased character positions.
    ///
    /// [`EL`]: https://vt100.net/docs/vt510-rm/EL.html
    pub struct EraseLineToStart => |_this: &Self, w: &mut dyn std::io::Write| {
        write!(w, "\x1B[1K")
    }
);

sequence!(
    /// [ED] — Erase in Display (to end)
    ///
    /// Erases characters from the cursor through the end of the display.
    ///
    /// ## Format
    ///
    /// **CSI** **J** (or **CSI** 0 **J**)
    ///
    /// ## Description
    /// This control function erases characters from part or all of the display.
    /// When you erase complete lines, they become single-height, single-width lines,
    /// with all visual character attributes cleared. ED works inside or outside the scrolling margins.
    ///
    /// [`ED`]: https://vt100.net/docs/vt510-rm/ED.html
    pub struct EraseDisplayToEnd => |_this: &Self, w: &mut dyn std::io::Write| {
        write!(w, "\x1B[J")
    }
);

sequence!(
    /// [ED] — Erase in Display (entire)
    ///
    /// Erases the complete display.
    ///
    /// ## Format
    ///
    /// **CSI** 2 **J**
    ///
    /// ## Description
    /// This control function erases the complete display. When you erase complete lines,
    /// they become single-height, single-width lines, with all visual character attributes cleared.
    /// ED works inside or outside the scrolling margins.
    ///
    /// **Programming Tip**
    /// Use this to erase the complete display in a fast, efficient manner.
    ///
    /// [`ED`]: https://vt100.net/docs/vt510-rm/ED.html
    pub struct EraseDisplay => |_this: &Self, w: &mut dyn std::io::Write| {
        write!(w, "\x1B[2J")
    }
);

sequence!(
    /// [REP] — Repeat
    ///
    /// Repeat the preceding character n times.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **b**
    ///
    /// ## Parameters
    /// - `Pn` is the number of times to repeat the character (0 means 1).
    ///
    /// ## Description
    /// This control function repeats the preceding graphic character n times.
    /// If the count is 0, nothing is emitted.
    pub struct Repeat(pub usize) => |this: &Self, w: &mut dyn std::io::Write| {
        if this.0 > 0 {
            write!(w, "\x1B[{}b", this.0)
        } else {
            Ok(())
        }
    }
);

sequence!(
    /// [ECH] — Erase Character
    ///
    /// Erase n characters at the cursor position (without moving cursor).
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **X**
    ///
    /// ## Parameters
    /// - `Pn` is the number of characters to erase. Default: 1.
    ///
    /// ## Description
    /// This control function erases one or more characters, from the cursor position to
    /// the right. ECH clears character attributes from erased character positions.
    /// ECH works inside or outside the scrolling margins.
    ///
    /// [`ECH`]: https://vt100.net/docs/vt510-rm/ECH.html
    pub struct EraseCharacter(pub usize) => |this: &Self, w: &mut dyn std::io::Write| {
        if this.0 > 0 {
            write!(w, "\x1B[{}X", this.0)
        } else {
            Ok(())
        }
    }
);