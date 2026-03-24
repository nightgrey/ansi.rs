use std::borrow::Cow;
use std::io;
use std::io::{Write};

use ansi::{escape, Attribute, Color, Style};
use geometry::{Axis, Bounded, Contains, Intersect, Point, Rect, Size};
use sigil::{Buffer, Capabilities, Grapheme, GraphemeArena, Rasterizer};
use gloss::*;
use tree::At;

fn main() {
    let mut document = Document::new();

    let root = document.node_mut(document.root);
    root.align_items.insert(AlignItems::Start);
    root.justify_items.insert(JustifyItems::Start);

    let n = document.insert_with(Node::Span(Cow::Borrowed("Hello Worldwwwwwwwwwwwwwwwww!")), |node| {
        node.color = Color::Red;
    });

    let row = document.insert_with(Node::Div(), |node| {
        node.flex_direction = FlexDirection::Row;
        node.flex_grow = 1.0;
        node.gap = Axis { horizontal: Dimension::Length(1), vertical: Dimension::Length(1) };
    });

    let a = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background_color = Color::Green;
    });


    let b = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background_color = Color::Yellow;
    });

    let c = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background_color = Color::Blue;
    });

    let mut arena = GraphemeArena::new();
    let mut buffer = Buffer::new(10, 11);
    let mut out = io::stdout().lock();
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);

    document.compute_layout(Space { width: Available::Definite(80), height: Available::Definite(24) });
    // let mut renderer = Renderer::new(BufferContext::new(&mut buffer, &mut arena));
    // 
    // 
    // renderer.render(&document).unwrap();
    // rasterizer.raster(&buffer, &arena);
    // rasterizer.write(&mut out).unwrap();
    // 
    // dbg!(document.bounds(n));
}

