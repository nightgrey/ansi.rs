use std::marker::Destruct;
use crate::parser::{Action, State, Table};

pub const trait Transition<T> {
    fn add(&mut self, value: T, state: State, action: Action, next: State);
}

impl const Transition<u8> for Table {
    fn add(&mut self, value: u8, state: State, action: Action, next: State) {
        self[Table::index(value, state)] = Table::value(action, next);
    }
}
impl<const T: usize> const Transition<&[u8; T]> for Table {
    fn add(&mut self, value: &[u8; T], state: State, action: Action, next: State) {
        let mut i = 0;

        while i < T {
            let byte = value[i];
            self.add(byte, state, action, next);
            i += 1;
        }
    }
}

impl const Transition<&[u8]> for Table {
    fn add(&mut self, value: &[u8], state: State, action: Action, next: State) {
        let mut i = 0;

        while i < value.len() {
            let byte = value[i];
            self.add(byte, state, action, next);
            i += 1;
        }
    }
}

impl const Transition<std::ops::RangeInclusive<u8>> for Table {
    fn add(&mut self, value: std::ops::RangeInclusive<u8>, state: State, action: Action, next: State) {
        let start = *value.start();
        let end = *value.end();

        let mut byte = start;
        while byte <= end && byte < 255 {
            self.add(byte, state, action, next);
            byte += 1;
        }
    }
}


impl<T: [const] Clone + [const] Destruct, const N: usize> const Transition<[T; N]> for Table where Table: [const] Transition<T> {
    fn add(&mut self, value: [T; N], state: State, action: Action, next: State) {
        let mut i = 0;

        while i < N {
            self.add(value[i].clone(), state, action, next);
            i += 1;
        }
    }
}
