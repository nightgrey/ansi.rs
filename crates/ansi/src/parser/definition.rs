use crate::parser::{Action, State, Table, Transition};
macro_rules! each {
    ($($state:ident),* => |$s:ident| $f:expr) => {
        {
           $(let $s = State::$state; $f)*
        }
    };
    (|$state:ident| $f:expr) => {
        each!(
            Ground,
            Escape,
            EscapeIntermediate,
            CsiEntry,
            CsiIgnore,
            CsiParam,
            CsiIntermediate,
            DcsEntry,
            DcsIgnore,
            DcsIntermediate,
            DcsParam,
            DcsData,
            OscData,
            SosData,
            PmData,
            ApcData,
            Utf8 => |$state| $f
        );
    }
}

impl const Table {
    pub fn default() -> Self {
    let mut table = Self::empty();

    // "Anywhere" transitions — apply to every state.
    each!(|state| {
        table.add(0x18, state, Action::Execute, State::Ground);
        table.add(0x1A, state, Action::Execute, State::Ground);
        table.add(0x9C, state, Action::None, State::Ground);

        table.add(0x80..=0x8F, state, Action::Execute, State::Ground);
        table.add(0x91..=0x97, state, Action::Execute, State::Ground);
        table.add(0x99, state, Action::Execute, State::Ground);
        table.add(0x9A, state, Action::Execute, State::Ground);

        table.add(0x1B, state, Action::None, State::Escape);
        table.add(0x90, state, Action::None, State::DcsEntry);
        table.add(0x9B, state, Action::None, State::CsiEntry);

        table.add(0x98, state, Action::None, State::SosData);
        table.add(0x9E, state, Action::None, State::PmData);
        table.add(0x9F, state, Action::None, State::ApcData);
        table.add(0x9D, state, Action::None, State::OscData);

    });

    table.action(0x00..=0x17, State::Ground, Action::Execute);
    table.action(0x19, State::Ground, Action::Execute);
    table.action(0x1C..=0x1F, State::Ground, Action::Execute);
    table.action(0x20..=0x7F, State::Ground, Action::Print);

    table.add(0xC2..=0xDF, State::Ground, Action::Collect, State::Utf8); // UTF8 2 byte sequence
    table.add(0xE0..=0xEF, State::Ground, Action::Collect, State::Utf8); // UTF8 3 byte sequence
    table.add(0xF0..=0xF4, State::Ground, Action::Collect, State::Utf8); // UTF8 4 byte sequence

    table.add(0x80..=0xBF, State::Utf8, Action::Collect, State::Utf8); // Continuation byte


    // State::Escape
    table.enter(State::Escape, Action::Clear);
    table.action(0x00..=0x17, State::Escape, Action::Execute);
    table.action(0x19, State::Escape, Action::Execute);
    table.action(0x1C..=0x1F, State::Escape, Action::Execute);
    table.action(0x7F, State::Escape, Action::Ignore);
    table.add(
        0x20..=0x2F,
        State::Escape,
        Action::Collect,
        State::EscapeIntermediate,
    ); 
    table.add(0x30..=0x4F, State::Escape, Action::Dispatch, State::Ground);
    table.add(0x51..=0x57, State::Escape, Action::Dispatch, State::Ground);
    table.add(0x59, State::Escape, Action::Dispatch, State::Ground);
    table.add(0x5A, State::Escape, Action::Dispatch, State::Ground);
    table.add(0x5C, State::Escape, Action::Dispatch, State::Ground);
    table.add(0x60..=0x7E, State::Escape, Action::Dispatch, State::Ground);
    table.add(0x5B, State::Escape, Action::None, State::CsiEntry);
    table.add(0x5D, State::Escape, Action::None, State::OscData);
    table.add(0x50, State::Escape, Action::None, State::DcsEntry);
    table.add(0x58, State::Escape, Action::None, State::SosData); // SOS
    table.add(0x5E, State::Escape, Action::None, State::PmData); // PM
    table.add(0x5F, State::Escape, Action::None, State::ApcData); // APC

    // State::EscapeIntermediate
    table.action(
        0x00..=0x17,
        State::EscapeIntermediate,
        Action::Execute);
    table.action(
        0x19,
        State::EscapeIntermediate,
        Action::Execute);
    table.action(
        0x1C..=0x1F,
        State::EscapeIntermediate,
        Action::Execute);
    table.action(
        0x20..=0x2F,
        State::EscapeIntermediate,
        Action::Collect);
    table.action(
        0x7F,
        State::EscapeIntermediate,
        Action::Ignore);
    table.add(
        0x30..=0x7E,
        State::EscapeIntermediate,
        Action::Dispatch,
        State::Ground,
    );

    // State::CsiEntry
    table.enter(State::CsiEntry, Action::Clear);
    table.action(
        0x00..=0x17,
        State::CsiEntry,
        Action::Execute);
    table.add(0x19, State::CsiEntry, Action::Execute, State::CsiEntry);
    table.action(
        0x1C..=0x1F,
        State::CsiEntry,
        Action::Execute);
    table.add(0x7F, State::CsiEntry, Action::Ignore, State::CsiEntry);
    table.add(
        0x20..=0x2F,
        State::CsiEntry,
        Action::Collect,
        State::CsiIntermediate,
    );
    table.add(0x3A, State::CsiEntry, Action::None, State::CsiIgnore);
    table.add(0x30..=0x39, State::CsiEntry, Action::Param, State::CsiParam);
    table.add(0x3B, State::CsiEntry, Action::Param, State::CsiParam);
    table.add(
        0x3C..=0x3F,
        State::CsiEntry,
        Action::Collect,
        State::CsiParam,
    );
    table.add(
        0x40..=0x7E,
        State::CsiEntry,
        Action::Dispatch,
        State::Ground,
    );

    // State::CsiIgnore
    table.add(
        0x00..=0x17,
        State::CsiIgnore,
        Action::Execute,
        State::CsiIgnore,
    );
    table.add(0x19, State::CsiIgnore, Action::Execute, State::CsiIgnore);
    table.add(
        0x1C..=0x1F,
        State::CsiIgnore,
        Action::Execute,
        State::CsiIgnore,
    );
    table.add(
        0x20..=0x3F,
        State::CsiIgnore,
        Action::Ignore,
        State::CsiIgnore,
    );
    table.add(0x7F, State::CsiIgnore, Action::Ignore, State::CsiIgnore);
    table.add(0x40..=0x7E, State::CsiIgnore, Action::None, State::Ground);

    // State::CsiParam
    table.add(
        0x00..=0x17,
        State::CsiParam,
        Action::Execute,
        State::CsiParam,
    );
    table.add(0x19, State::CsiParam, Action::Execute, State::CsiParam);
    table.add(
        0x1C..=0x1F,
        State::CsiParam,
        Action::Execute,
        State::CsiParam,
    );
    table.add(0x30..=0x39, State::CsiParam, Action::Param, State::CsiParam);
    table.add(0x3A, State::CsiParam, Action::Param, State::CsiParam);
    table.add(0x3B, State::CsiParam, Action::Param, State::CsiParam);
    table.add(0x7F, State::CsiParam, Action::Ignore, State::CsiParam);
    table.add(0x3C..=0x3F, State::CsiParam, Action::None, State::CsiIgnore);
    table.add(
        0x20..=0x2F,
        State::CsiParam,
        Action::Collect,
        State::CsiIntermediate,
    );
    table.add(
        0x40..=0x7E,
        State::CsiParam,
        Action::Dispatch,
        State::Ground,
    );

    // State::CsiIntermediate
    table.add(
        0x00..=0x17,
        State::CsiIntermediate,
        Action::Execute,
        State::CsiIntermediate,
    );
    table.add(
        0x19,
        State::CsiIntermediate,
        Action::Execute,
        State::CsiIntermediate,
    );
    table.add(
        0x1C..=0x1F,
        State::CsiIntermediate,
        Action::Execute,
        State::CsiIntermediate,
    );
    table.add(
        0x20..=0x2F,
        State::CsiIntermediate,
        Action::Collect,
        State::CsiIntermediate,
    );
    table.ignore(0x7F, State::CsiIntermediate);
    table.add(
        0x30..=0x3F,
        State::CsiIntermediate,
        Action::None,
        State::CsiIgnore,
    );
    table.add(
        0x40..=0x7E,
        State::CsiIntermediate,
        Action::Dispatch,
        State::Ground,
    );

    // State::DcsEntry
    table.enter(State::DcsEntry, Action::Clear);
    table.action(
        0x00..=0x17,
        State::DcsEntry,
        Action::Ignore,
    );
    table.action(0x19, State::DcsEntry, Action::Ignore);
    table.action(
        0x1C..=0x1F,
        State::DcsEntry,
        Action::Ignore
    );
    table.action(0x7F, State::DcsEntry, Action::Ignore);
    table.add(0x3A, State::DcsEntry, Action::None, State::DcsIgnore);
    table.add(
        0x20..=0x2F,
        State::DcsEntry,
        Action::Collect,
        State::DcsIntermediate,
    );
    table.add(0x30..=0x39, State::DcsEntry, Action::Param, State::DcsParam);
    table.add(0x3B, State::DcsEntry, Action::Param, State::DcsParam);
    table.add(
        0x3C..=0x3F,
        State::DcsEntry,
        Action::Collect,
        State::DcsParam,
    );
    table.add(0x40..=0x7E, State::DcsEntry, Action::None, State::DcsData);

    // State::DcsIntermediate
    table.action(
        0x00..=0x17,
        State::DcsIntermediate,
        Action::Ignore,
    );
    table.action(
        0x19,
        State::DcsIntermediate,
        Action::Ignore,
    );
    table.action(
        0x1C..=0x1F,
        State::DcsIntermediate,
        Action::Ignore,
    );
    table.action(
        0x20..=0x2F,
        State::DcsIntermediate,
        Action::Collect,
    );
    table.action(0x7F, State::DcsIntermediate, Action::Ignore);
    table.add(
        0x30..=0x3F,
        State::DcsIntermediate,
        Action::Ignore,
        State::DcsIgnore,
    );
    table.add(
        0x40..=0x7E,
        State::DcsIntermediate,
        Action::Collect,
        State::DcsData,
    );

    // State::DcsIgnore
    table.action(
        0x00..=0x17,
        State::DcsIgnore,
        Action::Ignore,
    );
    table.action(0x19, State::DcsIgnore, Action::Ignore);
    table.action(
        0x1C..=0x1F,
        State::DcsIgnore,
        Action::Ignore,
    );
    table.action(
            0x1C..=0x1F,
        State::DcsIgnore,
        Action::Ignore
    );
    table.action(
        0x20..=0x7F,
        State::DcsIgnore,
        Action::Ignore
    );

    // State::DcsParam
    table.action(
        0x00..=0x17,
        State::DcsParam,
        Action::Ignore
    );
    table.action(0x19, State::DcsParam, Action::Ignore);
    table.action(
        0x1C..=0x1F,
        State::DcsParam,
        Action::Ignore
    );
    table.action(0x30..=0x39, State::DcsParam, Action::Param);
    table.action(0x3B, State::DcsParam, Action::Param);
    table.action(0x7F, State::DcsParam, Action::Ignore);

    table.next(0x3A, State::DcsParam, State::DcsIgnore);
    table.next(0x3C..=0x3F, State::DcsParam, State::DcsIgnore);
    table.add(
        0x20..=0x2F,
        State::DcsParam,
        Action::Collect,
        State::DcsIntermediate,
    );
    table.next(0x40..=0x7E, State::DcsParam, State::DcsData);

    // String data
    each!(DcsData, OscData, SosData, PmData, ApcData => |state| {
        table.enter(state, Action::Clear);
        table.action(0x7F, state, Action::Ignore);
        table.next(0x9C, state, State::Ground);
        table.next(0x9C, state, State::Ground);
        table.exit(state, Action::Dispatch);
    });


    each!(DcsData, OscData, SosData, PmData, ApcData => |state| {
            table.add(0x00..=0x17, state, Action::Collect, state);
            table.add(0x19, state, Action::Collect, state);
            table.add(0x1C..=0x1F, state, Action::Collect, state);
            table.add(0x20..=0x7F, state, Action::Collect, state);
    });


    // UTF-8 passthrough for every string-data state. Overrides the C1
    // anywhere rules for 0x80..=0x9F — without this, a continuation byte
    // in that range (e.g. 0x9F in the 🦀 encoding `F0 9F A6 80`) would
    // fire an APC/SOS/etc. anywhere transition and shred the payload.
    // ST (0x9C) is re-bound afterwards so it still terminates the string.
    // each!(DcsData, OscData, SosData, PmData, ApcData => |state| {
    //         table.add(0x80..=0xFF, state, Action::Collect, state);
    //         table.add(0x9C, state, Action::None, State::Ground);
    //     });

    table
}
}