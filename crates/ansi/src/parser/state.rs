use derive_more::Display;
use maybe::Maybe;
use std::fmt::{Debug, Display, Formatter};
use utils::transition;

transition! {
    #[derive(Copy, Default, Display, Maybe)]
    #[derive_const(Clone, PartialEq, Eq)]
    #[display("Action::{_variant}")]
    enum Action;

    #[derive(Copy, Default, Display)]
    #[derive_const(Clone, PartialEq, Eq)]
    #[display("State::{_variant}")]
    enum State;

    // Anywhere transitions (from any state)
      _ => {
        0x18       => (State::Ground, Action::Execute),
        0x1a       => (State::Ground, Action::Execute),
        // 0x99       => (State::Ground, Action::Execute),
        // 0x9a       => (State::Ground, Action::Execute),
        // 0x9c       => (State::Ground, Action::None),
        0x1b       => (State::Escape, Action::None),

        // 0x98       => (State::SosPmData, Action::None),
        // 0x9e       => (State::SosPmData, Action::None),
        // 0x9f       => (State::SosPmData, Action::None),
        // 0x90       => (State::DcsEntry, Action::None),
        // 0x9d       => (State::OscData, Action::None),
        // 0x9b       => (State::CsiEntry, Action::None),

        // //  C1 controls
        // 0x80..=0x8f => (State::Ground, Action::Execute),
        // 0x91..=0x97 => (State::Ground, Action::Execute),
    },

    #[default]
    State::Ground => {
        0x00..=0x17 => (State::Ground, Action::Execute),
        0x19       => (State::Ground, Action::Execute),
        0x1c..=0x1f => (State::Ground, Action::Execute),
        0x20..=0x7f => (State::Ground, Action::Print),
        // Utf8
        0xC2..=0xF4 => (State::Ground, Action::Print),
    },

    State::Escape => {
        on_entry => Action::Clear,

        0x00..=0x17 => (State::Escape, Action::Execute),
        0x19       => (State::Escape, Action::Execute),
        0x1c..=0x1f => (State::Escape, Action::Execute),
        0x7f       => (State::Escape, Action::Ignore),
        0x20..=0x2f => (State::EscapeIntermediate, Action::Collect),
        0x30..=0x4f => (State::Ground, Action::EscDispatch),
        0x51..=0x57 => (State::Ground, Action::EscDispatch),
        0x59       => (State::Ground, Action::EscDispatch),
        0x5a       => (State::Ground, Action::EscDispatch),
        0x5c       => (State::Ground, Action::EscDispatch),
        0x60..=0x7e => (State::Ground, Action::EscDispatch),
        0x5b       => (State::CsiEntry, Action::None),
        0x5d       => (State::OscData, Action::None),
        0x50       => (State::DcsEntry, Action::None),
        0x58       => (State::SosPmData, Action::None),
        0x5e       => (State::SosPmData, Action::None),
        0x5f       => (State::ApcData, Action::None),
    },

    State::EscapeIntermediate => {
        0x00..=0x17 => (State::EscapeIntermediate, Action::Execute),
        0x19       => (State::EscapeIntermediate, Action::Execute),
        0x1c..=0x1f => (State::EscapeIntermediate, Action::Execute),
        0x20..=0x2f => (State::EscapeIntermediate, Action::Collect),
        0x7f       => (State::EscapeIntermediate, Action::Ignore),
        0x30..=0x7e => (State::Ground, Action::EscDispatch),
    },

    State::CsiEntry => {
        on_entry => Action::Clear,

        0x00..=0x17 => (State::CsiEntry, Action::Execute),
        0x19       => (State::CsiEntry, Action::Execute),
        0x1c..=0x1f => (State::CsiEntry, Action::Execute),
        0x7f       => (State::CsiEntry, Action::Ignore),
        0x20..=0x2f => (State::CsiIntermediate, Action::Collect),
        0x3a       => (State::CsiIgnore, Action::None),
        0x30..=0x39 => (State::CsiParam, Action::Param),
        0x3b       => (State::CsiParam, Action::Param),
        0x3c..=0x3f => (State::CsiParam, Action::Collect),
        0x40..=0x7e => (State::Ground, Action::CsiDispatch),
    },

    State::CsiParam => {
        0x00..=0x17 => (State::CsiParam, Action::Execute),
        0x19       => (State::CsiParam, Action::Execute),
        0x1c..=0x1f => (State::CsiParam, Action::Execute),
        0x30..=0x3b => (State::CsiParam, Action::Param),
        0x7f       => (State::CsiParam, Action::Ignore),
        0x3c..=0x3f => (State::CsiIgnore, Action::None),
        0x20..=0x2f => (State::CsiIntermediate, Action::Collect),
        0x40..=0x7e => (State::Ground, Action::CsiDispatch),
    },

    State::CsiIntermediate => {
        0x00..=0x17 => (State::CsiIntermediate, Action::Execute),
        0x19       => (State::CsiIntermediate, Action::Execute),
        0x1c..=0x1f => (State::CsiIntermediate, Action::Execute),
        0x20..=0x2f => (State::CsiIntermediate, Action::Collect),
        0x7f       => (State::CsiIntermediate, Action::Ignore),
        0x30..=0x3f => (State::CsiIgnore, Action::None),
        0x40..=0x7e => (State::Ground, Action::CsiDispatch),
    },

    State::CsiIgnore => {
        0x00..=0x17 => (State::CsiIgnore, Action::Execute),
        0x19       => (State::CsiIgnore, Action::Execute),
        0x1c..=0x1f => (State::CsiIgnore, Action::Execute),
        0x20..=0x3f => (State::CsiIgnore, Action::Ignore),
        0x7f       => (State::CsiIgnore, Action::Ignore),
        0x40..=0x7e => (State::Ground, Action::None),
    },

    State::DcsEntry => {
        on_entry => Action::Clear,

        0x00..=0x17 => (State::DcsEntry, Action::Ignore),
        0x19       => (State::DcsEntry, Action::Ignore),
        0x1c..=0x1f => (State::DcsEntry, Action::Ignore),
        0x7f       => (State::DcsEntry, Action::Ignore),
        0x3a       => (State::DcsIgnore, Action::None),
        0x20..=0x2f => (State::DcsIntermediate, Action::Collect),
        0x30..=0x39 => (State::DcsParam, Action::Param),
        0x3b       => (State::DcsParam, Action::Param),
        0x3c..=0x3f => (State::DcsParam, Action::Collect),
        0x40..=0x7e => (State::DcsData, Action::None),
    },

    State::DcsParam => {
        0x00..=0x17 => (State::DcsParam, Action::Ignore),
        0x19       => (State::DcsParam, Action::Ignore),
        0x1c..=0x1f => (State::DcsParam, Action::Ignore),
        0x30..=0x39 => (State::DcsParam, Action::Param),
        0x3b       => (State::DcsParam, Action::Param),
        0x7f       => (State::DcsParam, Action::Ignore),
        0x3a       => (State::DcsParam, Action::Param),
        0x3c..=0x3f => (State::DcsIgnore, Action::None),
        0x20..=0x2f => (State::DcsIntermediate, Action::Collect),
        0x40..=0x7e => (State::DcsData, Action::None),
    },

    State::DcsIntermediate => {
        0x00..=0x17 => (State::DcsIntermediate, Action::Ignore),
        0x19       => (State::DcsIntermediate, Action::Ignore),
        0x1c..=0x1f => (State::DcsIntermediate, Action::Ignore),
        0x20..=0x2f => (State::DcsIntermediate, Action::Collect),
        0x7f       => (State::DcsIntermediate, Action::Ignore),
        0x30..=0x3f => (State::DcsIgnore, Action::None),
        0x40..=0x7e => (State::DcsData, Action::None),
    },

    State::DcsData => {
        on_entry => Action::Dcs,
        on_exit  => Action::DcsEnd,
        0x00..=0x17 => (State::DcsData, Action::DcsData),
        0x19       => (State::DcsData, Action::DcsData),
        0x1c..=0x1f => (State::DcsData, Action::DcsData),
        0x20..=0x7e => (State::DcsData, Action::DcsData),
        0x7f       => (State::DcsData, Action::Ignore),
        0x80..=0xff => (State::DcsData, Action::DcsData),
    },

    State::DcsIgnore => {
        0x00..=0x17 => (State::DcsIgnore, Action::Ignore),
        0x19       => (State::DcsIgnore, Action::Ignore),
        0x1c..=0x1f => (State::DcsIgnore, Action::Ignore),
        0x20..=0x7f => (State::DcsIgnore, Action::Ignore),
    },

    State::OscData => {
        on_entry => Action::Osc,
        on_exit  => Action::OscEnd,


        0x00..=0x06 => (State::OscData, Action::Ignore),
        0x07       => (State::Ground, Action::Ignore),
        0x08..=0x17 => (State::OscData, Action::Ignore),
        0x19       => (State::OscData, Action::Ignore),
        0x1c..=0x1f => (State::OscData, Action::Ignore),

        0x20..=0x7f => (State::OscData, Action::OscData),
        // Utf8 (leading and continuation bytes)
        0x80..=0xff => (State::OscData, Action::OscData),
    },

    State::SosPmData => {
        0x00..=0x17 => (State::SosPmData, Action::Ignore),
        0x19       => (State::SosPmData, Action::Ignore),
        0x1c..=0x1f => (State::SosPmData, Action::Ignore),
        0x20..=0x7f => (State::SosPmData, Action::Ignore),
    },

    State::ApcData => {
        on_entry => Action::ApcStart,
        on_exit  => Action::ApcEnd,

        0x00..=0x17 => (State::ApcData, Action::ApcByte),
        0x19       => (State::ApcData, Action::ApcByte),
        0x1c..=0x1f => (State::ApcData, Action::ApcByte),
        0x20..=0x7f => (State::ApcData, Action::ApcByte),
    },
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
