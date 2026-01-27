use ansi::io::Write;
use ansi::{Color, Style};
use kasten::{layout, Context, Buffer, Constraints, Edges, Node, Content, Rect, render, Position};

macro_rules! la {
    () => {};
}

fn main() {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let ui = Node::Style(
        Style::new().bold().background(Color::Blue).foreground(Color::White),
        Box::new(Node::Pad(
            Edges::all(1),
            Box::new(Node::Stack(vec![
                Node::Base(Content::Text("Header".into())),
                Node::Style(
                    Style::new().foreground(Color::Yellow).bold(),
                    Box::new(Node::Base(Content::Fill('.'))),
                ),
                Node::Base(Content::Fill('.')),
            ])),
        )),
    );

    let mut buffer = Buffer::new(Rect::new((0, 0), (80, 60)));

    // // 1. Layout
    let tree = layout(&ui, buffer.bounds, Constraints::Fixed(buffer.bounds.width(), buffer.bounds.height()));

    // // 2. Render to buffer
    let ctx = Context::default();
    render(&tree, &mut buffer, &ctx);
    // buffer.text(Position::new(2, 1)..Position::new(1, 78), &"Hello".to_string(), &Style::new().bold());
    dbg!(buffer.index_of(&Position::new(2, 1)), buffer.index_of(&Position::new(1, 78)), buffer.len());
    lock.write_escape(&buffer).unwrap();
}