use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use strum::EnumCount;
use super::{Action, State, Transition};

/// DEC ANSI transition table
///
/// https://vt100.net/emu/dec_ansi_parser
#[derive(Clone, Debug)]
pub struct Table {
    inner: [u16; State::COUNT * 256],
    enter: [Action; State::COUNT],
    exit: [Action; State::COUNT],
}

pub static GLOBAL: Table = Table::default();

impl const Table {
    // Table values are encoded as:
    //
    //   inner index:   (state << INDEX_STATE_SHIFT) | byte
    //   inner value:   (action << TRANSITION_ACTION_SHIFT) | next_state
    //
    // `State` uses 5 bits (17 states); `Action` takes the remaining bits.
    //
    // "No transition" is encoded as `Action::None` (0).
    const TRANSITION_ACTION_SHIFT: usize = 5;
    const TRANSITION_STATE_MASK: u16 = 0x1F; // 5 bits for State
    const INDEX_STATE_SHIFT: usize = 8;

    pub const GLOBAL: &'static Self = &GLOBAL;

    pub fn empty() -> Self {
        Table {
            inner: [u16::default(); _],
            enter: [Action::None; _],
            exit: [Action::None; _],
        }
    }

    pub fn add<Bytes>(&mut self, bytes: Bytes, state: State, action: Action, next: State) where Self: [const] Transition<Bytes> {
        Transition::add(self, bytes, state, action, next);
    }

    pub fn action<Bytes>(&mut self, bytes: Bytes, state: State, action: Action) where Self: [const] Transition<Bytes> {
        Transition::add(self, bytes, state, action, state);
    }
    pub fn next<Bytes>(&mut self, bytes: Bytes, state: State, next: State) where Self: [const] Transition<Bytes> {
        Transition::add(self, bytes, state, Action::None, next);
    }

    pub fn enter(&mut self, state: State, action: Action) {
        self.enter[state as usize] = action;
    }

    pub fn exit(&mut self, state: State, action: Action) {
        self.exit[state as usize] = action;
    }

    pub fn ignore<Bytes>(&mut self, bytes: Bytes, state: State) where Self: [const] Transition<Bytes> {
        Transition::add(self, bytes, state, Action::Ignore, state);
    }

    pub fn transition(&self, state: State, byte: u8) -> (State, Action) {
        let index = Self::index(byte, state);
        let value = self.inner[index];
        (
            State::from((value & Self::TRANSITION_STATE_MASK) as u8),
            Action::from((value >> Self::TRANSITION_ACTION_SHIFT) as u8),
        )
    }

    pub fn on_enter(&self, state: State) -> Action {
        self.enter[state as usize]
    }

    pub fn on_exit(&self, state: State) -> Action {
        self.exit[state as usize]
    }

    pub fn index(byte: u8, state: State) -> usize {
        const INDEX_STATE_SHIFT: usize = 8;

        ((state as usize) << INDEX_STATE_SHIFT) | (byte as usize)
    }

    pub fn value(action: Action, next: State) -> u16 {
        const TRANSITION_ACTION_SHIFT: usize = 5;
        const TRANSITION_STATE_MASK: u16 = 0x1F; // 5 bits for State
        ((action as u16) << TRANSITION_ACTION_SHIFT) | (next as u16)
    }
}

impl const Index<usize> for Table {
    type Output = u16;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
impl const IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
impl Default for Table {
    fn default() -> Self {
        Self::default()
    }
}

