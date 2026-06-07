use ansi::{escape, Color, TextCursorEnable};
use ui::*;
use std::io::{self};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    escape!(&mut stdout, TextCursorEnable::Reset);

    let mut engine = Engine::new(40, 20);
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
            .bold().border(Border::Solid),
    );

    let debug = engine.insert(Element::Span("Debug"));

    engine.render(&mut stdout)?;

    engine.set_root(
        Element::Div()
    );

    loop {
        let time = std::time::Instant::now();
        engine.render(&mut stdout)?;

        let after = time.elapsed();
        engine.set(debug, Element::Span(format!("Time ({:?})", after)));
        sleep(Duration::from_millis(100));
    }
    Ok(())
}
