use std::borrow::Cow;
use std::io::{self, Write as _};
use std::time::Instant;
use bon::__::ide::builder_top_level::start_fn::doc;
use ansi::{Color, EraseDisplay, EraseDisplayToEnd, Home, Style, SGR};
use gloss::*;
use sigil::{ Buffer, GraphemeArena, Rasterizer};
use tree::At;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let mut arena = GraphemeArena::new();
    let mut buffer = Buffer::new(20, 30);
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);

    let mut document = Document::new();
    let now = Instant::now();

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
        node.set_color(Color::White);
        node.set_border(Border::Bold);
    });
    let a = document.insert_with_children_at_with(Node::Div(), [Node::Span(Cow::Borrowed("A"))], At::Child(row), |node| {
        node.set_background(Color::Green);

    });

    let b = document.insert_with_children_at_with(Node::Div(), [Node::Span(Cow::Borrowed("B"))], At::Child(row), |node| {
        node.set_background(Color::Yellow);
    });

    let c = document.insert_with_children_at_with(Node::Div(), [Node::Span(Cow::Borrowed("C"))], At::Child(row), |node| {
        node.set_background(Color::Blue);
    });

    document.insert_with_children_at_with(Node::Span(Cow::Borrowed("D")), [Node::Span(Cow::Borrowed("D"))], At::Child(c), |node| {
        node.set_background(Color::Red);
    });


    document.compute_layout(Space::new(buffer.width, buffer.height));

    let mut lock = stdout.lock();
    let mut renderer = BufferRenderer::new(&mut buffer, &mut arena);

    renderer.render(&document)?;
    rasterizer.raster(&buffer, &arena)?;
    rasterizer.flush(&mut lock)?;
    
    let root = document.node_mut(document.root);
    root.set_background(Color::White);
    
    rasterizer.raster(&buffer, &arena)?;
    rasterizer.flush(&mut lock)?;

    lock.flush()?;
    lock.write_all(b"x")?;

    Ok(())
}
