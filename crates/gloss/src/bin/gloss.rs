use std::borrow::Cow;
use std::io::{self, Write as _};
use bon::__::ide::builder_top_level::start_fn::doc;
use ansi::{Color, EraseDisplay, EraseDisplayToEnd, Home, Style, SGR};
use gloss::*;
use sigil::{ Buffer, GraphemeArena, Rasterizer};
use tree::At;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let mut document = Document::new();

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
        node.set_width(Dimension::MAX);
        node.set_color(Color::White);
    });

    let a = document.insert_at_with_children(Node::Div(), [Node::Span(Cow::Borrowed("A"))], At::Child(row), |node| {
        node.set_background(Color::Green);

    });

    let b = document.insert_at_with_children(Node::Div(), [Node::Span(Cow::Borrowed("B"))], At::Child(row), |node| {
        node.set_background(Color::Yellow);
    });

    let c = document.insert_at_with_children(Node::Div(), [Node::Span(Cow::Borrowed("C"))], At::Child(row), |node| {
        node.set_background(Color::Blue);
    });

    document.insert_at_with_children(Node::Span(Cow::Borrowed("D")), [Node::Span(Cow::Borrowed("D"))], At::Child(c), |node| {
        node.set_background(Color::Red);
    });

    let mut arena = GraphemeArena::new();
    let mut buffer = Buffer::new(20, 30);
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);

    document.compute_layout(Space::new(buffer.width, buffer.height));

    let mut renderer = BufferRenderer::new(&mut buffer, &mut arena);

    renderer.render(&document)?;
    rasterizer.raster(&buffer, &arena)?;
    rasterizer.write(&mut stdout)?;

    dbg!(document.print_layout());
    dbg!(&rasterizer.as_str());
    Ok(())
}
