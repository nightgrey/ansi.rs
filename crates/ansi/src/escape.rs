
pub trait Escape: Sized + Copy {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;
    fn escape_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        use std::io::Cursor;

        // Create a shim which translates a `io::Write` to a `fmt::Write` and saves off
        // I/O errors, instead of discarding them.
        struct Adapter<'a, Inner: std::fmt::Write + 'a> {
            inner: &'a mut Inner,
            error: std::fmt::Result,
        }

        impl<Inner: std::fmt::Write> io::Write for Adapter<'_, Inner> {
            fn escape(&mut self, escape: impl Escape) -> std::io::Result<()> {
                let mut buf = Vec::<u8>::new();
                let mut cursor = Cursor::new(&mut buf);

                escape.escape(&mut cursor)?;

                unsafe {
                    self.inner
                        .write_str(&String::from_utf8_unchecked(buf))
                        .map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::Other, "Failed to write string")
                        })?;
                }
                Ok(())
            }
        }

        let mut adapter = Adapter {
            inner: f,
            error: Ok(()),
        };

        match io::Write::escape(&mut adapter, *self) {
            Ok(()) => Ok(()),
            Err(..) => {
                // Check whether the error came from the underlying `Write`.
                if adapter.error.is_err() {
                    adapter.error
                } else {
                    // This shouldn't happen: the underlying stream did not error,
                    // but somehow the formatter still errored?
                    panic!(
                        "a formatting trait implementation returned an error when the underlying stream did not"
                    );
                }
            }
        }
    }
}

pub fn escape(w: &mut impl std::io::Write, escape: impl Escape) {
    let _ = escape.escape(w);
}

pub mod io {
    use super::*;

    pub trait Write {
        fn escape(&mut self, escape: impl Escape) -> std::io::Result<()>;
    }

    impl<W: std::io::Write> Write for W {
        #[inline]
        fn escape(&mut self, escape: impl Escape) -> std::io::Result<()> {
            escape.escape(self)
        }
    }
}

pub mod fmt {
    use super::*;

    pub trait Write {
        fn escape(&mut self, escape: impl Escape) -> std::fmt::Result;
    }

    impl<W: std::fmt::Write> Write for W {
        #[inline]
        fn escape(&mut self, escape: impl Escape) -> std::fmt::Result {
            escape.escape_fmt(self)
        }
    }
}
