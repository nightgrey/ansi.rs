use crate::parser::{Inter, Params};

pub trait Handler {
    /// Draw a character to the screen and update states.
    fn print(&mut self, byte: char) {}

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, byte: u8) {}

    /// The final character of an escape sequence has arrived.
    fn esc(&mut self, intermediates: &Inter, final_byte: u8) {}

    /// A final character has arrived for a CSI sequence.
    fn csi(&mut self, params: Params<'_>, intermediates: &Inter, final_byte: char) {}

    /// Invoked when a final character arrives in first part of device control
    /// string. Subsequent bytes in the control string are delivered via
    /// [`Handler::dcs_byte`], and termination via [`Handler::dcs_termination`].
    fn dcs(&mut self, params: Params<'_>, intermediates: &Inter, final_char: char) {}

    /// A byte of a DCS data string. C0 controls are also passed here.
    fn dcs_byte(&mut self, byte: u8) {}

    /// The DCS data string has been terminated.
    fn dcs_termination(&mut self, byte: u8) {}

    /// Begin an operating system command. Subsequent body bytes are delivered
    /// via [`Handler::osc_byte`]; termination via [`Handler::osc_termination`].
    fn osc(&mut self, params: Params<'_>) {}

    /// A byte of OSC data.
    fn osc_byte(&mut self, byte: u8) {}

    /// The OSC string has been terminated.
    fn osc_termination(&mut self, byte: u8) {}
}
