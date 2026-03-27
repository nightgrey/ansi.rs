use crate::Escape;

/// [DECSTBM] - Set Top and Bottom Margins
///
/// Sets the top and bottom margins for the current page. Scrolling cannot be performed
/// outside the margins.
///
/// ## Format
///
/// **CSI** *Pt* **;** *Pb* **r**
///
/// ## Parameters
/// - `Pt` is the line number for the top margin
/// - `Pb` is the line number for the bottom margin
///
/// ## Notes
/// - The value of the top margin (`Pt`) must be less than the bottom margin (`Pb`)
/// - The maximum size of the scrolling region is the page size
/// - DECSTBM moves the cursor to column 1, line 1 of the page
/// - Default margins are at the page limits
///
/// [`DECSTBM`]: https://vt100.net/docs/vt510-rm/DECSTBM.html
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SetTopBottomMargins {
    None,
    Some(usize, usize),
}

impl Escape for SetTopBottomMargins {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::None => {
                write!(w, "\x1B[r")
            }
            Self::Some(top, bottom) => {
                write!(w, "\x1B[{};{}r", top + 1, bottom + 1)
            }
        }
    }
}

pub type DECSTBM = SetTopBottomMargins;
