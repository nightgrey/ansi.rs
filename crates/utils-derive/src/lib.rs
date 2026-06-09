extern crate proc_macro;

mod advance;
mod transition;
mod utils;

use advance::*;
use transition::*;
use utils::*;

#[proc_macro]
pub fn transitions(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    transitions_inner(item)
}

#[proc_macro]
pub fn advance(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    advance_inner(item)
}
