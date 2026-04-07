use std::io::{self, Write as _};
use ansi::{Color};
use geometry::{Bounded, Point, Rect};
use gloss::*;

fn main() -> io::Result<()> {
    let mut arena = Arena::new();
    let mut buffer = Buffer::new(44, 11);
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);
    let mut document = Document::new();
    let mut stdout = io::stdout();

    let root = document.node_mut(document.root);
    root.color = Some(Color::Green);

    root.background = Some(Color::Green);
    document.insert_with(
        Node::Span("Hello World!"),
        |node| {
            node.margin.top = Dimension::Length(1);
            node.border = BorderStyle::Bold;
            node.text_decoration = Some(TextDecoration::Underline);
            node.font_weight = Some(FontWeight::Bold);
            node.background = Some(Color::None);
        },
    );
    //
    // let row = document.insert_with(Node::Div(), |node| {
    //     node.set_color(Color::Red);
    //     node.set_border(Border::Bold);
    // });
    //
    // let a = document.insert_at_with(Node::Div(), At::Child(row), |node| {
    //     node.set_background(Color::Green);
    //     node.set_color(Color::None);
    // });
    //
    // let a_content = document.insert_at(Node::Span("A"), At::Child(a));
    //
    // let b = document.insert_at_with(Node::Div(), At::Child(row), |node| {
    //     node.set_background(Color::Yellow);
    //     node.set_color(Color::None);
    // });
    //
    // let b_content = document.insert_at(Node::Span("B"), At::Child(b));
    // let c = document.insert_at_with(Node::Div(), At::Child(row), |node| {
    //     node.set_background(Color::Blue);
    //     node.set_color(Color::None);
    // });
    //
    // let c_content = document.insert_at(Node::Span("C"), At::Child(c));

    document.compute_layout(Space::from(buffer.size()));
        let mut renderer = Painter::new(&mut buffer, &mut arena);

    renderer.with(|renderer| {
        renderer.foreground(Color::Blue);
        renderer.glyph = '-';
        renderer.rect(Rect::new(0, 0, 10, 5));
    });

    renderer.with(|renderer| {
        renderer.foreground(Color::Red);
        renderer.glyph = 'x';
        renderer.outline(Rect::new(0, 0, 10, 5));
    });
        // renderer.render(&document)?;

        rasterizer.raster(&buffer, &arena)?;
        rasterizer.flush(&mut stdout)?;

    Ok(())
}
