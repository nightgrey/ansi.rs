use crate::parser::{Handler, Intermediates, Params, Paras, Table};

use super::{Action, State};
use arrayvec::ArrayVec;
use smallvec::SmallVec;
use utils::SmallByteString;

const EXIT_ACTIONS: &[Action] = &[];

const ENTRY_ACTIONS: &[Action] = &[];

#[derive(Debug, Default)]
pub struct Engine {
    pub state: State,

    pub params: ParamsBuilder,
    pub bytes: ArrayVec<char, 1024>,
    pub intermediates: ArrayVec<u8, 16>,

    pub data: ArrayVec<u8, 1024>,
    pub utf8: ArrayVec<u8, 4>,
    pub xterm: ArrayVec<u8, 3>,
}

impl Engine {
    pub fn advance(&mut self, handler: &mut dyn Handler, chars: impl AsRef<[u8]>) {
        chars.as_ref().iter().for_each(|&char| {
            match self.state {
                State::Utf8 => self.advance_utf8(handler, char),
                _ => self.state(handler, char),
            };
        });
    }

    fn advance_utf8(&mut self, handler: &mut dyn Handler, char: u8) {
        self.collect_utf8(char);

        let expected_len = match self.utf8[0] as u8 {
            0x00..=0x7F => Some(1),
            0xC0..=0xDF => Some(2),
            0xE0..=0xEF => Some(3),
            0xF0..=0xF7 => Some(4),
            _ => None,
        }
        .expect("invalid UTF-8 start byte in Utf8 state");

        if self.utf8.len() < expected_len {
            return;
        }

        // Decode and print the rune
        if let Some(ch) = self.utf8() {
            handler.utf8(ch);
        }

        self.state = State::Ground;
        self.utf8.clear();
    }

    fn advance_xterm(&mut self, handler: &mut dyn Handler, char: u8) -> Action {
        self.collect_xterm(char);

        if !self.xterm.is_full() {
            return Action::Collect;
        }

        self.intermediates.clear();
        self.params.push_param(self.xterm[0] as char);
        self.params.finish_param();
        self.params.push_param(self.xterm[1] as char);
        self.params.finish_param();
        self.params.push_param(self.xterm[2] as char);
        self.params.finish_param();
        // Dispatch with 'M' as the command byte and xterm as params
        handler.handle_csi(self.params.as_slice(), self.intermediates.as_slice(), 'M');

        self.clear();
        self.state = State::Ground;
        Action::Dispatch
    }

    fn state(&mut self, handler: &mut dyn Handler, char: u8) {
        let (next_state, action) = Table::global_transition(self.state, char);
        let prev_state = self.state;

        println!("[char] {:?} [0x{:02x}]", char as char, char);
        println!("[State] {:?}", next_state);

        if (next_state != State::Ground) {
            let exit_action = Table::global().exit(prev_state, char);
            let entry_action = Table::global().entry(next_state, char);

            if (exit_action != Action::None) {
                if exit_action != Action::None {
                    println!("[Exit] {:?}", exit_action);
                }
                self.action(handler, exit_action, char);
            }

            if (action != Action::None) {
                println!("[Action] {:?}", action);
                self.action(handler, action, char);
            }

            if (entry_action != Action::None) {
                if entry_action != Action::None {
                    println!("[Entry] {:?}", entry_action);
                }
                self.action(handler, entry_action, char);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, char);
        }
        println!("---");
    }

    fn action(&mut self, handler: &mut dyn Handler, action: Action, char: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Prefix => {}
            Action::Print => {
                handler.utf8(char as char);
            }

            Action::Clear => {
                self.clear();
            }

            Action::Print => {
                handler.utf8(char as char);
            }

            Action::Execute => {}

            Action::Prefix => {
                self.intermediates.push(char);
            }

            Action::Collect => {
                match self.state {
                    State::Utf8 => {
                        // Reset UTF-8 counter and start collecting
                        self.reset_utf8();
                        self.collect_utf8(char);
                    }
                    _ => {
                        // Collect intermediate bytes
                        self.intermediates.push(char);
                    }
                }
            }

            Action::Param => match char {
                b'0'..=b'9' => {
                    self.params.push_byte(char as u8);
                }
                b':' => {
                    self.params.finish_param();
                }
                b';' => {
                    self.params.finish_group();
                }
                _ => {}
            },
            Action::Record => {}
            Action::OscStart | Action::DcsStart => {}
            Action::OcsEnd | Action::DcsEnd => {}

            Action::Dispatch => {
                if self.params.has_unfinished() {
                    self.params.finish_param();
                }

                self.bytes.clear();

                match self.state {
                    State::CsiEntry | State::CsiParam | State::CsiIntermediate => {
                        handler.handle_csi(
                            self.params.as_slice(),
                            &self.intermediates,
                            char as char,
                        );
                    }
                    State::Escape | State::EscapeIntermediate => {
                        handler.handle_esc(&self.intermediates, char as u8);
                    }
                    State::DcsEntry | State::DcsParam | State::DcsIntermediate | State::DcsData => {
                        handler.handle_dcs(
                            self.params.as_slice(),
                            &self.intermediates,
                            char as char,
                            &self.data,
                        );
                    }
                    State::Data => {
                        handler.handle_apc(&self.data);
                    }
                    _ => (),
                }
            }
        }
    }

    fn collect_xterm(&mut self, char: u8) {
        self.params.push_byte(char);
        self.params.finish_param();
    }

    fn collect_utf8(&mut self, char: u8) {
        if !self.utf8.is_full() {
            self.utf8.push(char);
        }
    }

    fn reset_utf8(&mut self) {
        self.utf8.clear();
    }

    /// Clear engine parameters and command
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.data.clear();

        self.utf8.clear();
        self.xterm.clear();
    }

    pub fn utf8(&self) -> Option<char> {
        str::from_utf8(&self.utf8[..self.utf8.len()])
            .and_then(|s| Ok(s.chars().next()))
            .ok()
            .flatten()
    }
    /// Determine the number of bytes in a UTF-8 Handled from the first byte
    fn utf8_len(byte: u8) -> Option<usize> {
        match byte {
            0x00..=0x7F => Some(1),
            0xC0..=0xDF => Some(2),
            0xE0..=0xEF => Some(3),
            0xF0..=0xF7 => Some(4),
            _ => None,
        }
    }
}

