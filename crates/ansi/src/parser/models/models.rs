use super::{DataString, Intermediates, Parameters};
use std::fmt;
use utils::Nested;

#[derive(Clone, PartialEq, Eq)]
pub struct Esc {
    pub intermediates: Intermediates,
    pub final_byte: u8,
}

impl fmt::Debug for Esc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\\x1B{:?}{:?}",
            self.intermediates, self.final_byte as char
        )
    }
}

impl fmt::Display for Esc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B{}{}", self.intermediates, self.final_byte as char)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Csi<const N: usize = 2> {
    pub params: Parameters<N>,
    pub intermediates: Intermediates,
    pub final_char: char,
}

impl fmt::Debug for Csi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\\x1B")?;
        for (pi, params) in self.params.iter().enumerate() {
            for (i, &param) in params.iter().enumerate() {
                write!(f, "{:?}", param)?;
                if i > 1 {
                    f.write_str(":")?;
                }
            }
            if pi > 0 {
                f.write_str(";")?;
            }
        }
        write!(f, "{}", self.final_char)
    }
}

impl fmt::Display for Csi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\x1B")?;
        for (pi, params) in self.params.iter().enumerate() {
            for (i, &param) in params.iter().enumerate() {
                write!(f, "{}", param)?;
                if i > 1 {
                    f.write_str(":")?;
                }
            }
            if pi > 0 {
                f.write_str(";")?;
            }
        }
        write!(f, "{}", self.final_char)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Dcs<const N: usize = 2> {
    pub params: Parameters<N>,
    pub intermediates: Intermediates,
    pub data: DataString,
}

impl fmt::Debug for Dcs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\\x1B{:?}{:?}", self.intermediates, self.data)
    }
}

impl fmt::Display for Dcs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B{}{}", self.intermediates, self.data)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Osc<const N: usize = 2> {
    pub params: Parameters<N>,
    pub intermediates: Intermediates,
    pub data: DataString,
}

impl fmt::Debug for Osc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\\x1B]{}{}", self.intermediates, self.data)
    }
}

impl fmt::Display for Osc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B]{}{}", self.intermediates, self.data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sos(pub DataString);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pm(pub DataString);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apc(pub DataString);
