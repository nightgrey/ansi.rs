use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum ElementKind<'a> {
    Span(Cow<'a, str>),
    Div,
}
