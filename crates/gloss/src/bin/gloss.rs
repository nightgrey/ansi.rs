use std::borrow::Cow;
use std::io;
use bon::__::ide::builder_top_level::start_fn::doc;
use ansi::{Color, Style};
use gloss::*;
use sigil::{ Buffer, GraphemeArena, Rasterizer};
use tree::At;

fn main() {
    let mut document = Document::new();

    let root = document.node_mut(document.root);
    root.set_display(Display::Flex);
    root.set_flex_direction(FlexDirection::Column);

    let title = document.insert_with(
        Node::Span(Cow::Borrowed("Hello World!")),
        |node| {
            node.set_border(Border::Solid);
            node.set_background_color(Color::White);
        },
    );

    let row = document.insert_with(Node::Div(), |node| {
        node.set_flex_direction(FlexDirection::Row);
        node.set_width(Dimension::Percent(1.0));
    });

    let a = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.set_background_color(Color::Green);
    });

    let b = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.set_background_color(Color::Yellow);
    });

    let c = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.set_background_color(Color::Blue);
    });

    let mut arena = GraphemeArena::new();
    let mut buffer = Buffer::new(20, 30);
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);

    document.compute_layout(Space::new(buffer.width, buffer.height));

    let mut renderer = Renderer::new(BufferRenderingContext::new(&mut buffer, &mut arena));

    renderer.render(&document).unwrap();
    rasterizer.raster(&buffer, &arena).unwrap();
    rasterizer.flush(&mut io::stdout()).unwrap();

    document.print_layout();

}
