extern crate proc_macro;

use proc_macro2::{token_stream, Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::iter::Peekable;

/// A single cell in the transition table: (action, target_state) or empty.
type Cell = Option<(TokenTree, TokenTree)>;

#[proc_macro]
pub fn transitions(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    let mut iter = item.into_iter().peekable();


    let mut states_iter = next_group(&mut iter).into_iter().peekable();

    let mut states: Vec<Ident> = Vec::new();
    let mut transitions: Vec<[Cell; 256]> = Vec::new();
    let mut entry_actions: Vec<Option<TokenTree>> = Vec::new();
    let mut exit_actions: Vec<Option<TokenTree>> = Vec::new();
    let mut anywhere: [Cell; 256] = [const { None }; 256];

    while states_iter.peek().is_some() {
        let ident = match states_iter.next() {
            Some(TokenTree::Ident(i)) => i,
            token => panic!("Expected ident, but got {:?}", token),
        };
        let is_anywhere = &ident == "Anywhere";

        let mut body = next_group(&mut states_iter).into_iter().peekable();
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

        optional_punct(&mut states_iter, ',');
    }

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

    // Emit literalt transition table.
    let transitions: Vec<_> = transitions
        .iter()
        .map(|cells| {
            let lines = cells.iter().map(|c| match c {
                Some((action, target)) => {
                    quote!(pack(Action::#action, State::#target))
                }
                None => 0u8.into_token_stream(),
            });

            quote!([#(#lines),*])
        })
        .collect();

    let entries = entry_actions.iter().map(|e| match e {
        Some(a) => quote!(Action::#a),
        None => quote!(Action::None),
    });

    let exits = exit_actions.iter().map(|e| match e {
        Some(a) => quote!(Action::#a),
        None => quote!(Action::None),
    });

    let asserts = states.iter().enumerate().map(|(i, state)| {
        let i = i as u8;
        let str = format!("State::{} does not match index {}.", state, i);
        quote!(assert!(State::#state as u8 == #i, #str))
    });

    quote!(
        const _: () = {
            #(#asserts);*
        };

        pub const TRANSITIONS: [[u8; 256]; State::COUNT] = [
            #(#transitions),*
        ];

        pub const ENTRY_ACTIONS: [Action; State::COUNT] = [
            #(#entries),*
        ];

        pub const EXIT_ACTIONS: [Action; State::COUNT] = [
            #(#exits),*
        ];
    )
    .into()
}

/// Parse `=> ActionIdent` and return the action token.
fn parse_action(iter: &mut Peekable<token_stream::IntoIter>) -> TokenTree {
    expect_punct(iter, '=');
    expect_punct(iter, '>');
    iter.next().unwrap()
}
fn optional_punct(iter: &mut Peekable<token_stream::IntoIter>, c: char) -> bool {
    match iter.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == c => iter.next().is_some(),
        _ => false,
    }
}

fn expect_punct(iter: &mut impl Iterator<Item = TokenTree>, c: char) {
    match iter.next() {
        Some(TokenTree::Punct(ref punct)) if punct.as_char() == c => (),
        token => panic!("Expected punctuation '{}', but got {:?}", c, token),
    }
}

fn next_usize(iter: &mut impl Iterator<Item = TokenTree>) -> usize {
    match iter.next() {
        Some(TokenTree::Literal(literal)) => {
            let literal = literal.to_string();
            if let Some(prefix) = literal.strip_prefix("0x") {
                usize::from_str_radix(prefix, 16).unwrap()
            } else {
                literal.parse::<usize>().unwrap()
            }
        }
        token => panic!("Expected literal, but got {:?}", token),
    }
}

fn next_group(iter: &mut impl Iterator<Item = TokenTree>) -> TokenStream {
    match iter.next() {
        Some(TokenTree::Group(group)) => group.stream(),
        token => panic!("Expected group, but got {:?}", token),
    }
}
