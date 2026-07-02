use std::io;
use derive_more::Deref;
use super::*;
use crate::Escape;

/// [SM] - Set Mode
///
/// Sets one or more terminal modes.
///
/// ## Format
///
/// **DEC Format:** CSI ? *Pd* [; *Pd* ...] **h**
///
/// **ANSI Format:** CSI *Pa* **;** *Ps* **h**
///
/// ## Parameters
/// - `Pd` are mode values from [`Mode`]
/// [`RM`]: https://vt100.net/docs/vt510-rm/SM.html
#[derive(Copy, Debug, Deref)]
#[derive_const(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Set(pub Mode);

const impl Set {
    #[inline]
    pub fn value(&self) -> &Mode {
        &self.0
    }
}

impl Escape for Set {
    fn escape(&self, w: &mut dyn  io::Write) -> io::Result<()> {
        match self.kind() {
            ModeKind::Ansi => write!(w, "\x1B[{}h", self.0),
            ModeKind::Dec => write!(w, "\x1B[?{}h", self.0),
        }
    }
}

pub type SM = Set;

/// [RM] - Reset Mode
///
/// Resets one or more terminal modes.
///
/// ## Format
///
/// **DEC Format:** CSI ? *Pd* [; *Pd* ...] **l**
///
/// ## Parameters
/// - `Pd` are DEC mode values from [`DecMode`]
///
/// This control function resets one or more DEC modes. You cannot reset ANSI
/// and DEC modes with the same RM sequence.
///
/// ## Examples
///
/// The following sequence resets (Hebrew) keyboard mapping (DECHEBM) and Hebrew encoding mode (DECHEM):
///
/// ```text
/// CSI ? 34; 36 l
/// ```
///
/// - 34 indicates (Hebrew) keyboard mapping
/// - 36 indicates Hebrew encoding mode
///
/// ## Programming Tip
/// Applications can use the SM and RM functions to restore any number of VT510 modes to a desired state.
/// See the Report Mode (DECRPM) section in this chapter for details.
///
/// [`RM`]: https://vt100.net/docs/vt510-rm/RM.html
#[derive(Copy, Debug, Deref)]
#[derive_const(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Reset(pub Mode);


impl Escape for Reset {
    fn escape(&self, w: &mut dyn io::Write) -> io::Result<()> {
        match self.kind() {
            ModeKind::Ansi => write!(w, "\x1B[{}l", self.0),
            ModeKind::Dec => write!(w, "\x1B[?{}l", self.0),
        }
    }
}


pub type RM = Reset;

/// [DECRPM] - Report Mode - Terminal To Host
///
/// Reports the terminal mode in response to a request mode (DECRQM) function.
/// In the response DECRPM informs the host if a certain mode is set (SM—Set Mode)
/// or reset (RM—Reset Mode).
///
/// ## Format
///
/// **ANSI Modes:** CSI *Pa* **;** *Ps* **$** y
/// **DEC Modes:** CSI **?** *Pd* **;** *Ps* **$** y
///
/// ## Parameters
/// - `Pa` is [`AnsiMode`] (for ANSI modes)
/// - `Pd` is [`DecMode`] (for DEC modes)
/// - `Ps` is [`ModeSetting`]
///
/// ## Programming Tip
/// Applications can use the information in the DECRPM report to save the current
/// mode settings. Later, the application can restore the mode settings that were
/// saved.
///
/// This operation is useful for applications that need to temporarily change
/// some of the terminal's mode settings. When the application is finished, it
/// can restore the mode settings that were in effect before the application
/// changed them.
///
/// [`DECRPM`]: https://vt100.net/docs/vt510-rm/DECRPM.html
#[derive(Copy, Debug, Deref)]
#[derive_const(Clone, Eq, PartialEq)]
pub struct Report(#[deref] pub Mode, pub ModeSetting);

impl Report {
    #[inline]
    pub fn mode(&self) -> &Mode {
        &self.0
    }

    #[inline]
    pub fn setting(&self) -> &ModeSetting {
        &self.1
    }
}

impl Escape for Report {
    fn escape(&self, w: &mut dyn io::Write) -> io::Result<()> {
        match self.kind() {
            ModeKind::Ansi => write!(w, "\x1B[{};{}$y", self.0, self.1),
            ModeKind::Dec => write!(w, "\x1B[?{};{}$y", self.0, self.1),
        }
    }
}

pub type DECRPM = Report;
