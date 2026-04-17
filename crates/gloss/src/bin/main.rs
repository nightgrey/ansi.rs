use std::io::{self};
use ansi::{Color};
use gloss::*;
use tree::At;

fn main() -> io::Result<()> {
    let mut ui = Engine::new(20, 20);

    let root = ui.root_mut();
    root.background = Some(Color::Red);
    root.color = Some(Color::White);
    root.padding = 1.into();

    ui.insert_with(
        Element::Span("👨🏿👨🏿 Hello"),
        |node| {
            node.background = Some(Color::None);
            node.border = Border::Bold;
            node.font_weight = Some(FontWeight::Bold);
        },
    );

    let abc = ui.insert_with(Element::Div(), |node| {
        node.border = Border::Bold;
    });

    let a = ui.insert_at_with(Element::Div(), At::Child(abc), |node| {
        node.background = Some(Color::Green);
    });

    let b = ui.insert_at_with(Element::Div(), At::Child(abc), |node| {
        node.background = Some(Color::Yellow);
    });

    let c = ui.insert_at_with(Element::Div(), At::Child(abc), |node| {
        node.background = Some(Color::Blue);
    });

    ui.insert_at(Element::Span("A"), At::Child(a));
    ui.insert_at(Element::Span("B"), At::Child(b));
    ui.insert_at(Element::Span("C"), At::Child(c));

    ui.render(&mut io::stdout())?;


    Ok(())
}
