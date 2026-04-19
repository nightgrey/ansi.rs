use ansi::Color;
use gloss::mock::*;
use gloss::*;
use std::io::{self, stdout};
use tree::At;

fn main() -> io::Result<()> {
    let mut ui = Engine::new(1200, 20);

    chess_board(&mut ui);

    ui.render(&mut stdout())?;

    Ok(())
}
