use derive_more::{Deref, DerefMut};
use maybe::Maybe;
use utils::Nested;
use crate::parser::collectors::Parameters;
use crate::parser::conditions::{is_end_of_csi, is_end_of_ground};
use super::{Action, ByteStr, ByteString, Handler, Params, State};
pub trait BaseHandler {
    /// Draw a character to the screen and update states.
    fn text(&mut self, bytes: &[u8]) {}

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, byte: u8) {}

    /// The final character of an escape sequence has arrived.
    fn esc(&mut self, intermediates: &ByteStr, final_byte: u8) {}

    /// A final character has arrived for a CSI sequence.
    fn csi(&mut self, params: Params<'_>, intermediates: &ByteStr, final_byte: char) {}

    /// Invoked when a final character arrives in first part of device control
    /// string. Subsequent bytes in the control string are delivered via
    /// [`crate::parser::Handler::dcs_byte`], and termination via [`crate::parser::Handler::dcs_end`].
    fn dcs_start(&mut self, params: Params<'_>, intermediates: &ByteStr, final_char: char) {}

    /// A byte of a DCS data string. C0 controls are also passed here.
    fn dcs_byte(&mut self, byte: u8) {}


    /// The DCS data string has been terminated.
    fn dcs_end(&mut self, byte: u8) {}

    /// Begin an operating system command. Subsequent body bytes are delivered
    /// via [`crate::parser::Handler::osc_byte`]; termination via [`crate::parser::Handler::osc_end`].
    fn osc_start(&mut self) {}

    /// A byte of OSC data.
    fn osc_byte(&mut self, byte: u8) {}

    /// The OSC string has been terminated.
    fn osc_end(&mut self, byte: u8) {}
}

#[derive(Debug, Default)]
pub struct BaseParser {
    pub state: State,

    pub params: Parameters,
    pub intermediates: ByteString,
}

impl BaseParser {
    pub fn advance(&mut self, handler: &mut impl BaseHandler, bytes: &[u8]) -> usize {
        let mut i = 0;

        while i != bytes.len() {
            i += match self.state {
                State::Ground => self.advance_ground(handler, &bytes[i..]),
                State::CsiIgnore => self.advance_csi_ignore(handler, &bytes[i..]),
                _ => self.advance_rest(handler, bytes[i]),
            };
        }

        i
    }

    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
    }

    pub fn flush(&mut self) {
        self.state = State::Ground;
        self.clear();
    }

    #[inline]
    pub(super) fn advance_ground(&mut self, handler: &mut impl BaseHandler, bytes: &[u8]) -> usize {
        // Find next control character (not just ESC)
        let end = bytes.iter()
            .position(|&b| is_end_of_ground(b))
            .unwrap_or(bytes.len());

        if end > 0 {
            handler.text(&bytes[..end]);
        }

        if end < bytes.len() {
            let byte = bytes[end];
            if byte == 0x1B {
                self.state = State::Escape;
                self.clear();
            } else {
                handler.execute(byte);
            }
            end + 1
        } else {
            end
        }
    }

    #[inline]
    pub(super) fn advance_csi_ignore(&mut self, handler: &mut impl BaseHandler, bytes: &[u8]) -> usize {
        // Find next control character (not just ESC)
        let end = bytes.iter()
            .position(|&b| is_end_of_csi(b))
            .unwrap_or(bytes.len());

        if end > 0 {
            handler.text(&bytes[..end]);
        }

        end
    }

    #[inline]
    pub(super) fn advance_rest(&mut self, handler: &mut impl BaseHandler, byte: u8) -> usize {
        self.transition(handler, byte);
        1
    }

    #[inline]
    pub(super) fn transition(&mut self, handler: &mut impl BaseHandler, byte: u8) {
        let prev_state = self.state;
        let (action, next_state) = self.state.transition(byte);

        if next_state != State::None {
            let exit = prev_state.exit();
            if exit.is_some() {
                self.action(handler, exit, byte);
            }

            self.action(handler, action, byte);

            let entry = next_state.entry();
            if entry.is_some() {
                self.action(handler, entry, byte);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }

    fn action(&mut self, handler: &mut impl BaseHandler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Clear => self.clear(),

            Action::Print => handler.text(&[byte]),
            Action::Execute => handler.execute(byte),

            Action::Collect => self.intermediates.push(byte),

            Action::Param => match byte {
                b'0'..=b'9' => self.params.push_digit(byte),
                b':' => self.params.push_sub(),
                b';' => self.params.push_main(),
                _ => {}
            },

            Action::EscDispatch => handler.esc(self.intermediates.as_ref(), byte),
            Action::CsiDispatch => {
                self.params.finish();
                handler.csi(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }

            Action::DcsStart => {
                self.params.finish();
                handler.dcs_start(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }
            Action::DcsPut => handler.dcs_byte(byte),
            Action::DcsEnd => handler.dcs_end(byte),

            Action::OscStart => handler.osc_start(),
            Action::OscPut => handler.osc_byte(byte),
            Action::OscEnd => handler.osc_end(byte),
        }
    }

}

/// A wrapper around [`BaseParser`] that handles UTF-8 codepoints.
#[derive(Debug, Default)]
pub struct UnicodeParser {
    inner: BaseParser,
    utf8: [u8; 4],
    utf8_len: usize,
}

impl UnicodeParser {
    pub fn advance(&mut self, handler: &mut impl UnicodeHandler, bytes: &[u8]) -> usize {
        let mut i = 0;

        // Handle partial codepoints from previous calls.
        if self.utf8_len > 0 {
            i += self.advance_utf8(handler, bytes);
        }

        while i != bytes.len() {
            i += match self.inner.state {
                State::Ground => self.advance_ground(handler, &bytes[i..]),
                State::CsiIgnore => self.inner.advance_csi_ignore(handler, &bytes[i..]),
                _ => self.inner.advance_rest(handler, bytes[i]),
            };
        }
        i
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.utf8_len = 0;
    }

    pub fn flush(&mut self) {
        self.inner.state = State::Ground;
        self.clear();
    }

    /// Advance the parser while processing a partial utf8 codepoint.
    #[inline]
    fn advance_utf8(&mut self, handler: &mut impl UnicodeHandler, bytes: &[u8]) -> usize {
        // Try to copy up to 3 more characters, to ensure the codepoint is complete.
        let old_bytes = self.utf8_len;
        let to_copy = bytes.len().min(self.utf8.len() - old_bytes);
        self.utf8[old_bytes..old_bytes + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.utf8_len += to_copy;

        // Parse the unicode character.
        match str::from_utf8(&self.utf8[..self.utf8_len]) {
            // If the entire buffer is valid, use the first character and continue parsing.
            Ok(parsed) => {
                let c = unsafe { parsed.chars().next().unwrap_unchecked() };
                handler.print(c);

                self.utf8_len = 0;
                c.len_utf8() - old_bytes
            },
            Err(err) => {
                // If we have any valid bytes, that means we partially copied another
                // utf8 character into `partial_utf8`. Since we only care about the
                // first character, we just ignore the rest.

                let valid_len = err.valid_up_to();

                if valid_len > 0 {
                    let c = unsafe {
                        let parsed = str::from_utf8_unchecked(&self.utf8[..valid_len]);
                        parsed.chars().next().unwrap_unchecked()
                    };

                    handler.print(c);

                    self.utf8_len = 0;
                    return valid_len - old_bytes;
                }

                match err.error_len() {
                    // If the partial character was also invalid, emit the replacement
                    // character.
                    Some(invalid_len) => {
                        handler.print('�');

                        self.utf8_len = 0;
                        invalid_len - old_bytes
                    },
                    // If the character still isn't complete, wait for more data.
                    None => to_copy,
                }
            },
        }
    }

    #[inline]
    fn advance_ground(&mut self, handler: &mut impl UnicodeHandler, bytes: &[u8]) -> usize {
        // Find the next escape character.
        let bytes_len = bytes.len();
        let chars_len = memchr::memchr(0x1B, bytes).unwrap_or(bytes_len);

        // If the next character is ESC, just process it and short-circuit.
        if chars_len == 0 {
            self.inner.state = State::Escape;
            self.inner.clear();
            return 1;
        }

        match str::from_utf8(&bytes[..chars_len]) {
            Ok(parsed) => {
                Self::dispatch_ground(handler, parsed);
                let mut parsed_len = chars_len;

                // If there's another character, it must be escape so process it directly.
                if parsed_len < bytes_len {
                    self.inner.state = State::Escape;
                    self.inner.clear();
                    parsed_len += 1;
                }

                parsed_len
            },
            // Handle invalid and partial utf8.
            Err(err) => {
                // Dispatch all the valid bytes.
                let valid_len = err.valid_up_to();
                let parsed = unsafe { str::from_utf8_unchecked(&bytes[..valid_len]) };
                Self::dispatch_ground(handler, parsed);

                match err.error_len() {
                    Some(len) => {
                        // Execute C1 escapes or emit replacement character.
                        if len == 1 && bytes[valid_len] <= 0x9F {
                            handler.execute(bytes[valid_len]);
                        } else {
                            handler.print('�');
                        }

                        // Restart processing after the invalid bytes.
                        //
                        // While we could theoretically try to just re-parse
                        // `bytes[valid_bytes + len..plain_chars]`, it's easier
                        // to just skip it and invalid utf8 is pretty rare anyway.
                        valid_len + len
                    },
                    None => {
                        if chars_len < bytes_len {
                            // Process bytes cut off by escape.
                            handler.print('�');
                            self.inner.state = State::Escape;
                            self.inner.clear();
                            chars_len + 1
                        } else {
                            // Process bytes cut off by the buffer end.
                            let extra_bytes = bytes_len - valid_len;
                            let partial_len = self.utf8_len + extra_bytes;
                            self.utf8[self.utf8_len..partial_len]
                                .copy_from_slice(&bytes[valid_len..valid_len + extra_bytes]);
                            self.utf8_len = partial_len;
                            bytes_len
                        }
                    },
                }
            },
        }
    }

    /// Handle ground dispatch of print/execute for all characters in a string.
    #[inline]
    fn dispatch_ground(handler: &mut impl UnicodeHandler, text: &str) {
        for c in text.chars() {
            match c {
                '\x00'..='\x1f' | '\u{80}'..='\u{9f}' => handler.execute(c as u8),
                _ => handler.print(c),
            }
        }
    }
}

pub trait UnicodeHandler: BaseHandler {
    fn string(&mut self, str: &str) {
        for c in str.chars() {
            self.print(c);
        }
    }
    fn print(&mut self, ch: char) {}
}
