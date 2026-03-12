use std::io::{self, Write};
use sigil::*;

fn main() -> io::Result<()> {
    tree()?;
    Ok(())
}
pub fn tree() -> io::Result<()> {
    let mut engine = Engine::new(30, 5);

    let header = engine
        .append_root(Element::text("=== Header ===".to_string()));
    let body = engine
        .append_root(Element::text("Hello, world!".to_string()));
    let footer = engine
        .append_root(Element::text("=== Footer ===".to_string()));

    // Render
    let mut stdout = io::stdout();
    engine.frame(&mut stdout)?;

    Ok(())
}
