// use super::{Action, State};
// use arrayvec::ArrayVec;
// use compact_str::CompactString;
// use derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator};
// use segmented_vec::{SegmentedSlice, SegmentedVec};

// #[derive(Deref, DerefMut, Index, IndexMut, IntoIterator, AsRef, AsMut, Into, From, Clone)]
// struct Params(SegmentedVec<u8>);

// impl Params {
//     pub fn new() -> Self {
//         Self(SegmentedVec::new())
//     }
// }
// #[test]
// fn qwe() {
//     let mut p = Params::new();

//     p.push(1);
//     p.push(2);
//     p.push(3);
//     p.push(4);
//     for x in p {
//         dbg!(x);
//     }
// }

// #[derive(Debug, Default)]
// pub struct Engine {
//     state: State,

//     params: [u8; 32],
//     bytes: CompactString,
//     intermediates: CompactString,
//     data: CompactString,

//     utf8: ArrayVec<u8, 4>,
//     xterm: ArrayVec<u8, 3>,
// }

// impl Engine {
//     fn params(&self) -> GroupedParams<'_> {
//         GroupedParams::from(&self.params)
//     }

//     fn intermediates(&self) -> &str {
//         &self.intermediates
//     }

//     pub fn data(&self) -> &str {
//         &self.data
//     }

//     pub fn state(&self) -> State {
//         self.state
//     }

//     pub fn utf8(&self) -> Option<char> {
//         str::from_utf8(&self.utf8[..self.utf8.len()])
//             .and_then(|s| Ok(s.chars().next()))
//             .ok()
//             .flatten()
//     }

//     pub fn advance(&mut self, handler: &mut dyn Handler, bytes: impl AsRef<[u8]>) {
//         bytes.as_ref().iter().for_each(|&byte| {
//             match self.state {
//                 State::Utf8 => self.advance_utf8(handler, byte),
//                 // State::XtermParam => self.advance_xterm(handler, byte),
//                 _ => self.advance_byte(handler, byte),
//             };
//         });
//     }

//     fn advance_utf8(&mut self, handler: &mut dyn Handler, byte: u8) -> Action {
//         self.collect_utf8(byte);

//         let expected_len = match self.utf8[0] {
//             0x00..=0x7F => Some(1),
//             0xC0..=0xDF => Some(2),
//             0xE0..=0xEF => Some(3),
//             0xF0..=0xF7 => Some(4),
//             _ => None,
//         }
//         .expect("invalid UTF-8 start byte in Utf8 state");

//         if self.utf8.len() < expected_len {
//             return Action::Collect;
//         }

//         // Decode and print the rune
//         if let Some(ch) = self.utf8() {
//             handler.utf8(ch);
//         }

//         self.state = State::Ground;
//         self.utf8.clear();

//         Action::Print
//     }
//     fn advance_xterm(&mut self, handler: &mut dyn Handler, byte: u8) -> Action {
//         self.collect_xterm(byte);

//         if !self.xterm.is_full() {
//             return Action::Collect;
//         }

//         self.intermediates.clear();
//         self.params.push_byte(self.xterm[0] as u16);
//         self.params.finish_param();
//         self.params.push_byte(self.xterm[1] as u16);
//         self.params.finish_param();
//         self.params.push_byte(self.xterm[2] as u16);
//         self.params.finish_param();
//         // Dispatch with 'M' as the command byte and xterm as params
//         handler.handle_csi(self.params(), self.intermediates(), 'M');

//         self.clear();
//         self.state = State::Ground;
//         Action::Dispatch
//     }
//     fn advance_byte(&mut self, handler: &mut dyn Handler, byte: u8) -> Action {
//         let (next_state, action) = TransitionTable::global().transition(self.state, byte);

//         // We need to clear the parser state if the state changes from EscapeState.
//         // This is because when we enter the EscapeState, we don't get a chance to
//         // clear the parser state. For example, when a sequence terminates with a
//         // ST (\x1b\\ or \x9c), we dispatch the current sequence and transition to
//         // EscapeState. However, the parser state is not cleared in this case and
//         // we need to clear it here before dispatching the esc sequence.
//         if self.state != next_state {
//             if self.state == State::Escape {
//                 self.action(Action::Clear, next_state, handler, byte);
//             }

