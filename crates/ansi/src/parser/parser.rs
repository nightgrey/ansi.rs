use std::fmt::{from_fn, Debug, Formatter, Pointer};
use super::{Action, DataString, FinalChar, Handler, Intermediates, Parameter, Params, State, Table};
use arrayvec::ArrayVec;
use bilge::prelude::Integer;
use smallvec::SmallVec;
use log::debug;
use utils::debug;

enum Part {
    Action(Action),
    State(State)
}

impl Part {
    fn is_none(&self) -> bool {
        match self {
            Self::Action(action) => action == &Action::None,
            Self::State(state) => state == &State::Ground,
        }
    }
}

impl Debug for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action(action) => action.fmt(f),
            Self::State(state) => state.fmt(f),
        }
    }
}

fn step(byte: u8, exit: Action, prev_state: State, action: Action, next_state: State, entry: Action) {
    let transition = |state: State, action: Action| {
        if action != Action::None {
            format!("[{:?}, {:?}]", state, action)
        } else {
            format!("{:?}", state)
        }
    };
    println!("-- {} --", debug(byte));
    if action == Action::None {
        println!("{} ---> {}", transition(prev_state, exit), transition(next_state, entry));

    } else {
        println!("{} --> {:?} --> {}", transition(prev_state, exit), action, transition(next_state, entry));
    }
}

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: ParameterState,
    pub intermediates: Intermediates,

    pub data: DataString,
    pub utf8: ArrayVec<u8, 4>,
}

impl Parser {
    pub fn advance(&mut self, handler: &mut dyn Handler, bytes: impl AsRef<[u8]>) {
        for &byte in bytes.as_ref() {
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

            step(byte,
                 exit_action,
                prev_state,
                action,
                next_state,
                 entry_action,
            );

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
            step(byte,
                 Action::None,
                 prev_state,
                 action,
                 next_state,
                 Action::None,
            );
            self.action(handler, action, byte);
        }
    }

