extern crate proc_macro;

use std::iter::Peekable;
use proc_macro2::{token_stream, Span, Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};

/// A single cell in the transition table: (action, target_state) or empty.
type Cell = Option<(TokenTree, TokenTree)>;

#[proc_macro]
pub fn transitions(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    let mut iter = item.into_iter().peekable();

    let n = next_usize(&mut iter);
    expect_punct(&mut iter, ',');

    let mut states_iter = next_group(&mut iter).into_iter().peekable();

    let mut state_names: Vec<TokenTree> = Vec::new();
    let mut state_cells: Vec<[Cell; 256]> = Vec::new();
    let mut entry_actions: Vec<Option<TokenTree>> = Vec::new();
    let mut exit_actions: Vec<Option<TokenTree>> = Vec::new();
    let mut anywhere: [Cell; 256] = [const { None }; 256];

    while states_iter.peek().is_some() {
        let state_name = states_iter.next().unwrap();
        let is_anywhere = matches!(
            &state_name,
            TokenTree::Ident(i) if i.to_string() == "Anywhere"
        );

        if matches!(
            &state_name,
            TokenTree::Ident(i) if i.to_string() == "None"
        ) {
            continue;
        }

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
                },
                Some("on_exit") => {
                    if is_anywhere {
                        panic!("`on_exit` not allowed on `Anywhere`");
                    }
                    body.next();
                    exit = Some(parse_action(&mut body));
                },
                Some(other) => {
                    panic!("Unexpected identifier `{}` in state body", other);
                },
                None => {
                    parse_transition(&mut body, is_anywhere, &mut cells);
                },
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
            state_names.push(state_name);
            state_cells.push(cells);
            entry_actions.push(entry);
            exit_actions.push(exit);
        }

        optional_punct(&mut states_iter, ',');
    }

    // Merge Anywhere into every state. Per Ruby semantics: per-state wins,
    // and a collision is an error (matches the original `raise`).
    for (idx, cells) in state_cells.iter_mut().enumerate() {
        for (byte, ac) in anywhere.iter().enumerate() {
            if let Some(ac) = ac {
                if cells[byte].is_some() {
                    let name = match &state_names[idx] {
                        TokenTree::Ident(i) => i.to_string(),
                        t => format!("{:?}", t),
                    };
                    panic!(
                        "State `{}` already defines transition for 0x{:02x}, \
                         but Anywhere also defines one",
                        name, byte,
                    );
                }
                cells[byte] = Some(ac.clone());
            }
        }
    }

    // Emit literal rows.
    let rows: Vec<TokenStream> = state_cells
        .iter()
        .map(|cells| {
            let exprs = cells.iter().map(|c| match c {
                Some((action, target)) => quote!(
                    Action::#action as u8 | ((State::#target as u8) << 4)
                ),
                None => quote!(0u8),
            });
            quote!([#(#exprs),*])
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

    quote!(
        pub const TRANSITIONS: [[u8; 256]; State::COUNT - 1] = [
            #(#rows),*
        ];

        pub const ENTRY_ACTIONS: [Action; State::COUNT - 1] = [
            #(#entries),*
        ];

        pub const EXIT_ACTIONS: [Action; State::COUNT - 1] = [
            #(#exits),*
        ];
    )
        .into()
}


fn parse_transition(
    iter: &mut Peekable<token_stream::IntoIter>,
    is_anywhere: bool,
    cells: &mut [Cell; 256],
) {
    let start = next_usize(iter);
    let end = if optional_punct(iter, '.') {
        expect_punct(iter, '.');
        expect_punct(iter, '=');
        next_usize(iter)
    } else {
        start
    };

    expect_punct(iter, '=');
    expect_punct(iter, '>');

    let (action, target) = match iter.next() {
        Some(TokenTree::Group(group)) => {
            let mut t = group.stream().into_iter().peekable();
            let action = t.next().unwrap();
            expect_punct(&mut t, ',');
            let target = t.next().unwrap();
            (action, target)
        },
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
        },
        token => panic!("Expected group or ident, but got {:?}", token),
    };

    for byte in start..=end {
        cells[byte] = Some((action.clone(), target.clone()));
    }
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
        },
        token => panic!("Expected literal, but got {:?}", token),
    }
}

fn next_group(iter: &mut impl Iterator<Item = TokenTree>) -> TokenStream {
    match iter.next() {
        Some(TokenTree::Group(group)) => group.stream(),
        token => panic!("Expected group, but got {:?}", token),
    }
}