use derive_more::FromStrError;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Control {
    NUL = 0,
    SOH = 1,
    STX = 2,
    ETX = 3,
    EOT = 4,
    ENQ = 5,
    ACK = 6,
    BEL = 7,
    BS = 8,
    TAB = 9,
    LF = 10,
    VT = 11,
    FF = 12,
    CR = 13,
    SO = 14,
    SI = 15,
    DLE = 16,
    DC1 = 17,
    DC2 = 18,
    DC3 = 19,
    DC4 = 20,
    NAK = 21,
    SYN = 22,
    ETB = 23,
    CAN = 24,
    EM = 25,
    SUB = 26,
    ESC = 27,
    FS = 28,
    GS = 29,
    RS = 30,
    US = 31,
    DEL = 127,
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::NUL => f.write_str("NU"),
            Control::SOH => f.write_str("SO"),
            Control::STX => f.write_str("ST"),
            Control::ETX => f.write_str("ET"),
            Control::EOT => f.write_str("EO"),
            Control::ENQ => f.write_str("EN"),
            Control::ACK => f.write_str("AC"),
            Control::BEL => f.write_str("BE"),
            Control::BS => f.write_str("BS"),
            Control::TAB => f.write_str("TA"),
            Control::LF => f.write_str("LF"),
            Control::VT => f.write_str("VT"),
            Control::FF => f.write_str("FF"),
            Control::CR => f.write_str("CR"),
            Control::SO => f.write_str("SO"),
            Control::SI => f.write_str("SI"),
            Control::DLE => f.write_str("DL"),
            Control::DC1 => f.write_str("DC"),
            Control::DC2 => f.write_str("DC"),
            Control::DC3 => f.write_str("DC"),
            Control::DC4 => f.write_str("DC"),
            Control::NAK => f.write_str("NA"),
            Control::SYN => f.write_str("SY"),
            Control::ETB => f.write_str("ET"),
            Control::CAN => f.write_str("CA"),
            Control::EM => f.write_str("EM"),
            Control::SUB => f.write_str("SU"),
            Control::ESC => f.write_str("ES"),
            Control::FS => f.write_str("FS"),
            Control::GS => f.write_str("GS"),
            Control::RS => f.write_str("RS"),
            Control::US => f.write_str("US"),
            Control::DEL => f.write_str("DE"),
        }
    }
}

impl std::fmt::Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Same as Display – reuse it to avoid duplication
        std::fmt::Display::fmt(self, f)
    }
}

impl TryFrom<u8> for Control {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Control::NUL),
            1 => Ok(Control::SOH),
            2 => Ok(Control::STX),
            3 => Ok(Control::ETX),
            4 => Ok(Control::EOT),
            5 => Ok(Control::ENQ),
            6 => Ok(Control::ACK),
            7 => Ok(Control::BEL),
            8 => Ok(Control::BS),
            9 => Ok(Control::TAB),
            10 => Ok(Control::LF),
            11 => Ok(Control::VT),
            12 => Ok(Control::FF),
            13 => Ok(Control::CR),
            14 => Ok(Control::SO),
            15 => Ok(Control::SI),
            16 => Ok(Control::DLE),
            17 => Ok(Control::DC1),
            18 => Ok(Control::DC2),
            19 => Ok(Control::DC3),
            20 => Ok(Control::DC4),
            21 => Ok(Control::NAK),
            22 => Ok(Control::SYN),
            23 => Ok(Control::ETB),
            24 => Ok(Control::CAN),
            25 => Ok(Control::EM),
            26 => Ok(Control::SUB),
            27 => Ok(Control::ESC),
            28 => Ok(Control::FS),
            29 => Ok(Control::GS),
            30 => Ok(Control::RS),
            31 => Ok(Control::US),
            127 => Ok(Control::DEL),
            _ => Err(()),
        }
    }
}

impl TryFrom<char> for Control {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Control::try_from(value as u8)
    }
}

impl std::str::FromStr for Control {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Match against uppercase names only (the canonical form)
        match s.to_ascii_uppercase().as_str() {
            "NUL" => Ok(Control::NUL),
            "SOH" => Ok(Control::SOH),
            "STX" => Ok(Control::STX),
            "ETX" => Ok(Control::ETX),
            "EOT" => Ok(Control::EOT),
            "ENQ" => Ok(Control::ENQ),
            "ACK" => Ok(Control::ACK),
            "BEL" => Ok(Control::BEL),
            "BS" => Ok(Control::BS),
            "TAB" => Ok(Control::TAB),
            "LF" => Ok(Control::LF),
            "VT" => Ok(Control::VT),
            "FF" => Ok(Control::FF),
            "CR" => Ok(Control::CR),
            "SO" => Ok(Control::SO),
            "SI" => Ok(Control::SI),
            "DLE" => Ok(Control::DLE),
            "DC1" => Ok(Control::DC1),
            "DC2" => Ok(Control::DC2),
            "DC3" => Ok(Control::DC3),
            "DC4" => Ok(Control::DC4),
            "NAK" => Ok(Control::NAK),
            "SYN" => Ok(Control::SYN),
            "ETB" => Ok(Control::ETB),
            "CAN" => Ok(Control::CAN),
            "EM" => Ok(Control::EM),
            "SUB" => Ok(Control::SUB),
            "ESC" => Ok(Control::ESC),
            "FS" => Ok(Control::FS),
            "GS" => Ok(Control::GS),
            "RS" => Ok(Control::RS),
            "US" => Ok(Control::US),
            "DEL" => Ok(Control::DEL),
            _ => Err(FromStrError::new("Control")),
        }
    }
}

impl const PartialEq<u8> for Control {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}

impl const PartialEq<Control> for u8 {
    fn eq(&self, other: &Control) -> bool {
        PartialEq::eq(other, self)
    }
}

impl const PartialEq<char> for Control {
    fn eq(&self, other: &char) -> bool {
        PartialEq::eq(self, &(*other as u8))
    }
}

impl const PartialEq<Control> for char {
    fn eq(&self, other: &Control) -> bool {
        PartialEq::eq(other, &(*self as u8))
    }
}
