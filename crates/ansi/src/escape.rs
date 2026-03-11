
pub trait Escape {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl<T: AsRef<str>> Escape for T {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(self.as_ref().as_bytes())
    }
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
                inner: self,
                error: Ok(()),
            };
    
            match io::Write::escape(&mut adapter, escape) {
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
}


/// Writes escaped content to a writer, handling multiple arguments with short-circuit error handling.
///
/// This macro accepts a 'writer' and a list of values to be escaped. Values will be
/// escaped and the result will be passed to the writer. The writer may be any value
/// with a `write_fmt` method; generally this comes from an
/// implementation of either the [`fmt::Write`] or the [`io::Write`] trait. The macro
/// returns whatever the `write_fmt` method returns; commonly a [`fmt::Result`], or an
/// [`io::Result`].
///
///
/// # Examples
///
/// ```
/// use ansi::{Escape, CursorUp, CursorDown, CursorForward};
///
/// fn main() -> std::io::Result<()> {
///     use ansi::escape;
///
///     let mut w = Vec::new();
///     escape!(&mut w, CursorUp(1), CursorDown(1), CursorForward(1))?;
///
///     assert_eq!(w, b"\x1B[A\x1B[B\x1B[C");
///     Ok(())
/// }
/// ```
///
/// # Arguments
///
/// * `$dst` - A mutable reference to any type implementing [`std::io::Write`]. The destination
///   where escaped content will be written.
/// * `$arg` - One or more expressions to be escaped and written. Each argument is processed
///   in order and must be compatible with [`Write::escape`].
///
/// # Returns
///
/// Returns `std::io::Result<()>` which is:
/// - `Ok(())` if all arguments were successfully written
/// - `Err(e)` if any write operation fails, with subsequent arguments not processed
///
/// # Performance Notes
///
/// - Arguments are evaluated left to right
/// - Processing stops immediately upon the first error (short-circuit evaluation)
/// - No intermediate allocations are required; output is written directly
#[macro_export]
macro_rules! escape {
    ($dst:expr, $arg: expr) => {
           $dst.escape($arg)
    };
    ($dst:expr, $first: expr, $($args:expr),* $(,)?) => {{
        use $crate::Escape as _;
        let mut result: std::io::Result<()> = $dst.escape($first);
        $(
                if result.is_ok() {
                  result = $dst.escape($args);
                }
        )*
        result
    }};
}

/// Writes escaped value to the writer.
///
/// A single-value, functional version of [`escape!`].
pub fn escape(w: &mut impl std::io::Write, escape: impl Escape) -> std::io::Result<()> {
    io::Write::escape(w, escape)
}

