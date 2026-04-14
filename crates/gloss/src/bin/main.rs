use std::io::{self, Write as _};
use ansi::{Color};
use geometry::{Bounded, Point, Rect};
use gloss::*;
use tree::At;

fn main() -> io::Result<()> {
    let mut arena = Arena::new();
    let mut buffer = Buffer::new(40, 10);
    let mut rasterizer = Rasterer::inline(buffer.width, buffer.height);
    let mut document = Document::new();
    let mut stdout = io::stdout();

    let root = document.node_mut(document.root);
    root.background = Some(Color::Red);
    root.color = Some(Color::White);
    root.border = BorderStyle::Bold;

    document.insert_with(
        Node::Span("👨🏿👨🏿 Hello"),
        |node| {
            node.background = Some(Color::None);
            node.color = Some(Color::White);
            node.border = BorderStyle::Bold;
            node.font_weight = Some(FontWeight::Bold);
        },
    );

    let row = document.insert_with(Node::Div(), |node| {
        node.border = BorderStyle::Bold;
    });

    let a = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background = Some(Color::Green);
    });

    let a_content = document.insert_at(Node::Span("A"), At::Child(a));

    let b = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background = Some(Color::Yellow);
    });

    let b_content = document.insert_at(Node::Span("B"), At::Child(b));
    let c = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background = Some(Color::Blue);
    });

    let c_content = document.insert_at(Node::Span("C"), At::Child(c));

    document.compute_layout(Space::from(buffer.size()));
    let mut painter = BufferDrawingContext::new(&mut buffer, &mut arena).painter();

    painter.paint(&document);

    rasterizer.raster(&buffer, &arena)?;
    rasterizer.flush(&mut stdout)?;

    Ok(())
}
