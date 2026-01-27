use ansi::io::Write;
use kasten::{layout, render, Buffer, Context, Constraints, Node, Content, Rect};

fn main() {
    // Build a simple UI tree
    let ui = Node::Base(Content::Text("Hello, Kasten!".into()));

    // Create a buffer for rendering
    let mut buffer = Buffer::new(Rect::new((0, 0), (80, 24)));

    // Layout the tree
    let tree = layout(
        &ui,
        buffer.bounds,
        Constraints::Fixed(buffer.bounds.width(), buffer.bounds.height()),
    );

    // Render to buffer
    let ctx = Context::default();
    render(&tree, &mut buffer, &ctx);

    // Output to terminal
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_escape(&buffer).unwrap();
}
