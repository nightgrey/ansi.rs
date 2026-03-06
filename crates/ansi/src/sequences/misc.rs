sequence!(
    /// @TODO: Find documentation - which sequence is this? Documentation is LLM, find real one.
    ///
    /// [REP] - Repeat
    ///
    /// Repeat the preceding character `n` times.
    ///
    /// ## Format
    ///
    /// **CSI** *Pn* **b**
    ///
    /// ## Parameters
    /// - `Pn` is the number of times to repeat the preceding character.
    ///
    /// [`REP`]: https://vt100.net/docs/vt510-rm/REP.html
    pub struct Repeat(pub usize) => |this, w| {
        write!(w, "\x1B[{}b", this.0)
    }
);

pub type REP = Repeat;
