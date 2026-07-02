use std::fmt::{self, Write};

/// A struct to display a byte slice for ANSI debugging.
///
/// Non-printable control characters (0x00..=0x1F, 0x7F) are displayed as
/// `0xNN` hex escapes. All other bytes are interpreted as UTF-8 and displayed
/// as characters. Invalid UTF-8 sequences are replaced with the Unicode
/// replacement character (U+FFFD).
pub struct DebugAnsi<'a> {
    data: &'a [u8],
}

impl<'a> DebugAnsi<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        DebugAnsi { data }
    }
}

impl fmt::Display for DebugAnsi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0;
        while i < self.data.len() {
            let byte = self.data[i];
            if byte.is_ascii() {
                if byte.is_ascii_control() {
                    write!(f, "0x{:02X}", byte)?;
                } else {
                    f.write_char(byte as char)?;
                }
                i += 1;
            } else {
                let len = match byte {
                    0b110_00000..=0b110_11111 => 2,
                    0b1110_0000..=0b1110_1111 => 3,
                    0b11110_000..=0b11110_111 => 4,
                    _ => 1,
                };
                let end = i + len;
                if end > self.data.len() {
                    f.write_char(char::REPLACEMENT_CHARACTER)?;
                    i += 1;
                } else {
                    match std::str::from_utf8(&self.data[i..end]) {
                        Ok(s) => {
                            f.write_char(s.chars().next().unwrap())?;
                        }
                        Err(_) => {
                            f.write_char(char::REPLACEMENT_CHARACTER)?;
                        }
                    }
                    i = end;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Debug for DebugAnsi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       fmt::Display::fmt(self, f)
    }
}

/// Like `DebugAnsi`, but non‑printable ASCII control characters (0x00‑0x1F, 0x7F)
/// are shown as `{AsciiName}` instead of `0xNN`.
///
/// # Example
/// ```
/// let data = b"\x1b[32mHello\x1b[0m\x80\xfe";
/// let named = DebugAnsiNamed::new(data);
/// assert_eq!(named.to_string(), "{Escape}[32mHello{Escape}[0m��");
/// ```
pub struct DebugAnsiNamed<'a> {
    data: &'a [u8],
}

impl<'a> DebugAnsiNamed<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        DebugAnsiNamed { data }
    }
}

/// Returns the `AsciiChar` variant name for a byte, if it is a control character.
fn ascii_control_name(byte: u8) -> Option<&'static str> {
    // Only the control block (0x00‑0x1F) and DELETE (0x7F) get a name.
    if !byte.is_ascii_control() {
        // 0x20‑0x7E are printable, no name needed.
        return None;
    }
    Some(match byte {
        0x00 => "Null",
        0x01 => "StartOfHeading",
        0x02 => "StartOfText",
        0x03 => "EndOfText",
        0x04 => "EndOfTransmission",
        0x05 => "Enquiry",
        0x06 => "Acknowledge",
        0x07 => "Bell",
        0x08 => "Backspace",
        0x09 => "CharacterTabulation",
        0x0A => "LineFeed",
        0x0B => "LineTabulation",
        0x0C => "FormFeed",
        0x0D => "CarriageReturn",
        0x0E => "ShiftOut",
        0x0F => "ShiftIn",
        0x10 => "DataLinkEscape",
        0x11 => "DeviceControlOne",
        0x12 => "DeviceControlTwo",
        0x13 => "DeviceControlThree",
        0x14 => "DeviceControlFour",
        0x15 => "NegativeAcknowledge",
        0x16 => "SynchronousIdle",
        0x17 => "EndOfTransmissionBlock",
        0x18 => "Cancel",
        0x19 => "EndOfMedium",
        0x1A => "Substitute",
        0x1B => "Escape",          // ← matches your request
        0x1C => "InformationSeparatorFour",
        0x1D => "InformationSeparatorThree",
        0x1E => "InformationSeparatorTwo",
        0x1F => "InformationSeparatorOne",
        0x7F => "Delete",
        _ => unreachable!("all ASCII control bytes are covered"),
    })
}

impl fmt::Display for DebugAnsiNamed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0;
        while i < self.data.len() {
            let byte = self.data[i];

            if byte.is_ascii() {
                // Only the control block (0x00‑0x1F) and DELETE (0x7F) get a name.
                if !byte.is_ascii_control() {
                    // 0x20‑0x7E are printable, no name needed.
                        f.write_char(byte as char)? ;
                }

                write!(f, "{{{}}}", match byte {
                    0x00 => "Null",
                    0x01 => "StartOfHeading",
                    0x02 => "StartOfText",
                    0x03 => "EndOfText",
                    0x04 => "EndOfTransmission",
                    0x05 => "Enquiry",
                    0x06 => "Acknowledge",
                    0x07 => "Bell",
                    0x08 => "Backspace",
                    0x09 => "CharacterTabulation",
                    0x0A => "LineFeed",
                    0x0B => "LineTabulation",
                    0x0C => "FormFeed",
                    0x0D => "CarriageReturn",
                    0x0E => "ShiftOut",
                    0x0F => "ShiftIn",
                    0x10 => "DataLinkEscape",
                    0x11 => "DeviceControlOne",
                    0x12 => "DeviceControlTwo",
                    0x13 => "DeviceControlThree",
                    0x14 => "DeviceControlFour",
                    0x15 => "NegativeAcknowledge",
                    0x16 => "SynchronousIdle",
                    0x17 => "EndOfTransmissionBlock",
                    0x18 => "Cancel",
                    0x19 => "EndOfMedium",
                    0x1A => "Substitute",
                    0x1B => "Escape",          // ← matches your request
                    0x1C => "InformationSeparatorFour",
                    0x1D => "InformationSeparatorThree",
                    0x1E => "InformationSeparatorTwo",
                    0x1F => "InformationSeparatorOne",
                    0x7F => "Delete",
                    _ => unreachable!("all ASCII control bytes are covered"),
                })?;
                i += 1;
            } else {
                // Multi‑byte UTF‑8 sequence handling.
                let len = (byte as char).len_utf8();
                let end = i + len;
                if end > self.data.len() {
                    // Truncated sequence → replacement character.
                    f.write_char(char::REPLACEMENT_CHARACTER)?;
                    i += 1;
                } else {
                    match std::str::from_utf8(&self.data[i..end]) {
                        Ok(s) => {
                            // Write the decoded character.
                            f.write_char(s.chars().next().unwrap())?;
                        }
                        Err(_) => {
                            // Invalid sequence → replacement character.
                            f.write_char(char::REPLACEMENT_CHARACTER)?;
                        }
                    }
                    i = end;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Debug for DebugAnsiNamed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DebugAnsiNamed")
            .field(&format_args!("{}", self))
            .finish()
    }
}