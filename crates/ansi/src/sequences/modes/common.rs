//! Common modes
//!
//! Provides convenient access to enable and disable common modes.
//!
//! For all modes and sequences beyond [`SM`] and [`RM`], check out [`super::modes`].
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