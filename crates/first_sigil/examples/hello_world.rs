use ansi::io::Write;
use first_sigil::{Layout, LayoutContext};
use first_sigil::{Buffer, Constraints, Content, Node, Rect};

fn main() {
    // Build a simple UI tree
    let ui = Node::Base(Content::Text("Hello, Kasten!".into()));

    // Create a buffer for rendering
    let mut buffer = Buffer::new(Rect::new((0, 0), (80, 24)));

    // Layout the tree
    let tree = Layout::new(&ui, buffer.bounds);

    // Render to buffer
    tree.render(&mut buffer);

    // Output to terminal
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.escape(&buffer).unwrap();
}
