use ansi::io::Write;
use ansi::{Color, Style, UnderlineStyle};
use sigil::{Align, Alignment, Buffer, Constraints, Content, Edges, Layout, LayoutNode, Node, Position, Rect, align, center, fill, layer, pad, size, stack, style, text, Constraint, container, row};
use std::io;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();

    let bounds = Rect::new((0, 0), (80, 30));

    let header = Style::new()
        .foreground(Color::BrightRed)
        .bold()
        .underline()
        .underline_style(UnderlineStyle::Curly);
    let sub = Style::new()
        .foreground(Color::Index(241))
        .bold();

    let ui = stack![
            size!(
                Constraints::Vertical(1) => style!(sub => fill!('x'))
            ),

            size!(
                Constraints::Vertical(3) => align!(
                    Alignment::CENTER => style!(header => text!("Hello from Rust! 🦀"))
                )
            ),

            size!(
                Constraints::Vertical( 1) => style!(sub => fill!('x'))
            ),

        size!(
            Constraints::Vertical(10) =>  row![
                text!("x"),
                text!("y"),
                text!("z"),
            ]
        ),


    ];
    let tree = Layout::new(&ui, bounds);
    let mut buffer = Buffer::new(bounds);

    tree.render(&mut buffer);
    lock.escape(&buffer)?;


    Ok(())
}
