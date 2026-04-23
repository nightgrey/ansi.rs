use std::bstr::ByteStr;
use super::{Inter, Params};
use std::char;
use crate::parser::{DataStr, FinalByte, FinalChar};

pub trait Handler {
    fn utf8(&mut self, char: char) {}
    fn control(&mut self, byte: FinalByte) {}
    fn handle_csi(&mut self, params: Params, intermediates: &Inter, final_char: FinalChar) {}
    fn handle_esc(&mut self, intermediates: &Inter, final_byte: FinalByte) {}
    fn handle_dcs(&mut self, params: Params, intermediates: &Inter, final_char: FinalChar, data: &DataStr) {}
    fn handle_osc(&mut self, params: Params, intermediates: &Inter, data: &DataStr) {}
    fn handle_sos(&mut self, data: &DataStr) {}
    fn handle_pm(&mut self, data: &DataStr) {}
    fn handle_apc(&mut self, data: &DataStr) {}
}
