use crate::parser::{ByteStr, Params};

pub trait Handler {
    /// Draw a character to the screen and update states.
    fn printable(&mut self, _byte: char) {}

    /// Draw a run of printable characters in a single call. The parser batches
    /// contiguous printable text between control bytes and hands it over here.
    /// Defaults to dispatching each char individually via [`Handler::printable`];
    /// override it to avoid per-char dispatch on text-heavy streams.
    fn printables(&mut self, str: &str) {
        for ch in str.chars() {
            self.printable(ch);
        }
    }

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, _byte: u8) {}

    /// The final character of an escape sequence has arrived.
    fn esc(&mut self, _intermediates: &ByteStr, _final_byte: u8) {}

    /// A final character has arrived for a CSI sequence.
    fn csi(&mut self, _params: Params<'_>, _intermediates: &ByteStr, _final_byte: char) {}

    /// Invoked when a final character arrives in first part of device control
    /// string. Subsequent bytes in the control string are delivered via
    /// [`Handler::dcs_byte`], and termination via [`Handler::dcs_termination`].
    fn dcs(&mut self, _params: Params<'_>, _intermediates: &ByteStr, _final_char: char) {}

    /// A byte of a DCS data string. C0 controls are also passed here.
    fn dcs_byte(&mut self, _byte: u8) {}

    /// The DCS data string has been terminated.
    fn dcs_termination(&mut self, _byte: u8) {}

    /// Begin an operating system command. Subsequent body bytes are delivered
    /// via [`Handler::osc_byte`]; termination via [`Handler::osc_termination`].
    fn osc(&mut self) {}

    /// A byte of OSC data.
    fn osc_byte(&mut self, _byte: u8) {}

    /// The OSC string has been terminated.
    fn osc_termination(&mut self, _byte: u8) {}
}
