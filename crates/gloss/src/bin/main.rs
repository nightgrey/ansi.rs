use std::io::{self};
use ansi::{Color};
use gloss::*;
use tree::At;

fn main() -> io::Result<()> {
    let mut ui = Engine::new(40, 10);

    let root = ui.root_mut();
    root.background = Some(Color::Red);
    root.color = Some(Color::White);
    root.border = BorderStyle::Bold;

    ui.insert_with(
        Node::Span("👨🏿👨🏿 Hello"),
        |node| {
            node.background = Some(Color::None);
            node.color = Some(Color::White);
            node.border = BorderStyle::Bold;
            node.font_weight = Some(FontWeight::Bold);
        },
    );

    let row = ui.insert_with(Node::Div(), |node| {
        node.border = BorderStyle::Bold;
    });

    let a = ui.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background = Some(Color::Green);
    });

    let a_content = ui.insert_at(Node::Span("A"), At::Child(a));

    let b = ui.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background = Some(Color::Yellow);
    });

    let b_content = ui.insert_at(Node::Span("B"), At::Child(b));
    let c = ui.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background = Some(Color::Blue);
    });

    let c_content = ui.insert_at(Node::Span("C"), At::Child(c));

    ui.render(&mut io::stdout())?;


    Ok(())
}
