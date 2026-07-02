use super::*;
use crate::{Intermediates, Parameters, Params};
use derive_more::{Deref, DerefMut};
use maybe::Maybe;
use std::fmt::{Debug, from_fn};

#[derive(Clone, PartialEq, Eq)]
pub enum Record {
    Char(char),
    Print(Vec<u8>),
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
            Record::Char(c) => write!(f, "Char({})", c.escape_default()),
            Record::Print(b) => write!(f, "Print({:?})", String::from_utf8_lossy(b)),
            Record::Execute(b) => write!(f, "Execute({})", (*b as char).escape_default()),
            Record::Esc(i, b) => write!(f, "Esc({:?},  {})", i, (*b as char).escape_default()),
            Record::Csi(p, i, c) => {
                write!(f, "Csi({:?}, {:?}, {})", p, i, (*c as char).escape_default())
            }
            Record::Dcs(p, i, c) => {
                write!(f, "Dcs({:?}, {:?}, {})", p, i, (*c as char).escape_default())
            }
            Record::DcsByte(b) => write!(f, "DcsByte({})", (*b as char).escape_default()),
            Record::DcsEnd(b) => {
                write!(f, "DcsEnd({})", (*b as char).escape_default())
            }
            Record::OscStart => write!(f, "OscStart"),
            Record::OscByte(b) => write!(f, "OscByte({})", (*b as char).escape_default()),
            Record::OscEnd(b) => {
                write!(f, "OscEnd({})", (*b as char).escape_default())
            }
            Record::ApcStart => write!(f, "ApcStart"),
            Record::ApcByte(b) => write!(f, "ApcByte({})", (*b as char).escape_default()),
            Record::ApcEnd(b) => {
                write!(f, "ApcEnd({})", (*b as char).escape_default())
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
        Parser::new().advance(&mut recorder, bytes.as_ref());
        recorder
    }
}
impl Handler for Recorder {
    fn print(&mut self, bytes: &[u8]) {
        self.push(Record::Print(bytes.to_vec()));
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

impl Utf8Handler for Recorder {
    fn char(&mut self, char: char) {
        self.push(Record::Char(char));
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
        let mut parser = Utf8Parser::advanced(&mut recorder, $bytes.as_ref());

        recorder
    }};
}

#[macro_export]
macro_rules! assert_parser {
    ($bytes:expr, [ $($record:expr),* ]) => {
        let mut recorder = Recorder::new();
        let mut parser = Parser::new();
        parser.advance(&mut recorder, $bytes.as_ref());
        assert_eq!(recorder, [$($record),*]);
    };
    ($bytes:expr, $($record:expr),*) => {
        let mut recorder = Recorder::new();
        let mut parser = Parser::new();
        parser.advance(&mut recorder, $bytes.as_ref());
        assert_eq!(recorder, [$($record),*]);
    };
}

#[macro_export]
macro_rules! assert_utf8_parser {
    ($bytes:expr, [ $($record:expr),* ]) => {
        let mut recorder = Recorder::new();
        let mut parser = Utf8Parser::new();
        parser.advance(&mut recorder, $bytes.as_ref());
        assert_eq!(recorder, [$($record),*]);
    };
    ($bytes:expr, $($record:expr),*) => {
        let mut recorder = Recorder::new();
        let mut parser = Utf8Parser::new();
        parser.advance(&mut recorder, $bytes.as_ref());
        assert_eq!(recorder, [$($record),*]);
    };
}
// ANSI color codes
const BOLD: &str = "\x1b[1m";

const FG_GREY: &str = "\x1b[38;2;150;150;150m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

pub fn debug_byte(byte: u8) -> String {
    format!("{YELLOW}[0x{byte:02X}]{RESET}")
}
pub fn debug_advance(bytes: &[u8]) {
    eprintln!("\n{GREEN}{s:?}{RESET} ", s = String::from_utf8_lossy(&bytes));
}

pub fn debug_transition(
    byte: u8,
    from: State,
    to: State,
    action: Action,
    exit: Action,
    entry: Action,
) {
    eprintln!(
        "{}",
        from_fn(|f| {
            // Byte
            write!(f, "{}", debug_byte(byte))?;

            if from != to {
                // State
                write!(f, " {BOLD}{BLUE} {to}{RESET}")?;

                if action.is_some() {
                    write!(f, " {MAGENTA}@ {action}{RESET}")?;
                }

                if entry.is_some() || exit.is_some() {
                    write!(f, " {FG_GREY}+ ")?;

                    if entry.is_some() && exit.is_some() {
                        write!(f, "{entry}..{exit}")?;
                    } else if entry.is_some() {
                        write!(f, "{entry}..")?;
                    } else if exit.is_some() {
                        write!(f, "..{exit}")?;
                    } else {
                        write!(f, "..")?;
                    }

                    write!(f, "{RESET}")?;
                }
            } else {
                write!(f, " {BOLD}{BLUE}..{RESET}")?;

                if action.is_some() {
                    write!(f, " {MAGENTA}@ {action}{RESET}")?;
                }
            }
            Ok(())
        })
    );
}

pub fn debug_print(bytes: &[u8], len: usize) {
    eprintln!(
        "{}",
        from_fn(|f| {
            let byte = bytes[0];
            write!(f, "{}", debug_byte(byte))?;

            // Byte
            write!(
                f,
                " {BOLD}{BLUE}{RESET}{:?} ",
                String::from_utf8_lossy(&bytes[..len]),
            )?;

            write!(f, "{MAGENTA}@ [..{}]{RESET}", len)
        })
    );
}
