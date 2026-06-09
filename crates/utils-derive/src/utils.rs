use proc_macro2::{TokenStream, TokenTree, token_stream};
use std::iter::Peekable;

/// Parse `=> ActionIdent` and return the action token.
pub fn parse_action(iter: &mut Peekable<token_stream::IntoIter>) -> TokenTree {
    expect_punct(iter, '=');
    expect_punct(iter, '>');
    iter.next().unwrap()
}

pub fn optional_punct(iter: &mut Peekable<token_stream::IntoIter>, c: char) -> bool {
    match iter.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == c => iter.next().is_some(),
        _ => false,
    }
}

pub fn expect_punct(iter: &mut impl Iterator<Item = TokenTree>, c: char) {
    match iter.next() {
        Some(TokenTree::Punct(ref punct)) if punct.as_char() == c => (),
        token => panic!("Expected punctuation '{}', but got {:?}", c, token),
    }
}

pub fn next_usize(iter: &mut impl Iterator<Item = TokenTree>) -> usize {
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

pub fn next_group(iter: &mut impl Iterator<Item = TokenTree>) -> TokenStream {
    match iter.next() {
        Some(TokenTree::Group(group)) => group.stream(),
        token => panic!("Expected group, but got {:?}", token),
    }
}
