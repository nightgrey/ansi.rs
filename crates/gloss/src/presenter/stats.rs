/// Per-frame counters returned by [`Presenter::present`].
///
/// All fields are populated regardless of which path produced the frame
/// (full-paint, diff, dirty). `bytes` is read from the byte-counting writer
/// after flush, so it reflects what was actually emitted.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PresentStats {
    /// Number of base cells emitted to the terminal.
    pub cells: usize,
    /// Number of distinct emission spans (a span ends with a cursor move).
    pub runs: usize,
    /// Bytes written for this frame, after flush.
    pub bytes: u64,
}
