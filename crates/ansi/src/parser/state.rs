use bilge::prelude::*;
use maybe::Maybe;
use utils::{transition};

transition! {
    #[bitsize(8)]
    #[derive(Copy, Debug, Default, Maybe)]
    #[derive_const(Clone, PartialEq, Eq)]
    #[repr(u8)]
    enum Action;

    #[bitsize(8)]
    #[derive(Copy, Debug, Default, Maybe)]
    #[derive_const(Clone, PartialEq, Eq)]
    #[repr(u8)]
    enum State;

    // Anywhere transitions (from any state)
   _ => {
        0x18       => [Action::Execute, State::Ground],
        0x1a       => [Action::Execute, State::Ground],
        0x80..=0x8f => [Action::Execute, State::Ground],
        0x91..=0x97 => [Action::Execute, State::Ground],
        0x99       => [Action::Execute, State::Ground],
        0x9a       => [Action::Execute, State::Ground],
        0x9c       => State::Ground,
        0x1b       => State::Escape,
        0x98       => State::SosPmApcData,
        0x9e       => State::SosPmApcData,
        0x9f       => State::SosPmApcData,
        0x90       => State::DcsEntry,
        0x9d       => State::OscString,
        0x9b       => State::CsiEntry,
    },

    #[default]
    State::Ground =>  {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x7f => Action::Print,
    },

    // State: Escape
    State::Escape => {
        on_entry => Action::Clear,

        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x7f       => Action::Ignore,
        0x20..=0x2f => [Action::Collect, State::EscapeIntermediate],
        0x30..=0x4f => [Action::EscDispatch, State::Ground],
        0x51..=0x57 => [Action::EscDispatch, State::Ground],
        0x59       => [Action::EscDispatch, State::Ground],
        0x5a       => [Action::EscDispatch, State::Ground],
        0x5c       => [Action::EscDispatch, State::Ground],
        0x60..=0x7e => [Action::EscDispatch, State::Ground],
        0x5b       => State::CsiEntry,
        0x5d       => State::OscString,
        0x50       => State::DcsEntry,
        0x58       => State::SosPmApcData,
        0x5e       => State::SosPmApcData,
        0x5f       => State::SosPmApcData,
    },

    // State: EscapeIntermediate
    State::EscapeIntermediate => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x2f => Action::Collect,
        0x7f       => Action::Ignore,
        0x30..=0x7e => [Action::EscDispatch, State::Ground],
    },

    // State: CsiEntry
    State::CsiEntry => {
        on_entry => Action::Clear,

        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x7f       => Action::Ignore,
        0x20..=0x2f => [Action::Collect, State::CsiIntermediate],
        0x3a       => State::CsiIgnore,
        0x30..=0x39 => [Action::Param, State::CsiParam],
        0x3b       => [Action::Param, State::CsiParam],
        0x3c..=0x3f => [Action::Collect, State::CsiParam],
        0x40..=0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: CsiIgnore
    State::CsiIgnore => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x3f => Action::Ignore,
        0x7f       => Action::Ignore,
        0x40..=0x7e => State::Ground,
    },

    // State: CsiParam
    State::CsiParam => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x30..=0x39 => Action::Param,
        0x3b       => Action::Param,
        0x7f       => Action::Ignore,
        0x3a       => Action::Param,
        0x3c..=0x3f => State::CsiIgnore,
        0x20..=0x2f => [Action::Collect, State::CsiIntermediate],
        0x40..=0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: CsiIntermediate
    State::CsiIntermediate => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x2f => Action::Collect,
        0x7f       => Action::Ignore,
        0x30..=0x3f => State::CsiIgnore,
        0x40..=0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: DcsEntry
    State::DcsEntry => {
        on_entry => Action::Clear,

        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x7f       => Action::Ignore,
        0x3a       => State::DcsIgnore,
        0x20..=0x2f => [Action::Collect, State::DcsIntermediate],
        0x30..=0x39 => [Action::Param, State::DcsParam],
        0x3b       => [Action::Param, State::DcsParam],
        0x3c..=0x3f => [Action::Collect, State::DcsParam],
        0x40..=0x7e => State::DcsData,
    },

    // State: DcsIntermediate
    State::DcsIntermediate => {
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x2f => Action::Collect,
        0x7f       => Action::Ignore,
        0x30..=0x3f => State::DcsIgnore,
        0x40..=0x7e => State::DcsData,
    },

    // State: DcsIgnore
    State::DcsIgnore => {
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x7f => Action::Ignore,
    },

    // State: DcsParam
    State::DcsParam => {

        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x30..=0x39 => Action::Param,
        0x3b       => Action::Param,
        0x7f       => Action::Ignore,
        0x3a       => Action::Param,
        0x3c..=0x3f => Action::Param,
        0x20..=0x2f => [Action::Collect, State::DcsIntermediate],
        0x40..=0x7e => State::DcsData,
    },

    // State: DcsPassthrough
    State::DcsData => {
        on_entry => Action::DcsStart,
        on_exit  => Action::DcsEnd,

        0x00..=0x17 => Action::DcsPut,
        0x19       => Action::DcsPut,
        0x1c..=0x1f => Action::DcsPut,
        0x20..=0x7e => Action::DcsPut,
        0x7f       => Action::Ignore,
        0xa0..=0xff => Action::DcsPut,
    },

    // State: SosPmApcString
    State::SosPmApcData => {
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x7f => Action::Ignore,
    },

    // State: OscString
    State::OscString => {
        on_entry => Action::OscStart,
        on_exit  => Action::OscEnd,

        // BEL is xterm's OSC terminator; on_exit fires OscEnd. Listed before
        // the C0 range so it wins (match arms are tried in source order).
        0x07       => State::Ground,
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x7f => Action::OscPut,
        0xa0..=0xff => Action::OscPut,
    },
}
