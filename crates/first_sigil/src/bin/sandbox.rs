use std::io;

fn main() -> io::Result<()> {
    notcurses::main().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    // sigil()
}
mod sigil {
    use ansi::{Color, Style, UnderlineStyle};
    use ansi::io::Write;
    use std::io;

    use sigil::{Align, Alignment, Buffer, Constraints, Content, Edges, Layout, LayoutNode, Node, Position, Rect, align, center, fill, layer, pad, size, stack, style, text, Constraint, container, row};

    pub fn main() -> io::Result<()> {
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
}

mod notcurses {
    use notcurses::*;
    use notcurses::sys::{NcBoxMask, NcCell, NcRgb};
    use notcurses::sys::c_api::{ncplane_box, ncplane_box_sized, ncplane_double_box_sized, NcBoxMask_u32};
    use ansi::Bits;

    pub fn main() -> NotcursesResult<()> {
        let mut nc = Notcurses::new_cli()?;

        let mut cli = nc.cli_plane()?;
        cli.set_bg(2);
        cli.styles().set(Style::Bold);
        cli.translate_root(Position::new(44, 1));
        cli.putstr("Hello from Rust! 🦀")?;
        cli.putstr("qwe")?;
        cli.set_fg(Channel::from_rgb(Rgb::new(255, 0, 0)));
        let mut ul = NcCell::new();
        let mut ur = NcCell::new();
        let mut ll = NcCell::new();
        let mut lr = NcCell::new();
        let mut hl = NcCell::new();
        let mut vl = NcCell::new();
        let mut child = cli.new_child_sized_at(Size::new(10, 10), Position::new(10, 10))?;

        ncplane_box_sized(child.into_ref_mut(), &ul, &ur, &ll, &lr, &hl, &vl, 10, 10, NcBoxMask::default());


        child.set_bg(2);
        child.set_base_bg(Channel::from_rgb(Rgb::new(255, 0, 0)))?;
        child.render()?;
        cli.render()?;
        Ok(())
    }
}
