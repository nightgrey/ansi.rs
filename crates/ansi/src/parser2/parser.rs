use arrayvec::ArrayVec;
use derive_more::{Deref, DerefMut};
use log::{debug, log};
use super::*;

pub trait Handler {
    /// Draw a character to the screen and update states.
    fn print(&mut self, byte: char) {}

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, byte: u8) {}

    /// The final character of an escape sequence has arrived.
    ///
    /// The `ignore` flag indicates that more than two intermediates arrived and
    /// subsequent characters were ignored.
    fn esc(&mut self, intermediates: &Inter, final_byte: u8) {}

    /// A final character has arrived for a CSI sequence
    ///
    /// The `ignore` flag indicates that either more than two intermediates
    /// arrived or the number of parameters exceeded the maximum supported
    /// length, and subsequent characters were ignored.
    fn csi(
        &mut self,
        params: Params,
        intermediates: &Inter,
        final_byte: char,
    ) {
    }

    /// Invoked when a final character arrives in first part of device control
    /// string.
    ///
    /// The control function should be determined from the private marker, final
    /// character, and execute with a parameter list. A handler should be
    /// selected for remaining characters in the string; the handler
    /// function should subsequently be called by `put` for every character in
    /// the control string.
    ///
    /// The `ignore` flag indicates that more than two intermediates arrived and
    /// subsequent characters were ignored.
    fn dcs(&mut self, params: Params, intermediates: &Inter, final_char: char) {}

    /// Pass bytes as part of a device control string to the handle chosen in
    /// `hook`. C0 controls will also be passed to the handler.
    fn dcs_byte(&mut self, byte: u8) {}

    /// Called when a device control string is terminated.
    ///
    /// The previously selected handler should be notified that the DCS has
    /// terminated.
    fn dcs_termination(&mut self, byte: u8) {}

    /// Dispatch an operating system command.
    fn osc(&mut self, params: Params) {}

}

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: ParametersBuilder,
    pub intermediates: Intermediates,

    pub data: DataString,
    pub utf8: ArrayVec<u8, 4>,
}

impl Parser {
    pub fn advance(&mut self, handler: &mut impl Handler, bytes: impl AsRef<[u8]>) {
        let mut i = 0;
        let bytes = bytes.as_ref();

        // Handle partial codepoints from previous calls to `advance`.
        if !self.utf8.is_empty() {
            i += self.advance_utf8(handler, bytes);
        }

        while i != bytes.len() {
            match self.state {
                State::Ground => i+=self.advance_ground(handler, &bytes[i..]),
                _ => {
                    let byte = bytes[i];
                    self.advance_byte(handler, byte);
                    i += 1;
                },
            }
        }
    }

