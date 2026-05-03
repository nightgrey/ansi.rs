/// Name of common ASCII control characters (NUL, BEL, ESC, DEL, etc.).
pub fn control(b: u8) -> Option<&'static str> {
    match b {
        0x00 => Some("NUL"),
        0x01 => Some("SOH"),
        0x02 => Some("STX"),
        0x03 => Some("ETX"),
        0x04 => Some("EOT"),
        0x05 => Some("ENQ"),
        0x06 => Some("ACK"),
        0x07 => Some("BEL"),
        0x08 => Some("BS"),
        0x09 => Some("TAB"),
        0x0A => Some("LF"),
        0x0B => Some("VT"),
        0x0C => Some("FF"),
        0x0D => Some("CR"),
        0x0E => Some("SO"),
        0x0F => Some("SI"),
        0x10 => Some("DLE"),
        0x11 => Some("DC1"),
        0x12 => Some("DC2"),
        0x13 => Some("DC3"),
        0x14 => Some("DC4"),
        0x15 => Some("NAK"),
        0x16 => Some("SYN"),
        0x17 => Some("ETB"),
        0x18 => Some("CAN"),
        0x19 => Some("EM"),
        0x1A => Some("SUB"),
        0x1B => Some("ESC"),
        0x1C => Some("FS"),
        0x1D => Some("GS"),
        0x1E => Some("RS"),
        0x1F => Some("US"),
        0x7F => Some("DEL"),
        _ => None,
    }
}

pub fn name(byte: u8) -> String {
    match control(byte) {
        Some(c) => c.to_string(),
        _ => byte.escape_ascii().to_string(),
    }
}

/// Name of common ASCII control characters (NUL, BEL, ESC, DEL, etc.).
pub fn named(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|&b| name(b)).collect()
}

pub fn hex(byte: u8) -> String {
    format!("0x{:02x}", byte)
}

pub fn hexed(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|&b| hex(b)).collect()
}

pub fn debug(byte: u8) -> String {
    format!("[{} / {}]", name(byte), hex(byte))
}