    #[inline]
    fn action(&mut self, handler: &mut dyn Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Print => {
            handler.print(byte as char);
            }

            Action::Execute => {
                handler.control(byte);
            }

            Action::Clear => {
                self.clear();
            }

            Action::Collect => {
                match self.state {
                    State::Utf8 | State::Ground => self.push_utf8(handler, byte),
                    State::DcsData | State::OscData | State::SosData | State::PmData | State::ApcData => {
                        self.data.push(byte);
                    }
                    _ => {
                        self.intermediates.push(byte);
                    }
                }
            }

            Action::Param => match byte {
                b'0'..=b'9' => self.params.push_digit(byte),
                // sub-parameter separator — keep accumulating within the group.
                b':' => self.params.push_subparam(),
                // parameter separator — close the whole group.
                b';' => self.params.finish_param(),
                _ => {}
            },

            Action::Dispatch => match self.state {
                State::Escape => handler.handle_esc(self.intermediates.as_ref(), byte),
                State::DcsData => handler.handle_dcs(
                    self.params.borrow(),
                    self.intermediates.as_ref(),
                    self.data.as_ref(),
                ),
                State::OscData => handler.handle_osc(self.params.borrow(), self.intermediates.as_ref(), self.data.as_ref()),
                State::SosData => handler.handle_sos(self.data.as_ref()),
                State::PmData => handler.handle_pm(self.data.as_ref()),
                State::ApcData => handler.handle_apc(self.data.as_ref()),
                State::CsiEntry | State::CsiParam | State::CsiIntermediate => {
                    if self.params.has_unfinished() {
                        self.params.finish_param();
                    }
                    handler.handle_csi(
                        self.params.borrow(),
                        self.intermediates.as_ref(),
                        byte as char,
                    );
                }
                _ => {}
            }
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
        self.inner.push(self.current_group.drain(..));
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
    use crate::parser::{Apc, Csi, DataStr, Dcs, Esc, Inter, Osc, Pm, Sos};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Value {
        Print(char),
        Control(u8),
        Csi(Csi),
        Esc(Esc),
        Dcs(Dcs),
        Osc(Osc),
        Sos(Sos),
        Pm(Pm),
        Apc(Apc),
    }

    impl PartialEq<Csi> for Value {
        fn eq(&self, other: &Csi) -> bool {
            match self {
                Value::Csi(csi) => csi == other,
                _ => false,
            }
        }
    }

    impl PartialEq<Esc> for Value {
        fn eq(&self, other: &Esc) -> bool {
            match self {
                Value::Esc(esc) => esc == other,
                _ => false,
            }
        }
    }

    impl PartialEq<Dcs> for Value {
        fn eq(&self, other: &Dcs) -> bool {
            match self {
                Value::Dcs(dcs) => dcs == other,
                _ => false,
            }
        }
    }

    impl PartialEq<Osc> for Value {
        fn eq(&self, other: &Osc) -> bool {
            match self {
                Value::Osc(osc) => osc == other,
                _ => false,
            }
        }
    }

    impl PartialEq<Sos> for Value {
        fn eq(&self, other: &Sos) -> bool {
            match self {
                Value::Sos(sos) => sos == other,
                _ => false,
            }
        }
    }

    impl PartialEq<Pm> for Value {
        fn eq(&self, other: &Pm) -> bool {
            match self {
                Value::Pm(pm) => pm == other,
                _ => false,
            }
        }
    }

    impl PartialEq<Apc> for Value {
        fn eq(&self, other: &Apc) -> bool {
            match self {
                Value::Apc(apc) => apc == other,
                _ => false,
            }
        }
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
        fn print(&mut self, ch: char) {
            self.0.push_back(Value::Print(ch));
        }

        fn control(&mut self, byte: u8) {
            self.0.push_back(Value::Control(byte));
        }

        fn handle_csi(&mut self, params: Params, intermediates: &Inter, final_char: FinalChar) {
            self.0.push_back(Value::Csi(Csi {
                params: params.to_owned(),
                intermediates: Intermediates::from(intermediates),
                final_char,
            }));
        }

        fn handle_esc(&mut self, intermediates: &Inter, final_byte: u8) {
            self.push_back(Value::Esc(Esc {
                intermediates: Intermediates::from(intermediates),
                final_byte,
            }));
        }

        fn handle_dcs(
            &mut self,
            params: Params,
            intermediates: &Inter,
            data: &DataStr,
        ) {
            self.0.push_back(Value::Dcs(Dcs {
                params: params.to_owned(),
                intermediates: Intermediates::from(intermediates),
                data: data.to_owned(),
            }));
        }

        fn handle_osc(&mut self, params: Params, intermediates: &Inter, data: &DataStr) {
            self.0.push_back(Value::Osc(Osc {
                params: params.to_owned(),
                intermediates: Intermediates::from(intermediates),
                data: DataString::from(data),
            }));
        }

        fn handle_sos(&mut self, data: &DataStr) {
            self.0.push_back(Value::Sos(Sos(data.to_owned())));
        }

        fn handle_pm(&mut self, data: &DataStr) {
            self.0.push_back(Value::Pm(Pm(data.to_owned())));
        }

        fn handle_apc(&mut self, data: &DataStr) {
            self.0.push_back(Value::Apc(Apc(data.to_owned())));
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
        engine: Parser,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                handler: Recorder::new(),
                engine: Parser::default(),
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
            vec![Value::Print('a'), Value::Print('b'), Value::Print('c')]
        );
    }

    #[test]
    fn prints_utf8() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance("\x1B[?1049h🦀🦀💔👨🏿").collect();
        assert_eq!(
            events,
            vec![
                Value::Csi(Csi {
                    params: Parameter::from_iter([1049]),
                    intermediates: Intermediates::from(b"?"),
                    final_char: 'h',
                }),
                Value::Print('🦀'),
                Value::Print('🦀'),
                Value::Print('💔'),
                Value::Print('👨'),
                Value::Print('🏿'),
            ]
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
        assert_eq!(events, vec![Value::Esc(Esc {
            intermediates: Intermediates::empty(),
            final_byte: b'7',
        })]);
    }

    #[test]
    fn esc_with_intermediate() {
        let mut h = Harness::new();
        // ESC # 8 — DECALN (screen alignment)
        let events: Vec<_> = h.advance(b"\x1B#8").collect();
        assert_eq!(events, vec![Value::Esc(Esc {
            intermediates: Intermediates::from(b"#"),
            final_byte: b'8',
        })]);
    }

    // ---- CSI ------------------------------------------------------------

    #[test]
    fn csi_single_param() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(Csi {
                params: params![[1]],
                intermediates: Intermediates::empty(),
                final_char: 'm',
            })]
        );
    }

    #[test]
    fn csi_multiple_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1;2;3m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(Csi {
                params: params![[1], [2], [3]],
                intermediates: Intermediates::empty(),
                final_char: 'm',
            })]
        );
    }

    #[test]
    fn csi_subparams() {
        let mut h = Harness::new();
        // SGR 38:2:255:128:0 — 24-bit fg via sub-params
        let events: Vec<_> = h.advance(b"\x1B[38:2:255:128:0m").collect();
        assert_eq!(
            events,
            vec![Value::Csi(Csi {
                params: params![[38, 2, 255, 128, 0]],
                intermediates: Intermediates::empty(),
                final_char: 'm',
            })]
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[1;2:3:4;5m").collect();
        let slice = [&[1], &[2, 3, 4], &[5]] as [&[_]; _];


        assert_eq!(events, vec![Value::Csi(Csi {
            params: params![[1], [2, 3, 4], [5]],
            intermediates: Intermediates::empty(),
            final_char: 'm',
        })]);
    }

    #[test]
    fn csi_private_marker() {
        let mut h = Harness::new();
        // DECSET — CSI ? 25 h
        let events: Vec<_> = h.advance(b"\x1B[?25h").collect();
        assert_eq!(
            events,
            vec![Value::Csi(Csi {
                params: params![[25]],
                intermediates: Intermediates::from(b"?"),
                final_char: 'h',
            })]
        );
    }

    #[test]
    fn csi_intermediate() {
        let mut h = Harness::new();
        // CSI SP q — DECSCUSR (cursor shape)
        let events: Vec<_> = h.advance(b"\x1B[2 q").collect();
        assert_eq!(
            events,
            vec![Csi {
                params: params![[2]],
                intermediates: Intermediates::from(&b" "[..]),
                final_char: 'q',
            }]
        );
    }

    #[test]
    fn csi_no_params() {
        let mut h = Harness::new();
        let events: Vec<_> = h.advance(b"\x1B[m").collect();
        assert_eq!(
            events,
            vec![Csi {
                params: Parameter::empty(),
                intermediates: Intermediates::empty(),
                final_char: 'm',
            }]
        );
    }

    #[test]
    fn csi_empty_param_becomes_zero() {
        let mut h = Harness::new();
        // `;1m` — empty leading param means 0
        let events: Vec<_> = h.advance(b"\x1B[;1m").collect();
        assert_eq!(
            events,
            vec![Csi {
                params: params![0, 1],
                intermediates: Intermediates::empty(),
                final_char: 'm',
            }]
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
               Value::Osc(Osc {
                    params: Parameter::empty(),
                    intermediates: Intermediates::empty(),
                    data: DataString::from(b"0;title"),
                }),
                Value::Esc(Esc {
                    intermediates: Intermediates::empty(),
                    final_byte: b'\\',
                })
            ]
        );
    }

    #[test]
    fn dcs() {
        let mut h = Harness::new();
        // DCS P $ q $| ESC \
        let events: Vec<_> = h.advance(b"\x1BP$q q\x1B\\").collect();
        assert_eq!(
            events,
            vec![
                Value::Dcs(Dcs {
                    params: Parameter::empty(),
                    intermediates: Intermediates::from(b"$q"),
                    data: DataString::from(b" q")
                }),
                Value::Esc(Esc {
                    intermediates: Intermediates::empty(),
                    final_byte: b'\\'
                })
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


    #[test]
    fn t() {

        let mut p = Parameter::<16>::empty();
        p.extend([1]);

        dbg!(&p);
        dbg!(p.iter().collect::<Vec<_>>());
    }
}
