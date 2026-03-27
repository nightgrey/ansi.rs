//! Common modes
//!
//! Provides shortcuts to set ([`SetMode`]) and reset ([`ResetMode`]) common modes.
//!
//! For anything beyond, check out [`sequences`].
use crate::Escape;
use crate::io::Write;
use crate::sequences::modes::*;
use crate::{ResetMode, SetMode};

sequence!(
    /// (25) Text Cursor Enable Mode (DECTCEM) is a mode that shows/hides the cursor.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECTCEM.html
    pub enum TextCursorEnable {
        /// Makes the cursor visible.
        Set,
        /// Makes the cursor invisible.
        Reset,
    } => |this, w| {
        match this {
            TextCursorEnable::Set => SetMode(DecMode::TextCursorEnable).escape(w),
            TextCursorEnable::Reset => ResetMode(DecMode::TextCursorEnable).escape(w),
        }
    }
);

pub type DECTCEM = TextCursorEnable;

sequence!(
    /// (1047) Alternate Screen Mode is a mode that determines whether the alternate screen
    /// buffer is active. When this mode is enabled, the alternate screen buffer is
    /// cleared.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    pub enum AlternateScreen {
        Set,
        Reset,
    } => |this, w| {
        match this {
            AlternateScreen::Set => w.escape(SetMode(DecMode::AlternateScreen)),
            AlternateScreen::Reset => w.escape(ResetMode(DecMode::AlternateScreen)),
        }
    }
);

sequence!(
    /// (2004) Bracketed Paste Mode is a mode that determines whether pasted text is
    /// bracketed with escape sequences.
    ///
    /// See:
    /// - https://cirw.in/blog/bracketed-paste
    /// - https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Bracketed-Paste-Mode
    pub enum BracketedPaste {
        Set,
        Reset,
    } => |this, w| {
        match this {
            BracketedPaste::Set => SetMode(DecMode::BracketedPaste).escape(w),
            BracketedPaste::Reset => ResetMode(DecMode::BracketedPaste).escape(w),
        }
    }
);

sequence!(
    /// (2026) Synchronized Output Mode
    ///
    /// See:
    /// - https://contour-terminal.org/vt-extensions/synchronized-output/
    /// - https://github.com/contour-terminal/vt-extensions/blob/master/synchronized-output.md
    pub enum SynchronizedOutput {
        Set,
        Reset,
    } => |this, w| {
        match this {
            SynchronizedOutput::Set => SetMode(DecMode::SynchronizedOutput).escape(w),
            SynchronizedOutput::Reset => ResetMode(DecMode::SynchronizedOutput).escape(w),
        }
    }
);
