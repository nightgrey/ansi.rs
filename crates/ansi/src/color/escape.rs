use crate::Color;

use crate::Escape;
use derive_more::{AsRef, Deref, DerefMut, From, Into};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Into, AsRef)]
#[repr(transparent)]
pub struct Background(Color);


impl Background {
    pub fn as_color(&self) -> Color {
        self.0
    }
    pub fn as_foreground(self) -> Foreground {
        Foreground(self.0)
    }

    pub fn as_underline(self) -> Underline {
        Underline(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Into, AsRef)]
#[repr(transparent)]
pub struct Foreground(Color);

impl Foreground {
    pub fn as_color(&self) -> Color {
        self.0
    }
    pub fn as_background(self) -> Background {
        Background(self.0)
    }

    pub fn as_underline(self) -> Underline {
        Underline(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Into, AsRef)]
#[repr(transparent)]
pub struct Underline(Color);

impl Underline {
    pub fn as_color(&self) -> Color {
        self.0
    }
    pub fn as_background(self) -> Background {
        Background(self.0)
    }

    pub fn as_foreground(self) -> Foreground {
        Foreground(self.0)
    }
}

impl Color {
    pub fn as_background(self) -> Background {
        Background(self)
    }

    pub fn as_foreground(self) -> Foreground {
        Foreground(self)
    }

    pub fn as_underline(self) -> Underline {
        Underline(self)
    }
}

impl Escape for Foreground {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use Color::*;

        match self.0 {
            None => Ok(()),
            Black => w.write_all(b"30"),
            Red => w.write_all(b"31"),
            Green => w.write_all(b"32"),
            Yellow => w.write_all(b"33"),
            Blue => w.write_all(b"34"),
            Magenta => w.write_all(b"35"),
            Cyan => w.write_all(b"36"),
            White => w.write_all(b"37"),
            BrightBlack => w.write_all(b"90"),
            BrightRed => w.write_all(b"91"),
            BrightGreen => w.write_all(b"92"),
            BrightYellow => w.write_all(b"93"),
            BrightBlue => w.write_all(b"94"),
            BrightMagenta => w.write_all(b"95"),
            BrightCyan => w.write_all(b"96"),
            BrightWhite => w.write_all(b"97"),
            Index(i) => {
                write!(w, "38;5;{}", i)
            }
            Rgb(r, g, b) => {
                write!(w, "38;2;{};{};{}", r, g, b)
            }
        }
    }
}

impl Escape for Background {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use Color::*;

        match self.0 {
            None => Ok(()),
            Black => w.write_all(b"40"),
            Red => w.write_all(b"41"),
            Green => w.write_all(b"42"),
            Yellow => w.write_all(b"43"),
            Blue => w.write_all(b"44"),
            Magenta => w.write_all(b"45"),
            Cyan => w.write_all(b"46"),
            White => w.write_all(b"47"),
            BrightBlack => w.write_all(b"90"),
            BrightRed => w.write_all(b"91"),
            BrightGreen => w.write_all(b"92"),
            BrightYellow => w.write_all(b"93"),
            BrightBlue => w.write_all(b"94"),
            BrightMagenta => w.write_all(b"95"),
            BrightCyan => w.write_all(b"96"),
            BrightWhite => w.write_all(b"97"),
            Index(index) => {
                write!(w, "38;5;{}", index)
            }
            Rgb(r, g, b) => {
                write!(w, "38;2;{};{};{}", r, g, b)
            }
        }
    }
}

impl Escape for Underline {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use Color::*;
        match self.0 {
            None => Ok(()),
            Black => w.write_all(b"58;5;0"),
            Red => w.write_all(b"58;5;1"),
            Green => w.write_all(b"58;5;2"),
            Yellow => w.write_all(b"58;5;3"),
            Blue => w.write_all(b"58;5;4"),
            Magenta => w.write_all(b"58;5;5"),
            Cyan => w.write_all(b"58;5;6"),
            White => w.write_all(b"58;5;7"),
            BrightBlack => w.write_all(b"58;5;8"),
            BrightRed => w.write_all(b"58;5;9"),
            BrightGreen => w.write_all(b"58;5;10"),
            BrightYellow => w.write_all(b"58;5;11"),
            BrightBlue => w.write_all(b"58;5;12"),
            BrightMagenta => w.write_all(b"58;5;13"),
            BrightCyan => w.write_all(b"58;5;14"),
            BrightWhite => w.write_all(b"58;5;15"),

            Index(i) => {
                write!(w, "58;5;{}", i)
            }
            Rgb(r, g, b) => {
                write!(w, "58;2;{};{};{}", r, g, b)
            }
        }
    }
}
