//! Common modes
//!
//! Provides shortcuts to set ([`SetMode`]) and reset ([`ResetMode`]) common modes.
//!
//! For anything beyond, check out [`super::modes`].
use super::*;
use crate::Escape;
use crate::io::Write;
use crate::SetMode;

sequence!(
    pub enum AlternateScreen {
        Enable,
        Disable,
    } => |this, w| {
        match this {
            AlternateScreen::Enable => w.escape(SetMode(DecMode::AlternateScreen)),
            AlternateScreen::Disable => w.escape(ResetMode(DecMode::AlternateScreen)),
        }
    }
);
sequence!(
    pub enum BracketedPaste {
        Enable,
        Disable,
    } => |this, w| {
        match this {
            BracketedPaste::Enable => SetMode(DecMode::BracketedPaste).escape(w),
            BracketedPaste::Disable => ResetMode(DecMode::BracketedPaste).escape(w),
        }
    }
);

sequence!(
    pub enum SynchronizedOutput {
        Enable,
        Disable,
    } => |this, w| {
        match this {
            SynchronizedOutput::Enable => SetMode(DecMode::SynchronizedOutput).escape(w),
            SynchronizedOutput::Disable => ResetMode(DecMode::SynchronizedOutput).escape(w),
        }
    }
);



sequence!(
    #[derive(Default)]
    pub enum TextCursor {
        /// Makes the cursor visible.
        #[default]
        Enable,
        /// Makes the cursor invisible.
        Disable,
    } => |this, w| {
        match this {
            TextCursor::Enable => SetMode(DecMode::Cursor).escape(w),
            TextCursor::Disable => ResetMode(DecMode::Cursor).escape(w),
        }
    }
);

pub type DECTCEM = TextCursor;