//             if action == Action::Put
//                 && self.state == State::DcsEntry
//                 && next_state == State::DcsData
//             {
//                 // Special case: non-string parameterized DCS data
//                 self.action(Action::Data, next_state, handler, 0);
//             }
//         }

//         // Handle special cases
//         if byte == std::ascii::Char::Null as u8 && self.state == State::Escape {
//             self.action(Action::Execute, next_state, handler, byte);
//         } else {
//             self.action(action, next_state, handler, byte);
//         }

//         self.state = next_state;
//         action
//     }

//     fn action(&mut self, action: Action, state: State, handler: &mut dyn Handler, byte: u8) {
//         match action {
//             Action::None | Action::Ignore => {}

//             Action::Clear => {
//                 self.clear();
//             }

//             Action::Print => {
//                 handler.utf8(byte as char);
//             }

//             Action::Execute => {
//                 handler.control(byte);
//             }

//             Action::Prefix => {
//                 self.intermediates.push(byte);
//             }

//             Action::Collect => {
//                 match state {
//                     State::Utf8 => {
//                         // Reset UTF-8 counter and start collecting
//                         self.reset_utf8();
//                         self.collect_utf8(byte);
//                     }
//                     _ => {
//                         // Collect intermediate bytes
//                         self.intermediates.push(byte);
//                     }
//                 }
//             }

//             Action::Param => match byte {
//                 b'0'..=b'9' => {
//                     self.params.push_digit(byte);
//                 }
//                 b':' => {
//                     self.params.finish_sub();
//                 }
//                 b';' => {
//                     self.params.finish_param();
//                 }
//                 _ => {}
//             },
//             Action::Data => {
//                 self.data.push(byte);
//             }

//             Action::Put => {
//                 self.data.push(byte);
//             }

//             Action::Dispatch => {
//                 if self.params.is_pending() {
//                     self.params.finish_param();
//                 }

//                 self.bytes.clear();

//                 match self.state {
//                     State::CsiEntry | State::CsiParam | State::CsiIntermediate => {
//                         handler.handle_csi(self.params(), self.intermediates(), byte as char);
//                     }
//                     State::Escape | State::EscapeIntermediate => {
//                         handler.handle_esc(self.intermediates(), byte);
//                     }
//                     State::DcsEntry | State::DcsParam | State::DcsIntermediate | State::DcsData => {
//                         handler.handle_dcs(
//                             self.params(),
//                             self.intermediates(),
//                             byte as char,
//                             self.data(),
//                         );
//                     }
//                     State::OscString => {
//                         handler.handle_osc(self.params(), self.intermediates(), self.data());
//                     }
//                     State::SosString => {
//                         handler.handle_sos(self.data());
//                     }
//                     State::PmString => {
//                         handler.handle_pm(self.data());
//                     }
//                     State::ApcData => {
//                         handler.handle_apc(self.data());
//                     }
//                     _ => (),
//                 }
//             }
//         }
//     }

//     fn collect_xterm(&mut self, byte: u8) {
//         self.params.push_byte(byte as u16);
//         self.params.finish_param();
//     }

//     fn collect_utf8(&mut self, byte: u8) {
//         if !self.utf8.is_full() {
//             self.utf8.push(byte);
//         }
//     }

//     fn reset_utf8(&mut self) {
//         self.utf8.clear();
//     }

//     /// Clear engine parameters and command
//     pub fn clear(&mut self) {
//         self.params.clear();
//         self.intermediates.clear();
//         self.data.clear();

//         self.utf8.clear();
//         self.xterm.clear();
//     }

//     /// Determine the number of bytes in a UTF-8 sequence from the first byte
//     fn utf8_sequence_len(byte: u8) -> Option<usize> {
//         match byte {
//             0x00..=0x7F => Some(1),
//             0xC0..=0xDF => Some(2),
//             0xE0..=0xEF => Some(3),
//             0xF0..=0xF7 => Some(4),
//             _ => None,
//         }
//     }
// }
