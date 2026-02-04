use std::io;

fn main() -> io::Result<()> {
    // notcurses::main().unwrap();
    sigil::main()?;
    Ok(())
}

mod notcurses {
    use ansi::Bits;
    use notcurses::sys::c_api::{
        NcBoxMask_u32, ncplane_box, ncplane_box_sized, ncplane_double_box_sized,
    };
    use notcurses::sys::{NcBoxMask, NcCell, NcRgb};
    use notcurses::*;

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

        ncplane_box_sized(
            child.into_ref_mut(),
            &ul,
            &ur,
            &ll,
            &lr,
            &hl,
            &vl,
            10,
            10,
            NcBoxMask::default(),
        );

        child.set_bg(2);
        child.set_base_bg(Channel::from_rgb(Rgb::new(255, 0, 0)))?;
        child.render()?;
        cli.render()?;
        Ok(())
    }
}

mod sigil {
    use std::io::Write;
    use geometry::Point;
    use sigil::*;

    pub fn main() -> std::io::Result<()> {
        let mut engine = Engine::new(30, 5);

        // Build a simple UI
        let root = engine.root().unwrap();

        let header = engine.elements.insert(Element::text("=== Header ===".to_string()));
        let body = engine.elements.insert(Element::text("Hello, world!".to_string()));
        let footer = engine.elements.insert(Element::text("=== Footer ===".to_string()));

        engine.elements.append_child(root, header);
        engine.elements.append_child(root, body);
        engine.elements.append_child(root, footer);

        // Render
        let mut stdout = std::io::stdout();
        write!(stdout, "\x1b[2J")?;  // Clear screen
        engine.frame(&mut stdout)?;

        Ok(())
    }
}