#[derive(Default, Debug)]
pub struct ParamsBuilder<const N: usize = 16> {
    inner: Params<N>,
    current_group: Option<SmallVec<u8, 4>>,
    current_param: Option<u8>,
}

impl<const N: usize> ParamsBuilder<N> {
    /// Check if there's a pending parameter that needs to be finished
    pub fn has_unfinished(&self) -> bool {
        // Has pending if current is not None, or if values exist beyond last boundary
        self.current_group.is_some() || self.current_param.is_some()
    }

    pub fn current_param(&mut self) -> &mut u8 {
        self.current_param.get_or_insert(0)
    }

    pub fn current_group(&mut self) -> &mut SmallVec<u8, 4> {
        self.current_group.get_or_insert_default()
    }

    pub fn push_param(&mut self, char: char) {
        self.current_param.replace(char as u8);
    }

    pub fn push_byte(&mut self, digit: u8) {
        self.push_param(
            self.current_param
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add((digit - b'0') as u8) as char,
        );
    }

    pub fn finish_group(&mut self) {
        self.current_group
            .get_or_insert_default()
            .push(self.current_param.take().unwrap_or(0));
    }

    pub fn finish_param(&mut self) {
        self.finish_group();
        self.inner.extend(self.current_group.take().unwrap());
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.current_param = None;
        self.current_group = None;
    }

    pub fn as_slice(&self) -> Paras<'_> {
        self.inner.as_slice()
    }
}

impl<'a, const N: usize> From<&'a ParamsBuilder<N>> for Paras<'a> {
    fn from(value: &'a ParamsBuilder<N>) -> Self {
        value.inner.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bilge::prelude::Int;
    use derive_more::{Deref, DerefMut};
    use std::collections::VecDeque;

    type Params = crate::parser::Params<16>;
    type Intermediates = crate::parser::Intermediates<16>;
    type Data = utils::SmallByteString<1024>;

    #[derive(Debug, Clone)]
    enum Value {
        Utf8(char),
        Control(u8),
        Csi(Params, Intermediates, char),
        Esc(Intermediates, u8),
        Dcs(Params, Intermediates, char, Data),
        Osc(Params, Intermediates, Data),
        Sos(Data),
        Pm(Data),
        Apc(Data),
    }

    #[derive(Default, Debug, DerefMut, Deref)]
    struct Handler(VecDeque<Value>);

    impl Handler {
        fn new() -> Self {
            Self(VecDeque::new())
        }

        fn pop(&mut self) -> Option<Value> {
            self.0.pop_front()
        }
    }

    impl super::Handler for Handler {
        fn utf8(&mut self, ch: char) {
            self.0.push_back(Value::Utf8(ch));
        }

        fn control(&mut self, byte: u8) {
            self.0.push_back(Value::Control(byte));
        }

        fn handle_csi(&mut self, params: Paras, intermediates: &[u8], final_char: char) {
            self.0.push_back(Value::Csi(
                params.to_params(),
                Intermediates::from(intermediates),
                final_char,
            ));
        }
        fn handle_esc(&mut self, intermediates: &[u8], final_byte: u8) {
            self.push_back(Value::Esc(Intermediates::from(intermediates), final_byte));
        }

        fn handle_dcs(
            &mut self,
            params: Paras,
            intermediates: &[u8],
            final_char: char,
            data: &[u8],
        ) {
            self.0.push_back(Value::Dcs(
                params.to_params(),
                Intermediates::from(intermediates),
                final_char,
                Data::from(data),
            ));
        }

        fn handle_osc(&mut self, params: Paras, intermediates: &[u8], data: &[u8]) {
            self.0.push_back(Value::Osc(
                params.to_params(),
                Intermediates::from(intermediates),
                Data::from(data),
            ));
        }

        fn handle_sos(&mut self, data: &[u8]) {
            self.0.push_back(Value::Sos(Data::from(data)));
        }

        fn handle_pm(&mut self, data: &[u8]) {
            self.0.push_back(Value::Pm(Data::from(data)));
        }

        fn handle_apc(&mut self, data: &[u8]) {
            self.0.push_back(Value::Apc(Data::from(data)));
        }
    }

    impl Iterator for Handler {
        type Item = Value;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }

    struct Engine {
        handler: Handler,
        engine: super::Engine,
    }

    impl Engine {
        fn default() -> Self {
            Self {
                handler: Handler::new(),
                engine: super::Engine::default(),
            }
        }

        fn advance(&mut self, chars: impl AsRef<[u8]>) -> Option<Value> {
            self.engine.advance(&mut self.handler, chars);

            self.handler.pop()
        }
    }

    #[test]
    fn test_csi() {
        let mut engine = Engine::default();

        dbg!(engine.advance("\x1B[1;2:3:4;5m"));
    }

    #[test]
    fn test_osc() {
        let mut engine = Engine::default();

        let osc = "\x1B]>52;hello\x07";
        let osc_extra = "\x1B]>52;hello;;ohooo;100\x07";
        dbg!(engine.advance(osc));
    }
}
