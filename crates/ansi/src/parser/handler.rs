use utils::SmallByteString;

use super::Paras;
use std::bstr::ByteStr;
use std::char;

pub trait Handler {
    fn utf8(&mut self, char: char) {}
    fn control(&mut self, byte: u8) {}
    fn handle_csi(&mut self, params: Paras, intermediates: &[u8], final_char: char) {}
    fn handle_esc(&mut self, intermediates: &[u8], final_byte: u8) {}
    fn handle_dcs(&mut self, params: Paras, intermediates: &[u8], final_char: char, data: &[u8]) {}
    fn handle_osc(&mut self, params: Paras, intermediates: &[u8], data: &[u8]) {}
    fn handle_sos(&mut self, data: &[u8]) {}
    fn handle_pm(&mut self, data: &[u8]) {}
    fn handle_apc(&mut self, data: &[u8]) {}
}