    fn advance_utf8(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        // Try to copy up to 3 more characters, to ensure the codepoint is complete.
        let old_bytes = self.utf8.len();
        let to_copy = bytes.len().min(self.utf8.len() - old_bytes);

        self.utf8[old_bytes..old_bytes + to_copy].copy_from_slice(&bytes[..to_copy]);

        // Parse the unicode character.
        match str::from_utf8(&self.utf8) {
            // If the entire buffer is valid, use the first character and continue parsing.
            Ok(parsed) => {
                let c = unsafe { parsed.chars().next().unwrap_unchecked() };
                handler.print(c);

                self.utf8.clear();
                c.len_utf8() - old_bytes
            },
            Err(err) => {
                let valid_bytes = err.valid_up_to();
                // If we have any valid bytes, that means we partially copied another
                // utf8 character into `utf8`. Since we only care about the
                // first character, we just ignore the rest.
                if valid_bytes > 0 {
                    let c = unsafe {
                        let parsed = str::from_utf8_unchecked(&self.utf8[..valid_bytes]);
                        parsed.chars().next().unwrap_unchecked()
                    };

                    handler.print(c);

                    self.utf8.clear();
                    return valid_bytes - old_bytes;
                }

                match err.error_len() {
                    // If the partial character was also invalid, emit the replacement
                    // character.
                    Some(invalid_len) => {
                        handler.print('�');

                        self.utf8.clear();
                        invalid_len - old_bytes
                    },
                    // If the character still isn't complete, wait for more data.
                    None => to_copy,
                }
            },
        }
    }
    fn advance_ground(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        #[inline]
        fn advance_ground_dispatch(handler: &mut impl Handler, parsed: &str) {
            for c in parsed.chars() {
                match c {
                    '\x00'..='\x1f' | '\u{80}'..='\u{9f}' => handler.execute(c as u8),
                    _ => handler.print(c),
                }
            }
        }


        // Find the next escape character.
        let num_bytes = bytes.len();
        let plain_chars = memchr::memchr(0x1B, bytes).unwrap_or(num_bytes);

        // If the next character is ESC, just process it and short-circuit.
        if plain_chars == 0 {
            self.state = State::Escape;
            self.clear();
            return 1;
        }

        match str::from_utf8(&bytes[..plain_chars]) {
            Ok(parsed) => {
                advance_ground_dispatch(handler, parsed);

                let mut processed = plain_chars;

                // If there's another character, it must be escape so process it directly.
                if processed < num_bytes {
                    self.state = State::Escape;
                    self.clear();
                    processed += 1;
                }

                processed
            },
            // Handle invalid and partial utf8.
            Err(err) => {
                // Dispatch all the valid bytes.
                let valid_bytes = err.valid_up_to();
                let parsed = unsafe { str::from_utf8_unchecked(&bytes[..valid_bytes]) };
                advance_ground_dispatch(handler, parsed);

                match err.error_len() {
                    Some(len) => {
                        // Execute C1 escapes or emit replacement character.
                        if len == 1 && bytes[valid_bytes] <= 0x9F {
                            handler.execute(bytes[valid_bytes]);
                        } else {
                            handler.print('�');
                        }

                        // Restart processing after the invalid bytes.
                        //
                        // While we could theoretically try to just re-parse
                        // `bytes[valid_bytes + len..plain_chars]`, it's easier
                        // to just skip it and invalid utf8 is pretty rare anyway.
                        valid_bytes + len
                    },
                    None => {
                        if plain_chars < num_bytes {
                            // Process bytes cut off by escape.
                            handler.print('�');
                            self.state = State::Escape;
                            self.clear();
                            plain_chars + 1
                        } else {
                            let len = self.utf8.len();
                            // Process bytes cut off by the buffer end.
                            let extra_bytes = num_bytes - valid_bytes;
                            let partial_len = len + extra_bytes;
                            self.utf8[len..partial_len]
                                .copy_from_slice(&bytes[valid_bytes..valid_bytes + extra_bytes]);

                            num_bytes
                        }
                    },
                }
            },
        }
    }
    #[inline]
    fn advance_byte(&mut self, handler: &mut impl Handler, byte: u8) {
        let prev_state = self.state;
        let (action, next_state) = transition(self.state, byte);
        println!("{:?} / 0x{:2x} | {:?} -> {:?} @ {:?}", byte as char, byte, prev_state, if next_state == State::None { prev_state } else { next_state }, action);

        if next_state != State::None {
            let exit_action = exit(prev_state);
            let entry_action = entry(next_state);

            if exit_action != Action::None {
                self.action(handler, exit_action, byte);
            }

            if action != Action::None {
                self.action(handler, action, byte);
            }

            if entry_action != Action::None {
                self.action(handler, entry_action, byte);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }

    #[inline]
    fn action(&mut self, handler: &mut impl Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Clear => {
                self.clear();
            }

            Action::Print => {
                handler.print(byte as char);
            }

            Action::Execute => {
                handler.execute(byte);
            }

            Action::Collect => {
            self.intermediates.push(byte);
            }

            Action::Param => match byte {
                b'0'..=b'9' => self.params.push_digit(byte),
                b':' => self.params.advance_sub(),
                b';' => self.params.advance_param(),
                _ => {}
            },

            Action::EscDispatch => handler.esc(self.intermediates.as_ref(), byte),
            Action::CsiDispatch => {
            self.params.advance_param();
                handler.csi(
                    self.params.as_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                )
            },

            Action::DcsDispatch => handler.dcs(
                self.params.as_slice(),
                self.intermediates.as_ref(),
                byte as char,
            ),
            Action::DcsByte => handler.dcs_byte(
                byte
            ),
            Action::DcsTermination => handler.dcs_termination(byte),

            Action::OscDispatch => handler.osc(self.params.as_slice()),

            Action::OscByte =>  {

            }
            Action::OscTermination => {

            }
            _ => {}
        }
    }

    fn push_utf8(&mut self, handler: &mut impl Handler, byte: u8) {
        if !self.utf8.is_full() {
            self.utf8.push(byte);
        }

        let expected = match self.utf8[0] {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ =>  {
                self.utf8.clear();
                self.state = State::Ground;
                return;
            }
        };

        if self.utf8.len() >= expected {
            if let Ok(s) = str::from_utf8(&self.utf8) {
                if let Some(ch) = s.chars().next() {
                    handler.print(ch);
                }
            }
            self.utf8.clear();
            self.state = State::Ground;
        }
    }

    /// Reset parameter / intermediate / data buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.data.clear();
        self.utf8.clear();
    }
}

