use crate::parser::{Handler, Parser};
use crate::{Intermediates, Parameters, Params};
use derive_more::{Deref, DerefMut};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub enum Record {
    Print(char),
    Execute(u8),
    Esc(Intermediates, u8),
    Csi(Parameters, Intermediates, char),
    Dcs(Parameters, Intermediates, char),
    DcsByte(u8),
    DcsEnd(u8),
    OscStart,
    OscByte(u8),
    OscEnd(u8),
    ApcStart,
    ApcByte(u8),
    ApcEnd(u8),
}

impl std::fmt::Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Record::Print(c) => write!(f, "Print({:?} / 0x{:2x})", *c, *c as u8),
            Record::Execute(b) => write!(f, "Execute({} / 0x{:2x})", *b as char, b),
            Record::Esc(i, b) => write!(f, "Esc({:?},  {} / 0x{:2x})", i, *b as char, b),
            Record::Csi(p, i, c) => {
                write!(f, "Csi({:?}, {:?}, {} / 0x{:2x})", p, i, { *c }, *c as u8)
            }
            Record::Dcs(p, i, c) => {
                write!(f, "DcsStart({:?}, {:?}, {} / 0x{:2x})", p, i, *c, *c as u8)
            }
            Record::DcsByte(b) => write!(f, "DcsByte({} / 0x{:2x})", *b as char, *b),
            Record::DcsEnd(b) => {
                write!(f, "DcsEnd({} / 0x{:2x})", *b as char, *b)
            }
            Record::OscStart => write!(f, "OscStart"),
            Record::OscByte(b) => write!(f, "OscByte({} / 0x{:2x})", *b as char, *b),
            Record::OscEnd(b) => {
                write!(f, "OscEnd({} / 0x{:2x})", *b as char, *b)
            }
            Record::ApcStart => write!(f, "ApcStart"),
            Record::ApcByte(b) => write!(f, "ApcByte({} / 0x{:2x})", *b as char, *b),
            Record::ApcEnd(b) => {
                write!(f, "ApcEnd({} / 0x{:2x})", *b as char, *b)
            }
        }
    }
}
#[derive(Default, DerefMut, Deref, PartialEq)]
pub struct Recorder {
    records: Vec<Record>,
}
impl Recorder {
    pub fn new() -> Self {
        Recorder::default()
    }

    pub fn record(bytes: impl AsRef<[u8]>) -> Recorder {
        let mut recorder = Recorder::new();
        Parser::advanced(&mut recorder, bytes.as_ref());
        recorder
    }
}
impl Handler for Recorder {
    fn print(&mut self, char: char) {
        self.push(Record::Print(char));
    }

    fn control(&mut self, byte: u8) {
        self.push(Record::Execute(byte));
    }

    fn esc(&mut self, intermediates: &[u8], final_byte: u8) {
        self.push(Record::Esc(Intermediates::from(intermediates), final_byte));
    }
    fn csi(&mut self, params: &Params, intermediates: &[u8], final_byte: char) {
        self.push(Record::Csi(
            params.to_owned(),
            Intermediates::from(intermediates),
            final_byte,
        ));
    }
    fn dcs(&mut self, params: &Params, intermediates: &[u8], final_char: char) {
        self.push(Record::Dcs(
            params.to_owned(),
            Intermediates::from(intermediates),
            final_char,
        ));
    }
    fn dcs_byte(&mut self, byte: u8) {
        self.push(Record::DcsByte(byte));
    }
    fn dcs_end(&mut self, byte: u8) {
        self.push(Record::DcsEnd(byte));
    }
    fn osc(&mut self) {
        self.push(Record::OscStart);
    }
    fn osc_byte(&mut self, byte: u8) {
        self.push(Record::OscByte(byte));
    }
    fn osc_end(&mut self, byte: u8) {
        self.push(Record::OscEnd(byte));
    }

    fn apc(&mut self) {
        self.push(Record::ApcStart);
    }

    fn apc_byte(&mut self, byte: u8) {
        self.push(Record::ApcByte(byte));
    }

    fn apc_end(&mut self, byte: u8) {
        self.push(Record::ApcEnd(byte));
    }
}

impl Debug for Recorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.records).finish()
    }
}
impl PartialEq<Vec<Record>> for Recorder {
    fn eq(&self, other: &Vec<Record>) -> bool {
        &self.records == other
    }
}
impl PartialEq<[Record]> for Recorder {
    fn eq(&self, other: &[Record]) -> bool {
        &self.records == other
    }
}
impl<const N: usize> PartialEq<[Record; N]> for Recorder {
    fn eq(&self, other: &[Record; N]) -> bool {
        &self.records == other
    }
}

#[macro_export]
macro_rules! records {
    ($bytes:expr) => {{
        let mut recorder = Recorder::new();
        let mut parser = Parser::advanced(&mut recorder, $bytes.as_ref());

        recorder
    }};
}

#[macro_export]
macro_rules! assert_parse {
    ($bytes:expr, [ $($record:expr),* ]) => {
        let mut recorder = Recorder::new();
        Parser::advanced(&mut recorder, $bytes.as_ref());
        assert_eq!(recorder, [$($record),*]);
    };
    ($bytes:expr, $($record:expr),*) => {
        let mut recorder = Recorder::new();
        Parser::advanced(&mut recorder, $bytes.as_ref());
        assert_eq!(recorder, [$($record),*]);
    };
}
