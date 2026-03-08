use utils::separate_by;
use crate::{Escape, Style};
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
#[derive(Copy, Clone, Debug,  PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
#[repr(transparent)]
pub struct SelectGraphicRendition(pub Style);



impl Escape for SelectGraphicRendition {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let this = self;
        use crate::io::Write;
        // Start CSI sequence
        w.write(b"\x1B[")?;
        let this = &this.0;

        if this.is_empty() {
            w.write(b"0m")?;
            return Ok(());
        }

        let bg = &this.bg;
        let fg = &this.fg;
        let ul = &this.ul;

        separate_by!({ w.write(b";") });

        // Background color
        if bg.is_some() {
            separate!(w.escape(bg.as_background())?);
        }

        // Foreground color
        if fg.is_some() {
            separate!(w.escape(fg.as_foreground())?);
        }

        // Underline color
        if ul.is_some() {
            separate!(w.escape(ul.as_underline())?);
        }

        // Attributes (bold, underline, etc.)
        for attr in this.attributes.sgr() {
            separate!(w.write_all(attr.as_bytes())?);
        }

        // Terminate CSI sequence
        write!(w, "m")
    }
}

pub type SGR = SelectGraphicRendition;


sequence!(
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
    pub struct Reset => |this, w| {
        write!(w, "\x1B[0m")
    }
);

