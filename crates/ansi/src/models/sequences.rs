use crate::{ByteString, Intermediate, Intermediates, Parameters, Params};

pub struct Esc {
    pub intermediates: Intermediates,
    pub final_byte: u8,
}

pub struct Csi {
    pub intermediates: Intermediates,
    pub params: Parameters,
    pub final_char: char,
}

pub struct Dcs {
    pub intermediates: Intermediates,
    pub params: Parameters,
    pub data: ByteString,
    pub final_char: char,
}

pub struct Osc {
    pub intermediates: Intermediates,
    pub params: Parameters,
    pub data: ByteString,
    pub final_char: char,
}

pub enum Sequence {
    Esc(Esc),
    Csi(Csi),
    Dcs(Dcs),
    Osc(Osc),
}
