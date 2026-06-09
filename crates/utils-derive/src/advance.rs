extern crate proc_macro;

use std::iter::Peekable;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote};
use std::str::FromStr;
use crate::utils::*;

/// A single cell in the transition table: (action, target_state) or empty.
type Cell = Option<(TokenTree, TokenTree)>;

fn hex_byte_literal(byte: u8) -> proc_macro2::Literal {
    proc_macro2::Literal::from_str(&format!("0x{:02X}", byte)).unwrap()
}

pub fn advance_inner(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    let iter = item.into_iter().peekable();

    let (states, mut transitions, entry_actions, exit_actions, anywhere) = parse(iter);

    // Merge Anywhere into every state. per-state takes priority over Anywhere.
    for state_transitions in transitions.iter_mut() {
        for (byte, anywhere_transition) in anywhere.iter().enumerate() {
            if let Some(anywhere) = anywhere_transition {
                if state_transitions[byte].is_none() {
                    state_transitions[byte] = Some(anywhere.clone());
                }
            }
        }
    }

    // ── Generate match arms for `advance` (returns only state + action) ──
    let match_arms = states
        .iter()
        .zip(transitions.iter())
        .map(|(state, cells)| {
            let arms = generate_byte_range_arms(cells);

            let state_token = quote!(State::#state);
            quote! {
                #state_token => match byte {
                    #(#arms),*
                },
            }
        }).collect::<Vec<_>>();

    // ── Generate `entry(state)` and `exit(state)` functions ──
    let entry_arms = states
        .iter()
        .zip(entry_actions.iter())
        .map(|(state, action_opt)| {
            let action = match action_opt {
                Some(a) => quote!(Action::#a),
                None => quote!(Action::None),
            };
            quote!(State::#state => #action)
        }).collect::<Vec<_>>();

    let exit_arms = states
        .iter()
        .zip(exit_actions.iter())
        .map(|(state, action_opt)| {
            let action = match action_opt {
                Some(a) => quote!(Action::#a),
                None => quote!(Action::None),
            };
            quote!(State::#state => #action)
        }).collect::<Vec<_>>();

    quote!(
        /// Advance the state machine given current state and byte input.
        /// Returns a `Transition` containing the next state and the transition action.
        /// Use `entry(state)` / `exit(state)` to obtain the per‑state actions.
        pub fn advance(state: State, byte: u8) -> Transition {
            match state {
                #(#match_arms,)*
                State::None => Transition::new(State::None, Action::None),
            }
        }

        /// Returns the entry action for the given state.
        pub fn entry(state: State) -> Action {
            match state {
                #(#entry_arms,)*
                _ => Action::None,
            }
        }

        /// Returns the exit action for the given state.
        pub fn exit(state: State) -> Action {
            match state {
                #(#exit_arms,)*
                _ => Action::None,
            }
        }

    ).into()
}


pub fn parse(mut iter: Peekable<proc_macro2::token_stream::IntoIter>) -> (Vec<Ident>, Vec<[Cell; 256]>, Vec<Option<TokenTree>>, Vec<Option<TokenTree>>, [Cell; 256]) {

    let mut states: Vec<Ident> = Vec::new();
    let mut transitions: Vec<[Cell; 256]> = Vec::new();
    let mut entry_actions: Vec<Option<TokenTree>> = Vec::new();
    let mut exit_actions: Vec<Option<TokenTree>> = Vec::new();
    let mut anywhere: [Cell; 256] = [const { None }; 256];

    while iter.peek().is_some() {
        let ident = match iter.next() {
            Some(TokenTree::Ident(i)) => i,
            token => panic!("Expected ident, but got {:?}", token),
        };
        let is_anywhere = &ident == "Anywhere";

        let mut body = next_group(&mut iter).into_iter().peekable();
        let mut cells: [Cell; 256] = [const { None }; 256];
        let mut entry: Option<TokenTree> = None;
        let mut exit: Option<TokenTree> = None;

        while body.peek().is_some() {
            let kw = match body.peek() {
                Some(TokenTree::Ident(i)) => Some(i.to_string()),
                _ => None,
            };
            match kw.as_deref() {
                Some("on_entry") => {
                    if is_anywhere {
                        panic!("`on_entry` not allowed on `Anywhere`");
                    }
                    body.next();
                    entry = Some(parse_action(&mut body));
                }
                Some("on_exit") => {
                    if is_anywhere {
                        panic!("`on_exit` not allowed on `Anywhere`");
                    }
                    body.next();
                    exit = Some(parse_action(&mut body));
                }
                Some(other) => {
                    panic!("Unexpected identifier `{}` in state body", other);
                }
                None => {
                    let body = &mut body;
                    let start = next_usize(body);
                    let end = if optional_punct(body, '.') {
                        expect_punct(body, '.');
                        expect_punct(body, '=');
                        next_usize(body)
                    } else {
                        start
                    };

                    expect_punct(body, '=');
                    expect_punct(body, '>');

                    let (action, target) = match body.next() {
                        Some(TokenTree::Group(group)) => {
                            let mut t = group.stream().into_iter().peekable();
                            let action = t.next().unwrap();
                            expect_punct(&mut t, ',');
                            let target = t.next().unwrap();
                            (action, target)
                        }
                        Some(TokenTree::Ident(ident)) => {
                            if is_anywhere {
                                panic!(
                                    "`Anywhere` transitions must specify a target state; \
                     `0x{:x}..=0x{:x} => {}` is action-only",
                                    start, end, ident,
                                );
                            }
                            let action = TokenTree::Ident(ident);
                            let target = TokenTree::Ident(Ident::new("None", Span::call_site()));
                            (action, target)
                        }
                        token => panic!("Expected group or ident, but got {:?}", token),
                    };

                    for byte in start..=end {
                        cells[byte] = Some((action.clone(), target.clone()));
                    }
                }
            }
            optional_punct(&mut body, ',');
        }

        if is_anywhere {
            for (i, c) in cells.into_iter().enumerate() {
                if c.is_some() {
                    anywhere[i] = c;
                }
            }
        } else {
            states.push(ident);
            transitions.push(cells);
            entry_actions.push(entry);
            exit_actions.push(exit);
        }

        optional_punct(&mut iter, ',');
    }

    (states, transitions, entry_actions, exit_actions, anywhere)
}
/// Generate match arms for byte ranges instead of individual bytes.
/// Consolidates consecutive bytes with the same transition into ranges.
fn generate_byte_range_arms(
    cells: &[Cell; 256],
) -> Vec<TokenStream> {
    let mut arms = Vec::new();
    let mut i = 0;

    while i < 256 {
        if let Some((action, target)) = &cells[i] {
            let start = i as u8;
            let mut end = i as u8;

            // Find the end of consecutive bytes with same action and target
            while end < 255 {
                let next_idx = (end as usize) + 1;
                if let Some((next_action, next_target)) = &cells[next_idx] {
                    if action.to_string() == next_action.to_string()
                        && target.to_string() == next_target.to_string()
                    {
                        end = next_idx as u8;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let action_token = match action {
                TokenTree::Ident(i) => quote!(Action::#i),
                _ => unreachable!(),
            };
            let target_token = match target {
                TokenTree::Ident(i) => {
                    if i == "None" {
                        quote!(State::None)
                    } else {
                        quote!(State::#i)
                    }
                }
                _ => unreachable!(),
            };

            let start_hex = hex_byte_literal(start);
            if start == end {
                arms.push(quote!(
                    #start_hex => Transition::new(#target_token, #action_token)
                ));
            } else {
                let end_hex = hex_byte_literal(end);
                arms.push(quote!(
                    #start_hex ..= #end_hex => Transition::new(#target_token, #action_token)
                ));
            }

            i = end as usize + 1;
        } else {
            i += 1;
        }
    }

    arms.push(quote! {
        _ => Transition::new(State::None, Action::None)
    });

    arms
}