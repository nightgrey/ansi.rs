use crossbeam::epoch::Pointable;
use parking_lot::{Mutex, MutexGuard};
use rustix::termios::{OptionalActions, Termios, tcgetattr, tcsetattr};
use utils::slot;

use std::io;

slot!(pub RawModeSlot, Termios);

/// RAII guard that saves the original termios and restores it on drop.
///
/// This is the foundation for panic-safe terminal cleanup: even if the
/// application panics, the Drop impl runs (unless `panic = "abort"`) and
/// the terminal returns to its original state.
///
/// The guard opens `/dev/tty` to get an owned fd that is valid for the
/// lifetime of the guard, avoiding unsafe `BorrowedFd` construction.
pub struct RawModeGuard {
    original: Termios,
    file: std::fs::File,
}

impl RawModeGuard {
    /// Enter raw mode on the controlling terminal, returning a guard that
    /// restores the original termios on drop.
    pub fn new() -> io::Result<Self> {
        Self::with(std::fs::File::open("/dev/tty")?)
    }

    /// Enter raw mode on a specific terminal file (e.g., a PTY slave for testing).
    pub fn with(file: std::fs::File) -> io::Result<Self> {
        let original = tcgetattr(&file).map_err(io::Error::other)?;

        let mut raw = original.clone();
        tcsetattr(&file, OptionalActions::Flush, &raw).map_err(io::Error::other)?;

        RawModeSlot::set(original.clone());

        Ok(Self { original, file })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Best-effort restore — ignore errors during cleanup.
        let _ = tcsetattr(&self.file, OptionalActions::Flush, &self.original);
        RawModeSlot::clear();
    }
}
