use ui::mock::*;
use ui::*;
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    let mut ui = Engine::new(80, 24);

    chess_board(&mut ui);

    ui.render(&mut stdout())?;

    Ok(())
}
