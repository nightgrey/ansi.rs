use std::io;

fn main() -> io::Result<()> {
    Ok(())
}

/*
// STAGE 1 Goal: Scene engine.
fn stage_1() -> io::Result<()> {
    let mut engine = Engine::new(30, 5);

    // Rows = styled div with rows layout
    let root = engine.insert(Rows::auto());
    root.padding = Padding::all(1);

    // Div = basic block element
    let teaser = engine.insert(Div::new());
    teaser.background = Color::Black;
    teaser.border = Border::Solid;
    teaser.padding = Padding::new(4, 2);
    teaser.align_items = AlignItems::Center;
    teaser.justify_content = JustifyContent::Center;
    teaser.width = Percentage(100.0);

    // Span = basic inline element
    let header = engine.insert_at(Span::new("SIGIL").foreground(Color::White).bold().underline(), teaser);

    // Columns = styled div with columns layout
    let columns = engine.insert(Columns::new(2));
    let column_1 = engine.insert_at(Div::new(), columns);
    column_1.padding = Padding::all(2);

    let lorem_ipsum = engine.insert_at(Span::new(LOREM_IPSUM), column_1);

    let column_2 = engine.insert_at(Div::new(), columns);
    column_2.padding = Padding::all(2);

    let lorem_ipsum = engine.insert_at(Span::new(LOREM_IPSUM), column_2);

    let mut stdout = io::stdout();
    engine.render(&mut stdout)?;

    Ok(())
}

// STAGE 2 Goal: Unsure yet, but more managed / declarative (Elm-ish? React-ish?)
fn stage_2() -> io::Result<()> {
    let mut app = App::new()?;

    app.run(|ctx| {


    });


    Ok(())
}
*/
