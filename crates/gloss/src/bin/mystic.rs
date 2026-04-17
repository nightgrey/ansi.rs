use std::io::{self, Write as _};
use ansi::{Color};
use gloss::*;

fn main() -> io::Result<()> {
    let mut engine = Engine::new(40, 10);

    let root = engine.root_mut();
    root.display = Display::Flex;
    root.flex_direction = FlexDirection::Column;
    root.align_items = Some(AlignItems::Center);
    root.align_content = Some(AlignContent::Center);
    root.justify_content = Some(ContentAlignment::Center);
    root.background = Some(Color::Rgb(0, 0, 255));
    root.border = Border::Bold;
    root.color = Some(Color::White);

    engine.insert_with(
        Element::Span("~ 肏 ~"),
        |node| {
            node.font_weight = Some(FontWeight::Bold);
        },
    );


    engine.insert_with(
        Element::Span("Mystical"),
        |node| {
            node.margin.top = 1.into();
            node.color = Some(Color::Rgb(255, 0, 0));
            node.font_weight = Some(FontWeight::Bold);
        },
    );

    engine.render(&mut io::stdout())?;

    Ok(())
}
