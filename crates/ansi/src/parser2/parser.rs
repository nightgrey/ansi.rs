use arrayvec::ArrayVec;
use derive_more::{Deref, DerefMut};
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
    fn esc(&mut self, intermediates: &[u8], final_byte: u8) {}

    /// A final character has arrived for a CSI sequence
    ///
    /// The `ignore` flag indicates that either more than two intermediates
    /// arrived or the number of parameters exceeded the maximum supported
    /// length, and subsequent characters were ignored.
    fn csi(
        &mut self,
        params: Params,
        intermediates: &[u8],
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
    fn dcs(&mut self, params: Params, intermediates: &[u8], final_char: char) {}

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
    pub fn advance(&mut self, handler: &mut dyn Handler, bytes: impl AsRef<[u8]>) {
        for &byte in bytes.as_ref() {
            match self.state {
                // State::Utf8 => self.advance_utf8(handler, byte),
                _ => self.advance_byte(handler, byte),
            }
        }
    }

    pub fn transition(state: State, byte: u8) -> (Action, State) {
        let change = TRANSITIONS[state as usize][byte as usize];
        dbg!(unpack(change));

         unpack(change)
    }

    pub fn entry(state: State) -> Action{
       ENTRY_ACTIONS[state as usize]
    }

    pub fn exit(state: State) -> Action{
        EXIT_ACTIONS[state as usize]
    }

    #[inline]
    fn advance_byte(&mut self, handler: &mut dyn Handler, byte: u8) {
        let (action, state) = Self::transition(self.state, byte);
        if state != self.state {
            let exit_action = Self::exit(self.state);
            let entry_action = Self::entry(state);

            println!("{:?} -> {:?} @ {:?}", self.state, state, action);
            if exit_action != Action::None {
                self.action(handler, exit_action, byte);
            }

            if action != Action::None {
                self.action(handler, action, byte);
            }

            if entry_action != Action::None {
                self.action(handler, entry_action, byte);
            }

            self.state = state;
        } else {
            self.action(handler, action, byte);
        }
    }

    #[inline]
    fn action(&mut self, handler: &mut dyn Handler, action: Action, byte: u8) {
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
                    self.params.borrow(),
                    self.intermediates.as_ref(),
                    byte as char,
                )
            },

            Action::DcsDispatch => handler.dcs(
                self.params.borrow(),
                self.intermediates.as_ref(),
                byte as char,
            ),
            Action::DcsByte => handler.dcs_byte(
                byte
            ),
            Action::DcsTermination => handler.dcs_termination(byte),

            Action::OscDispatch => handler.osc(self.params.borrow()),

            Action::OscByte =>  {

            }
            Action::OscTermination => {

            }
            _ => {}
        }
    }

    fn push_utf8(&mut self, handler: &mut dyn Handler, byte: u8) {
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
        fn esc(&mut self, intermediates: &[u8], final_byte: u8) {
            self.values.push(Value::Esc(Intermediates::from(intermediates), final_byte));
        }
        fn csi(&mut self, params: Params, intermediates: &[u8], final_byte: char) {
            self.values.push(Value::Csi(params.to_owned(), Intermediates::from(intermediates), final_byte));
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
    }

    #[test]
    fn test_parameters() {
     let mut parser = Parser::default();

        parser.advance(b"\x1B[3;4mxxxxxxxxxx");
        dbg!(parser.iter().collect::<Vec<_>>());
    }
}