#[derive(Deref, DerefMut, Debug, Default, Clone)]
struct ParametersBuilder {
    #[deref]
    #[deref_mut]
    pub inner: super::Parameters,
    pub param: Option<u16>,
}

impl ParametersBuilder {
    /// Accumulate an ASCII digit into the current sub-parameter value.
    pub fn push_digit(&mut self, digit: u8) {
        self.param.replace(self.param.unwrap_or(0)
            .saturating_mul(10)
            .saturating_add((digit - b'0') as u16)); // ECMA-48 allows parameters up to 16383 — clamp to that
    }

    /// Close the current sub-parameter (`:` separator).
    pub fn advance_sub(&mut self) {
        if self.param.is_none() {
            return;
        }

        self.inner.extend(self.param.take());
    }

    /// Close the current main parameter (`;` separator, or end of sequence).
    pub fn advance_param(&mut self) {
        self.advance_sub();
        self.inner.separate();
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.param = None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Value {
        Print(char),
        Execute(u8),
        Esc(Intermediates, u8),
        Csi(Parameters, Intermediates, char),
        Dcs(Parameters, Intermediates, char),
        DcsByte(u8),
        DcsTermination(u8),
        Osc(Parameters),

    }

    #[derive(Debug, Default, DerefMut, Deref)]
    struct Recorder {
        pub values: Vec<Value>,
    }

    impl Handler for Recorder {
        fn print(&mut self, ch: char) {
            self.values.push(Value::Print(ch));
        }
        fn execute(&mut self, byte: u8) {
            self.values.push(Value::Execute(byte));
        }
        fn esc(&mut self, intermediates: &Inter, final_byte: u8) {
            self.values.push(Value::Esc(Intermediates::from(intermediates), final_byte));
        }
        fn csi(&mut self, params: Params, intermediates: &Inter, final_byte: char) {
            self.values.push(Value::Csi(params.to_vec(), intermediates.to_owned(), final_byte));
        }
        fn dcs(&mut self, params: Params, intermediates: &Inter, final_char: char) {
            self.values.push(Value::Dcs(params.to_vec(), intermediates.to_owned(), final_char));
        }
        fn dcs_byte(&mut self, byte: u8) {
            self.values.push(Value::DcsByte(byte));
        }
        fn dcs_termination(&mut self, byte: u8) {
            self.values.push(Value::DcsTermination(byte));
        }
        fn osc(&mut self, params: Params) {
            self.values.push(Value::Osc(params.to_vec()));
        }
    }
    #[derive(Debug, Default, DerefMut, Deref)]
    struct Parser {
        inner: super::Parser,
        #[deref_mut]
        #[deref]
        recorder: Recorder,
    }

    impl Parser {
        pub fn advance(&mut self, bytes: impl AsRef<[u8]>) {
            self.inner.advance(&mut self.recorder, bytes);
        }

        pub fn test(&mut self, bytes: impl AsRef<[u8]>) -> Vec<Value> {
            self.recorder.values.clear();
            self.inner.advance(&mut self.recorder, bytes);
            let values = self.recorder.values.clone();
            self.recorder.values.clear();
            values
        }
    }

    #[test]
    fn test_transition() {
        println!("{:?}", transition(State::CsiParam, b';'));
    }

    #[test]
    fn test_parameters() {
     let mut parser = Parser::default();

    dbg!(parser.test("\x1b[1;2;3m👨🏿\x1b[0m"));
    }
}