use std::fmt::Debug;
use bitflags::Flags;
use crate::{Color, Escape, Style};
use derive_more::{Deref, DerefMut};
use etwa::Maybe;
use utils::separate_by;

/// [SGR] - Select Graphic Rendition
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

impl SelectGraphicRendition {
    pub const RESET: SelectGraphicRenditionReset = SelectGraphicRenditionReset;

    pub const fn reset() -> SelectGraphicRenditionReset {
        Self::RESET
    }

    pub const fn transition(from: Style, to: Style) -> SelectGraphicRenditionTransition {
        SelectGraphicRenditionTransition { from, to }
    }
}

impl Escape for SelectGraphicRendition {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        Style::escape(&self.0, w)
    }
}

pub type SGR = SelectGraphicRendition;

/// [SGR] - Select Graphic Rendition (Diff)
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
    PartialEq,
    derive_more::Constructor,
    derive_more::From,
    derive_more::Into,
)]
pub struct SelectGraphicRenditionTransition {
    pub from: Style,
    pub to: Style,
}

impl SelectGraphicRenditionTransition {
    pub const RESET: SelectGraphicRenditionReset = SelectGraphicRenditionReset;
}

impl Escape for SelectGraphicRenditionTransition {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use crate::io::Write as _;
        use std::io::Write as _;

        let from = self.from;
        let to = self.to;

        if from.is_none() && to.is_none() {
            return w.write_all(b"\x1B[0m");
        }

        w.write_all(b"\x1B[")?;

        separate_by!(w.write_all(b";")?);

        match (from.background, to.background) {
            (Color::None, Color::None) => {},
            (from, Color::None) => {
                separate!(w.write_all(b"49")?);
            },
            (Color::None, to) => {
                separate!(w.escape(to.as_background())?);
            },
            (_, to) => {
                separate!(w.escape(to.as_background())?);
            },
        };
        match (from.foreground, to.foreground) {
            (Color::None, Color::None) => {},
            (from, Color::None) => {
                separate!(w.write_all(b"39")?);
            },
            (Color::None, to) => {
                separate!(w.escape(to.as_foreground())?);
            },
            (_, to) => {
                separate!(w.escape(to.as_foreground())?);
            },
        };

        // Attributes (bold, underline, etc.)
        for attr in to.attributes.union(from.attributes) {
            match (from.contains(attr), to.contains(attr)) {
                (true, true) | (false, false) => continue,
                (true, false) => {
                    separate!(w.write_all(attr.sgr_unset().as_bytes())?);
                },
                (false, true) => {
                    separate!(w.write_all(attr.sgr().as_bytes())?);
                },
            }
        }

        w.write_all(b"m")
    }
}

impl Debug for SelectGraphicRenditionTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from;
        let to = self.to;

        let mut debug = f.debug_tuple("SelectGraphicRenditionTransition");

        if from.is_none() && to.is_none() {
            return debug.field(&"Reset").finish();
        }

        match (from.background, to.background) {
            (Color::None, Color::None) => {
            }
            (_, Color::None) => {
                debug.field(&"Background Reset");
            },
            (_, to) => {
                debug.field(&format!("Background ({:?})", to));
            },
        };

        match (from.foreground, to.foreground) {
            (Color::None, Color::None) => {
            },
            (from, Color::None) => {
                debug.field(&"Foreground Reset");
            },
            (_, to) => {
                debug.field(&format!("Foreground ({:?})", to));
            },
        };

        // Attributes (bold, underline, etc.)
        for attr in to.attributes.union(from.attributes) {
            match (from.contains(attr), to.contains(attr)) {
                (true, true) | (false, false) => continue,
                (true, false) => {
                    debug.field(&format!("Unset ({:?})", attr.to_string()));
                },
                (false, true) => {
                    debug.field(&format!("Set ({:?})", attr.to_string()));
                },
            }
        }

        debug.finish()
    }
}
pub type SGRDiff = SelectGraphicRenditionTransition;


/// [SGR] - Select Graphic Rendition Reset
///
/// Resets all attributes to their default values.
///
/// ## Format
///
/// **CSI** 0 **m**
///
/// ## Parameters
/// - None.
///
/// This control function resets all visual character attributes to their default values.
///
/// [`SGR`]: https://vt100.net/docs/vt510-rm/SGR.html
pub struct SelectGraphicRenditionReset;

impl Escape for SelectGraphicRenditionReset {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(b"\x1B[0m")
    }
}

pub type SGRReset = SelectGraphicRenditionReset;
