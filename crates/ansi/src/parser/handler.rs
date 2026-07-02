use crate::Params;

pub trait Handler {
    fn print(&mut self, bytes: &[u8]) { }

    fn control(&mut self, byte: u8) {}

    fn esc(&mut self, intermediates: &[u8], final_byte: u8) {}

    fn csi(&mut self, params: &Params, intermediates: &[u8], final_char: char) {}

    fn dcs(&mut self, params: &Params, intermediates: &[u8], final_char: char) {}

    fn dcs_data(&mut self, bytes: &[u8]) {}

    fn dcs_end(&mut self, byte: u8) {}

    fn osc(&mut self) {}

    fn osc_data(&mut self, bytes: &[u8]) {}

    fn osc_end(&mut self, byte: u8) {}
}

pub trait Utf8Handler: Handler {
    fn char(&mut self, char: char) {}
}