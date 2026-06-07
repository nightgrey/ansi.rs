use ansi::{escape, Color, TextCursorEnable};
use ui::*;
use std::io::{self};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    escape!(&mut stdout, TextCursorEnable::Reset);

    let mut engine = Engine::new(55, 20);
    let _root = engine.root_id();

    engine.set_root(
        Element::Div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .background(Color::Rgb(0, 0, 255))
            .border(Border::Solid)
            .color(Color::White),
    );

    engine.insert(Element::Span("~ 肏 ~").bold());

    engine.insert(
        Element::Span("Mystical")
            .margin((4, 4))
            .color(Color::Red)
            .border(Border::Solid)
            .padding((0, 1))
            .bold(),
    );

    let debug = engine.insert(Element::Span("Debug"));
    let time = std::time::Instant::now();

    engine.render(&mut stdout)?;
    let after = time.elapsed();
    engine.map_root(|e| e.background(Color::Rgb(22, 0, 22)).no_border());

    engine.set(debug, Element::Span(format!("Time ({:?})", after)));

    engine.render(&mut stdout)?;

    Ok(())
}
