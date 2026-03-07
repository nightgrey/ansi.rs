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
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq
)]
#[repr(transparent)]
pub struct SetMode<M = Mode>(pub M);

impl<M> const SetMode<M> {
    #[inline]
    pub fn value(&self) -> &M {
        &self.0
    }
}

impl Escape for SetMode<Mode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self.0 {
            Mode::Ansi(ansi) => write!(w, "\x1B[{}h", ansi),
            Mode::Dec(dec) => write!(w, "\x1B[?{}h", dec),
        }
    }
}

impl Escape for SetMode<AnsiMode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\x1B[{}h", self.0)
    }
}

impl Escape for SetMode<DecMode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\x1B[?{}h", self.0)
    }
}

pub type SM<M = Mode> = SetMode<M>;


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
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
)]
#[repr(transparent)]
pub struct ResetMode<M = Mode>(pub M);

impl<M> const ResetMode<M> {
    #[inline]
    pub fn value(&self) -> &M {
        &self.0
    }
}

impl Escape for ResetMode<Mode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self.0 {
            Mode::Ansi(ansi) => write!(w, "\x1B[{}l", ansi),
            Mode::Dec(dec) => write!(w, "\x1B[?{}l", dec),
        }
    }
}

impl Escape for ResetMode<AnsiMode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\x1B[{}l", self.0)
    }
}

impl Escape for ResetMode<DecMode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\x1B[?{}l", self.0)
    }
}

pub type RM<M = Mode> = ResetMode<M>;

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
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
)]
pub struct ReportMode<M = Mode> {
    pub mode: M,
    pub setting: ModeSetting,
}

impl Escape for ReportMode<Mode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self.mode {
            Mode::Ansi(ansi) => write!(w, "\x1B[{};{}$y", ansi, self.setting),
            Mode::Dec(dec) => write!(w, "\x1B[?{};{}$y", dec, self.setting),
        }
    }
}

impl Escape for ReportMode<AnsiMode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\x1B[{};{}$y", self.mode, self.setting)
    }
}

impl Escape for ReportMode<DecMode> {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(w, "\x1B[?{};{}$y", self.mode, self.setting)
    }
}

pub type DECRPM<M = Mode> = ReportMode<M>;


