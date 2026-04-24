use super::{Action, DataString, FinalChar, Handler, Intermediates, Parameter, Params, State, Table};
use arrayvec::ArrayVec;
use bilge::prelude::Integer;
use smallvec::SmallVec;

#[derive(Debug, Default)]
pub struct Engine {
    pub state: State,

    pub params: ParameterState,
    pub intermediates: Intermediates,

    pub data: DataString,
    pub utf8: ArrayVec<u8, 4>,

    /// Final byte of the DCS prefix — captured on entry to `DcsData` so
    /// `handle_dcs` can be invoked with it when the string terminates.
    dcs_final: u8,
}

impl Engine {
    pub fn advance(&mut self, handler: &mut dyn Handler, chars: impl AsRef<[u8]>) {
        for &byte in chars.as_ref().iter() {
            // Fast path: Ground + printable ASCII (0x20..=0x7E).
            // ESC (0x1B) and DEL (0x7F) are outside this range, so the only
            // transitions we skip are the no-op "stay in Ground" ones.
            if self.state == State::Ground && (0x20..=0x7E).contains(&byte) {
                handler.utf8(byte as char);
                continue;
            }
            self.step(handler, byte);
        }
    }

    #[inline]
    fn step(&mut self, handler: &mut dyn Handler, byte: u8) {
        let prev_state = self.state;
        let (next_state, action) = Table::GLOBAL.transition(prev_state, byte);

        if next_state != prev_state {
            let exit_action = Table::GLOBAL.on_exit(prev_state);
            let entry_action = Table::GLOBAL.on_enter(next_state);

            if exit_action != Action::None {
                self.action(handler, exit_action, byte);
            }
            if action != Action::None {
                self.action(handler, action, byte);
            }

            self.state = next_state;

            // Capture the DCS final byte on entry into the passthrough state.
            if next_state == State::DcsData {
                self.dcs_final = byte;
            }

            if entry_action != Action::None {
                self.action(handler, entry_action, byte);
            }
        } else if action != Action::None {
            self.action(handler, action, byte);
        }
    }

    #[inline]
    fn action(&mut self, handler: &mut dyn Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Print => {
                match self.state {
                    State::Utf8 => {
                        if !self.utf8.is_full() {
                            self.utf8.push(byte);
                        }

                        // utf8[0] is guaranteed by the table to be a lead byte in
                        // 0xC2..=0xF4 — pick the expected length from it.
                        let expected = match self.utf8[0] {
                            0xC2..=0xDF => 2,
                            0xE0..=0xEF => 3,
                            0xF0..=0xF4 => 4,
                            _ => {
                                self.utf8.clear();
                                self.state = State::Ground;
                                return;
                            }
                        };

                        if self.utf8.len() >= expected {
                            if let Ok(s) = str::from_utf8(&self.utf8) {
                                if let Some(ch) = s.chars().next() {
                                    handler.utf8(ch);
                                }
                            }
                            self.utf8.clear();
                        }
                    }
                    _ => {
                        handler.utf8(byte as char);
                    }
                }
            }

            Action::Execute => {
                handler.control(byte);
            }

            Action::Clear => {
                self.clear();
            }

            Action::Collect => {
                self.intermediates.push(byte);
            }

            Action::Param => match byte {
                b'0'..=b'9' => self.params.push_digit(byte),
                // sub-parameter separator — keep accumulating within the group.
                b':' => self.params.push_subparam(),
                // parameter separator — close the whole group.
                b';' => self.params.finish_param(),
                _ => {}
            },

            Action::Record => {
                self.data.push(byte);
            }

            Action::Dispatch => {
                if self.params.has_unfinished() {
                    self.params.finish_param();
                }
                match self.state {
                    State::CsiEntry | State::CsiParam | State::CsiIntermediate => {
                        handler.handle_csi(
                            self.params.borrow(),
                            self.intermediates.as_ref(),
                            byte as char,
                        );
                    }
                    State::DcsData => handler.handle_dcs(
                        self.params.borrow(),
                        self.intermediates.as_ref(),
                        self.dcs_final as char,
                        self.data.as_ref(),
                    ),
                    State::OscData => {
                        handler.handle_osc(self.params.borrow(), self.intermediates.as_ref(), self.data.as_ref())
                    }
                    State::SosData => handler.handle_sos(self.data.as_ref()),
                    State::PmData => handler.handle_pm(self.data.as_ref()),
                    State::ApcData => handler.handle_apc(self.data.as_ref()),
                    State::Escape | State::EscapeIntermediate => {
                        handler.handle_esc(self.intermediates.as_ref(), byte);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Reset parameter / intermediate / data buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.data.clear();
        self.utf8.clear();
        self.dcs_final = 0;
    }
}

/// Incremental builder for a nested [`Parameter`] structure.
///
/// ANSI parameters are two-level: `;` separates main parameters and `:`
/// separates sub-parameters within a main parameter. `1;2:3:4;5` parses to
/// three groups: `[[1], [2, 3, 4], [5]]`.
#[derive(Default, Debug)]
pub struct ParameterState<const N: usize = 16> {
    inner: Parameter<N>,
    /// Sub-parameters accumulated for the current (unfinished) main parameter.
    current_group: SmallVec<u16, 4>,
    /// Digit-accumulating value of the current sub-parameter.
    current_param: Option<u16>,
}

impl<const N: usize> ParameterState<N> {
    /// ECMA-48 allows parameters up to 16383 — clamp to that.
    const MAX: u16 = 16383;

    /// True if there are pending values that haven't been committed to `inner`.
    pub fn has_unfinished(&self) -> bool {
        self.current_param.is_some() || !self.current_group.is_empty()
    }

    /// Accumulate an ASCII digit into the current sub-parameter value.
    pub fn push_digit(&mut self, digit: u8) {
        let cur = self.current_param.unwrap_or(0);
        let new = cur
            .saturating_mul(10)
            .saturating_add((digit - b'0') as u16)
            .min(Self::MAX);
        self.current_param = Some(new);
    }

    /// Close the current sub-parameter (`:` separator).
    pub fn push_subparam(&mut self) {
        self.current_group
            .push(self.current_param.take().unwrap_or(0));
    }

    /// Close the current main parameter (`;` separator, or end of sequence).
    pub fn finish_param(&mut self) {
        self.push_subparam();
        self.inner.push_group(self.current_group.drain(..));
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.current_group.clear();
        self.current_param = None;
    }

    pub fn borrow(&self) -> Params<'_> {
        self.inner.borrow()
    }
}

impl<'a, const N: usize> From<&'a ParameterState<N>> for Params<'a> {
    fn from(value: &'a ParameterState<N>) -> Self {
        value.inner.borrow()
    }
}

#[cfg(test)]
mod tests {
    use crate::params;

