use ansi::io::Write;
use ansi::{Color, Style};
use kasten::{
    Align, Alignment, Buffer, Constraints, Content, Edges, Layout, LayoutNode, Node, Position,
    Rect, align, center, fill, layer, pad, size, stack, style, text,
};
use std::io;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();

    let size = terminal::size()?;
    let bounds = Rect::new((0, 0), (80, 10));

    let header = Style::new().foreground(Color::BrightRed).bold();
    let sub = Style::new().foreground(Color::Blue).bold();
    let ui = style!(
        Style::new().background(Color::Default).foreground(Color::White) =>
        stack![
            size!(
                Constraints::Fixed(40, 1) => fill!('x')
            ),

            style!(header => text!("Hello Ay! 👋")),

            size!(
                Constraints::Fixed(40, 1) => style!(sub => fill!('x'))
            ),

            fill!('.'),
        ]
    );
    let tree = Layout::new(&ui, bounds);
    let mut buffer = Buffer::new(bounds);

    tree.render(&mut buffer);
    lock.write_escape(&buffer)?;

    Ok(())
}
