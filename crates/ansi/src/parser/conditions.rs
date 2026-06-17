
const NUL: u8 = 0;
const SOH: u8 = 1;
const STX: u8 = 2;
const ETX: u8 = 3;
const EOT: u8 = 4;
const ENQ: u8 = 5;
const ACK: u8 = 6;
const BEL: u8 = 7;
const BS: u8 = 8;
const TAB: u8 = 9;
const LF: u8 = 10;
const VT: u8 = 11;
const FF: u8 = 12;
const CR: u8 = 13;
const SO: u8 = 14;
const SI: u8 = 15;
const DLE: u8 = 16;
const DC1: u8 = 17;
const DC2: u8 = 18;
const DC3: u8 = 19;
const DC4: u8 = 20;
const NAK: u8 = 21;
const SYN: u8 = 22;
const ETB: u8 = 23;
const CAN: u8 = 24;
const EM: u8 = 25;
const SUB: u8 = 26;
const ESC: u8 = 27;
const FS: u8 = 28;
const GS: u8 = 29;
const RS: u8 = 30;
const US: u8 = 31;
const DEL: u8 = 127;

macro_rules! table {
    (|$p:ident| $body:block) => {{
        let mut out: [bool; 256] = [false; 256];
        let mut i = 0;
        while i < 256 {
            let $p: u8 = i as u8;
            out[i] = $body;
            i += 1;
        }
        out
    }};
}


pub const fn is_end_of_csi(byte: u8) -> bool {
    static TABLE: [bool; 256] = table!(|b| { matches!(b, 0x40..=0x7e | ESC | CAN | SUB) });

    TABLE[byte as usize]
}

pub const fn is_end_of_ground(byte: u8) -> bool {
    static TABLE: [bool; 256] = table!(|b| { matches!(b, 0x1B | 0x00..=0x08 | 0x0b..=0x0c | 0x0e..=0x1f | DEL) });

    TABLE[byte as usize]
}