    use super::*;
    use derive_more::{Deref, DerefMut};
    use std::collections::VecDeque;
    use crate::parser::{DataStr, Inter};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Value {
        Utf8(char),
        Control(u8),
        Csi(Parameter, Intermediates, FinalChar),
        Esc(Intermediates, u8),
        Dcs(Parameter, Intermediates, FinalChar, DataString),
        Osc(Parameter, Intermediates, DataString),
        Sos(DataString),
        Pm(DataString),
        Apc(DataString),
    }

    #[derive(Default, Debug, DerefMut, Deref)]
    struct Recorder(VecDeque<Value>);

    impl Recorder {
        fn new() -> Self {
            Self(VecDeque::new())
        }

        fn pop(&mut self) -> Option<Value> {
            self.0.pop_front()
        }
    }

    impl Handler for Recorder {
        fn utf8(&mut self, ch: char) {
            self.0.push_back(Value::Utf8(ch));
        }

        fn control(&mut self, byte: u8) {
            self.0.push_back(Value::Control(byte));
        }

        fn handle_csi(&mut self, params: Params, intermediates: &Inter, final_char: FinalChar) {
            self.0.push_back(Value::Csi(
                params.to_owned(),
                Intermediates::from(intermediates),
                final_char,
            ));
        }
        
        fn handle_esc(&mut self, intermediates: &Inter, final_byte: u8) {
            self.push_back(Value::Esc(Intermediates::from(intermediates), final_byte));
        }

        fn handle_dcs(
            &mut self,
            params: Params,
            intermediates: &Inter,
            final_char: FinalChar,
            data: &DataStr,
        ) {
            self.0.push_back(Value::Dcs(
                params.to_owned(),
                Intermediates::from(intermediates),
                final_char,
                data.to_owned() as DataString
            ));
        }

        fn handle_osc(&mut self, params: Params, intermediates: &Inter, data: &DataStr) {
            self.0.push_back(Value::Osc(
                params.to_owned(),
                Intermediates::from(intermediates),
                DataString::from(data),
            ));
        }

        fn handle_sos(&mut self, data: &DataStr) {
            self.0.push_back(Value::Sos(data.to_owned()));
        }

        fn handle_pm(&mut self, data: &DataStr) {
            self.0.push_back(Value::Pm(data.to_owned()));
        }

        fn handle_apc(&mut self, data: &DataStr) {
            self.0.push_back(Value::Apc(data.to_owned()));
        }
    }

    impl Iterator for Recorder {
        type Item = Value;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }

    struct Harness {
        handler: Recorder,
        engine: Engine,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                handler: Recorder::new(),
                engine: Engine::default(),
            }
        }

