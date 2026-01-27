pub trait Escape {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;

    fn escape_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let mut buf = [0u8; 64];

        let mut cursor = std::io::Cursor::new(&mut buf[..]);
        if let Err(err) = self.escape(&mut cursor) {
            eprintln!("{err}");
            return Err(std::fmt::Error);
        }

        let position = cursor.position() as usize;
        // SAFETY: escape only writes ASCII
        f.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..position]) })
    }
}

pub mod io {
    use super::*;

    pub trait Write {
        fn write_escape(&mut self, escape: &impl Escape) -> std::io::Result<()>;
    }

    impl<W: std::io::Write> Write for W {
        #[inline]
        fn write_escape(&mut self, escape: &impl Escape) -> std::io::Result<()> {
            escape.escape(self)
        }
    }
}

pub mod fmt {
    use super::*;

    pub trait Write {
        fn write_escape(&mut self, escape: &impl Escape) -> std::fmt::Result;
    }

    impl<W: std::fmt::Write> Write for W {
        #[inline]
        fn write_escape(&mut self, escape: &impl Escape) -> std::fmt::Result {
            escape.escape_fmt(self)
        }
    }
}
