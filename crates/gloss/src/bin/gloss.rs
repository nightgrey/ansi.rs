use std::borrow::Cow;
use std::io::{self, Write as _};
use ansi::{Color};
use geometry::Bounded;
use gloss::*;
use sigil::{Buffer, Arena, Rasterizer};
use tree::At;

fn main() -> io::Result<()> {
    let mut arena = Arena::new();
    let mut buffer = Buffer::new(44, 44);
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);
    let mut document = Document::new();
    let mut stdout = io::stdout();

    let root = document.node_mut(document.root);
    root.set_display(Display::Flex);
    root.set_flex_direction(FlexDirection::Column);

    let title = document.insert_with(
        Node::Span(Cow::Borrowed("Hello World!")),
        |node| {
            node.set_border(Border::Solid);
            node.set_color(Color::Red);
            node.set_text_decoration(TextDecoration::Underline);
            node.set_font_weight(FontWeight::Bold);
            node.set_background(Color::White);
        },
    );

    let row = document.insert_with(Node::Div(), |node| {
        node.set_color(Color::Red);
        node.set_border(Border::Bold);
    });
    let a = document.insert_with_children_at_with(Node::Div(), [Node::Span(Cow::Borrowed("A"))], At::Child(row), |node| {
        node.set_background(Color::Green);
        node.set_color(Color::None);


    });

    let b = document.insert_with_children_at_with(Node::Div(), [Node::Span(Cow::Borrowed("B"))], At::Child(row), |node| {
        node.set_background(Color::Yellow);
    });

    let c = document.insert_with_children_at_with(Node::Div(), [Node::Span(Cow::Borrowed("C"))], At::Child(row), |node| {
        node.set_background(Color::Blue);
    });


    document.compute_layout(Space::from(buffer.size()));
    let mut renderer = Renderer::new(&mut buffer, &mut arena);

        renderer.render(&document)?;

        rasterizer.raster(&buffer, &arena)?;
        rasterizer.flush(&mut stdout)?;

    Ok(())
}
