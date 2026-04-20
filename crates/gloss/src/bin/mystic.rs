use ansi::Color;
use gloss::*;
use std::io::{self, Write as _};
use taffy::FlexboxContainerStyle;

fn main() -> io::Result<()> {
    let mut engine = Engine::new(40, 10);

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
            .margin((1, 1))
            .color(Color::Red)
            .bold(),
    );

    engine.render(&mut io::stdout())?;

    Ok(())
}
