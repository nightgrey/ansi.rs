use geometry::Size;
use rustix::io;
use std::io::stdin;

pub fn size() -> io::Result<Size> {
    let size = rustix::termios::tcgetwinsize(&stdin())?;

    Ok(Size::new(size.ws_col as usize, size.ws_row as usize))
}