        fn advance(&mut self, chars: impl AsRef<[u8]>) -> &mut Recorder {
            self.engine.advance(&mut self.handler, chars);
            &mut self.handler
        }
    }

    // ---- Ground / print / execute ---------------------------------------

    #[test]
    fn prints_plain_text() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance("abc").collect();
        assert_eq!(
            events,
            vec![Value::Utf8('a'), Value::Utf8('b'), Value::Utf8('c')]
        );
    }

    #[test]
    fn prints_utf8() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance("\x1B[1m🦀🦀💔👨🏿").collect();
        assert_eq!(
            events,
            vec![
                Value::Csi(params![[1]], Intermediates::empty(), 'm'),
                Value::Utf8('🦀'),
                Value::Utf8('🦀'),
                Value::Utf8('💔'),
                Value::Utf8('👨'),
                Value::Utf8('🏿'),
            ]
        );
        let mut h = Harness::new();
    }
    #[test]
    fn executes_c0_controls() {
        let mut h = Harness::new();
        // BEL, BS, TAB, LF, CR
        let events: Vec<_> = h.advance(b"\x07\x08\x09\x0A\x0D").collect();
        assert_eq!(
            events,
            vec![
                Value::Control(0x07),
                Value::Control(0x08),
                Value::Control(0x09),
                Value::Control(0x0A),
                Value::Control(0x0D),
            ]
        );
    }

    // ---- ESC ------------------------------------------------------------

    #[test]
    fn simple_esc_dispatch() {
        let mut h = Harness::new();
        // ESC 7 — DECSC
        let events: Vec<_> = h.advance(b"\x1B7").collect();
        assert_eq!(events, vec![Value::Esc(Intermediates::empty(), b'7')]);
    }

    #[test]
    fn esc_with_intermediate() {
        let mut h = Harness::new();
        // ESC # 8 — DECALN (screen alignment)
        let events: Vec<_> = h.advance(b"\x1B#8").collect();
        assert_eq!(events, vec![Value::Esc(Intermediates::from(b"#"), b'8')]);
    }

    // ---- CSI ------------------------------------------------------------

    #[test]
    fn csi_single_param() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(params![[1]], Intermediates::empty(), 'm')]
        );
    }

    #[test]
    fn csi_multiple_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1;2;3m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params![[1], [2], [3]],
                Intermediates::empty(),
                'm'
            )]
        );
    }

    #[test]
    fn csi_subparams() {
        let mut h = Harness::new();
        // SGR 38:2:255:128:0 — 24-bit fg via sub-params
        let events: Vec<_> = h.advance(b"\x1B[38:2:255:128:0m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params![[38, 2, 255, 128, 0]],
                Intermediates::empty(),
                'm'
            )]
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1;2:3:4;5m").collect();
        let slice = [&[1], &[2, 3, 4], &[5]] as [&[_]; _];

        let mut params = Parameter::empty();
        params.push_group([1u16]);
        params.push_group([2u16, 3u16, 4u16]);
        params.push_group([5u16]);
        assert_eq!(events, vec![Value::Csi(params, Intermediates::empty(), 'm')]);
    }

    #[test]
    fn csi_private_marker() {
        let mut h = Harness::new();
        // DECSET — CSI ? 25 h
        let events: Vec<_> = h.advance(b"\x1B[?25h").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params![[25]],
                Intermediates::from(b"?"),
                'h'
            )]
        );
    }

    #[test]
    fn csi_intermediate() {
        let mut h = Harness::new();
        // CSI SP q — DECSCUSR (cursor shape)
        let events: Vec<_> = h.advance(b"\x1B[2 q").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params![[2]],
                Intermediates::from(&b" "[..]),
                'q'
            )]
        );
    }

    #[test]
    fn csi_no_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(Parameter::empty(), Intermediates::empty(), 'm')]
        );
    }

    #[test]
    fn csi_empty_param_becomes_zero() {
        let mut h = Harness::new();
        // `;1m` — empty leading param means 0
        let events: Vec<_> = h.advance(b"\x1B[;1m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(params![0, 1], Intermediates::empty(), 'm')]
        );
    }

    // ---- OSC ------------------------------------------------------------

    #[test]
    fn osc_st_terminated() {
        let mut h = Harness::new();
        // OSC 0 ; title ST  (ST = ESC \)
        let events: Vec<_> = h.advance(b"\x1B]0;title\x1B\\").collect();
        // Expect an Osc event, followed by an empty ESC \ dispatch.
        assert_eq!(
            events,
            vec![
                Value::Osc(
                    Parameter::empty(),
                    Intermediates::empty(),
                    DataString::from(b"0;title")
                ),
                Value::Esc(Intermediates::empty(), b'\\'),
            ]
        );
    }

    #[test]
    fn dcs() {
        let mut h = Harness::new();
        // DCS P $ q $| ESC \
        let events: Vec<_> = h.advance(b"\x1B$q\x1B\x1B\\").collect();
        assert_eq!(
            events,
            vec![
                Value::Dcs(
                    Parameter::empty(),
                    Intermediates::from(&b"$"[..]),
                    'q',
                    DataString::from(b"")
                ),
                Value::Esc(Intermediates::empty(), b'\\'),
            ]
        );

    }

    // ---- Cancellation ---------------------------------------------------

    #[test]
    fn can_cancels_sequence() {
        let mut h = Harness::new();
        // ESC then CAN (0x18) — CAN returns to Ground without dispatch.
        let events: Vec<_> = h.advance(b"\x1B\x18").collect();
        assert_eq!(events, vec![Value::Control(0x18)]);
    }
}
