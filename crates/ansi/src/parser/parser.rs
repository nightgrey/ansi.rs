use crate::parser::{Action, Handler, Params, Paras, State, Table};

use arrayvec::ArrayVec;
use smallvec::SmallVec;

#[derive(Debug, Default)]
pub struct Engine {
    pub state: State,

    pub params: ParamsBuilder,
    pub intermediates: ArrayVec<u8, 16>,

    pub data: ArrayVec<u8, 1024>,
    pub utf8: ArrayVec<u8, 4>,

    /// Final byte of the DCS prefix — captured on entry to `DcsData` so
    /// `handle_dcs` can be invoked with it when the string terminates.
    dcs_final: u8,
}

impl Engine {
    pub fn advance(&mut self, handler: &mut dyn Handler, chars: impl AsRef<[u8]>) {
        for &byte in chars.as_ref().iter() {
            match self.state {
                State::Utf8 => self.advance_utf8(handler, byte),
                _ => self.step(handler, byte),
            }
        }
    }

    fn advance_utf8(&mut self, handler: &mut dyn Handler, byte: u8) {
        if !self.utf8.is_full() {
            self.utf8.push(byte);
        }

        let expected_len = match self.utf8[0] {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ => {
                self.utf8.clear();
                self.state = State::Ground;
                return;
            }
        };

        if self.utf8.len() < expected_len {
            return;
        }

        if let Ok(s) = str::from_utf8(&self.utf8) {
            if let Some(ch) = s.chars().next() {
                handler.utf8(ch);
            }
        }

        self.utf8.clear();
        self.state = State::Ground;
    }

    fn step(&mut self, handler: &mut dyn Handler, byte: u8) {
        let prev_state = self.state;
        let (next_state, action) = Table::global_transition(prev_state, byte);

        if next_state != prev_state {
            let exit_action = Table::global().exit(prev_state);
            let entry_action = Table::global().entry(next_state);

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

    fn action(&mut self, handler: &mut dyn Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore | Action::Prefix => {}

            Action::Print => {
                handler.utf8(byte as char);
            }

            Action::Execute => {
                handler.control(byte);
            }

            Action::Clear => {
                self.clear();
            }

            Action::Collect => {
                let _ = self.intermediates.try_push(byte);
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
                let _ = self.data.try_push(byte);
            }

            Action::OscStart | Action::DcsStart => {
                self.data.clear();
            }

            Action::OscEnd => {
                if self.params.has_unfinished() {
                    self.params.finish_param();
                }
                handler.handle_osc(self.params.as_slice(), &self.intermediates, &self.data);
            }

            Action::DcsEnd => {
                if self.params.has_unfinished() {
                    self.params.finish_param();
                }
                handler.handle_dcs(
                    self.params.as_slice(),
                    &self.intermediates,
                    self.dcs_final as char,
                    &self.data,
                );
            }

            Action::Dispatch => {
                if self.params.has_unfinished() {
                    self.params.finish_param();
                }

                match self.state {
                    State::CsiEntry | State::CsiParam | State::CsiIntermediate => {
                        handler.handle_csi(
                            self.params.as_slice(),
                            &self.intermediates,
                            byte as char,
                        );
                    }
                    State::Escape | State::EscapeIntermediate => {
                        handler.handle_esc(&self.intermediates, byte);
                    }
                    _ => (),
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

/// Incremental builder for a nested [`Params`] structure.
///
/// ANSI parameters are two-level: `;` separates main parameters and `:`
/// separates sub-parameters within a main parameter. `1;2:3:4;5` parses to
/// three groups: `[[1], [2, 3, 4], [5]]`.
#[derive(Default, Debug)]
pub struct ParamsBuilder<const N: usize = 16> {
    inner: Params<N>,
    /// Sub-parameters accumulated for the current (unfinished) main parameter.
    current_group: SmallVec<u16, 4>,
    /// Digit-accumulating value of the current sub-parameter.
    current_param: Option<u16>,
}

impl<const N: usize> ParamsBuilder<N> {
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
    use derive_more::{Deref, DerefMut};
    use std::collections::VecDeque;
    use utils::SmallByteString;

    type Params = crate::parser::Params<16>;
    type Intermediates = crate::parser::Intermediates<16>;
    type Data = SmallByteString<1024>;

    #[derive(Debug, Clone, PartialEq, Eq)]
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

    fn params_from<const G: usize>(groups: [&[u16]; G]) -> Params {
        let mut p = Params::new();
        for g in groups {
            p.push_group(g.iter().copied());
        }
        p
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
        assert_eq!(events, vec![Value::Esc(Intermediates::new(), b'7')]);
    }

    #[test]
    fn esc_with_intermediate() {
        let mut h = Harness::new();
        // ESC # 8 — DECALN (screen alignment)
        let events: Vec<_> = h.advance(b"\x1B#8").collect();
        assert_eq!(
            events,
            vec![Value::Esc(Intermediates::from(&b"#"[..]), b'8')]
        );
    }

    // ---- CSI ------------------------------------------------------------

    #[test]
    fn csi_single_param() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(params_from([&[1]]), Intermediates::new(), 'm')]
        );
    }

    #[test]
    fn csi_multiple_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1;2;3m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params_from([&[1], &[2], &[3]]),
                Intermediates::new(),
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
                params_from([&[38, 2, 255, 128, 0]]),
                Intermediates::new(),
                'm'
            )]
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1;2:3:4;5m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params_from([&[1], &[2, 3, 4], &[5]]),
                Intermediates::new(),
                'm'
            )]
        );
    }

    #[test]
    fn csi_private_marker() {
        let mut h = Harness::new();
        // DECSET — CSI ? 25 h
        let events: Vec<_> = h.advance(b"\x1B[?25h").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params_from([&[25]]),
                Intermediates::from(&b"?"[..]),
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
                params_from([&[2]]),
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
            vec![Value::Csi(Params::new(), Intermediates::new(), 'm')]
        );
    }

    #[test]
    fn csi_empty_param_becomes_zero() {
        let mut h = Harness::new();
        // `;1m` — empty leading param means 0
        let events: Vec<_> = h.advance(b"\x1B[;1m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(
                params_from([&[0], &[1]]),
                Intermediates::new(),
                'm'
            )]
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
                    Params::new(),
                    Intermediates::new(),
                    Data::from(&b"0;title"[..])
                ),
                Value::Esc(Intermediates::new(), b'\\'),
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
