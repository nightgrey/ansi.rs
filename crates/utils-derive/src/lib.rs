extern crate proc_macro;

mod vt100;

use proc_macro::TokenStream;

#[proc_macro]
pub fn transition(item: TokenStream) -> TokenStream {
    vt100::transition_fn(item)
}

#[proc_macro]
pub fn state_machine(input: TokenStream) -> TokenStream {
    vt100::state_machine_fn(input)
}
