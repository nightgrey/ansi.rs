extern crate proc_macro;

mod advance;
mod transition;
mod utils;
mod state_machine;

use advance::*;
use transition::*;


use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote};
use syn::{
    bracketed, braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Expr, Ident, LitInt, Path, Result, Token,
};

#[proc_macro]
pub fn transitions(item: TokenStream) -> TokenStream {
    transitions_inner(item)
}

#[proc_macro]
pub fn advance(item: TokenStream) -> TokenStream {
    advance_inner(item)
}

#[proc_macro]
pub fn state_machine(input: TokenStream) -> TokenStream {
    let machine = parse_macro_input!(input as state_machine::Machine);

    match state_machine::expand(machine) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
