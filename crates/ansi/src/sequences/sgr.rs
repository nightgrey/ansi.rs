use crate::{Escape, Style};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use etwa::Maybe;
use utils::separate_by;
/// [SGR] - Select Graphic Rendition
///
///
/// Selects one or more character attributes at the same time.
///
/// ## Format
///
/// **CSI** *Ps* [ **;** *Ps* ... ] **m**
///
/// ## Parameters
/// - `Ps` can be anything that implements [`Styleable`], for example [`Style`] or [`Attributes`].
///
/// This control function selects visual character attributes.
///
/// After you select an attribute, the terminal applies that attribute to all new
/// characters received. If you move characters by scrolling, then the attributes
/// move with the characters.
///
/// [`SGR`]: https://vt100.net/docs/vt510-rm/SGR.html
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    derive_more::Constructor,
    derive_more::From,
    derive_more::Into,
    Deref,
    DerefMut,
)]
#[repr(transparent)]
pub struct SelectGraphicRendition(pub Style);

#[allow(non_upper_case_globals)]
impl SelectGraphicRendition {
    pub const Reset: Self = Self(Style::None);
}

impl Escape for SelectGraphicRendition {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self.0.is_none() {
            true => w.write_all(b"\x1B[0m"),
            false => Style::escape(&self.0, w),
        }
    }
}

pub type SGR = SelectGraphicRendition;

/// [SGR] - Select Graphic Rendition (Reset)
///
/// A shortcut and efficient alternative to `SGR(Style::RESET)`.
/// When possible, prefer this over `SGR` with reset styles.
///
/// ## Format
///
/// **CSI** 0 **m**
///
/// [`SGR`]: https://vt100.net/docs/vt510-rm/SGR.html
#[allow(non_upper_case_globals)]
pub const Reset: SelectGraphicRendition = SelectGraphicRendition::Reset;
