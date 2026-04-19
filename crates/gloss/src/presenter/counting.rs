#![forbid(unsafe_code)]

//! Counting writer for tracking bytes emitted.
//!
//! This module provides a wrapper around any `Write` implementation that
//! counts the number of bytes written. This is used to verify O(changes)
//! output size for diff-based rendering.
//!
//! # Usage
//!
//! ```
//! use crate::CountingWriter;
//! use std::io::Write;
//!
//! let mut buffer = Vec::new();
//! let mut writer = CountingWriter::new(&mut buffer);
//!
//! writer.write_all(b"Hello, world!").unwrap();
//! assert_eq!(writer.bytes_written(), 13);
//!
//! writer.reset_counter();
//! writer.write_all(b"Hi").unwrap();
//! assert_eq!(writer.bytes_written(), 2);
//! ```

use derive_more::{AsMut, AsRef};
use std::borrow::Borrow;
use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

/// A write wrapper that counts bytes written.
///
/// Wraps any `Write` implementation and tracks the total number of bytes
/// written through it. The counter can be reset between operations.
#[derive(Debug, AsMut, AsRef)]
pub struct CountingWriter<W: ?Sized + Write> {
    /// Total bytes written since last reset.
    bytes: u64,
    #[as_mut]
    #[as_ref]
    /// The underlying writer.
    inner: W,
}

impl<W: Write> CountingWriter<W> {
    /// Create a new counting writer wrapping the given writer.
    #[inline]
    pub fn new(inner: W) -> Self {
        Self { inner, bytes: 0 }
    }

    /// Get the number of bytes written since the last reset.
    #[inline]
    pub fn count(&self) -> u64 {
        self.bytes
    }

    /// Reset the byte counter to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.bytes = 0;
    }

    /// Unwraps this `CountingWriter<W>`, returning the underlying writer.
    pub fn into_inner(mut self) -> W {
        self.inner
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)?;
        self.bytes += buf.len() as u64;
        Ok(())
    }
}

impl<W: Write> Borrow<W> for CountingWriter<W> {
    fn borrow(&self) -> &W {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============== CountingWriter Tests ==============

    #[test]
    fn counting_writer_basic() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);

        writer.write_all(b"Hello").unwrap();
        assert_eq!(writer.count(), 5);

        writer.write_all(b", world!").unwrap();
        assert_eq!(writer.count(), 13);
    }

    #[test]
    fn counting_writer_reset() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);

        writer.write_all(b"Hello").unwrap();
        assert_eq!(writer.count(), 5);

        writer.reset();
        assert_eq!(writer.count(), 0);

        writer.write_all(b"Hi").unwrap();
        assert_eq!(writer.count(), 2);
    }

    #[test]
    fn counting_writer_write() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);

        // write() may write partial buffer
        let n = writer.write(b"Hello").unwrap();
        assert_eq!(n, 5);
        assert_eq!(writer.count(), 5);
    }

    #[test]
    fn counting_writer_flush() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);

        writer.write_all(b"test").unwrap();
        writer.flush().unwrap();

        // flush doesn't change byte count
        assert_eq!(writer.count(), 4);
    }

    #[test]
    fn counting_writer_into_inner() {
        let buffer: Vec<u8> = Vec::new();
        let writer = CountingWriter::new(buffer);
        let inner = writer.into_inner();
        assert!(inner.is_empty());
    }

    #[test]
    fn counting_writer_inner_ref() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);
        writer.write_all(b"test").unwrap();

        assert_eq!(writer.into_inner().len(), 4);
    }

    // --- CountingWriter edge cases ---

    #[test]
    fn counting_writer_debug() {
        let buffer: Vec<u8> = Vec::new();
        let writer = CountingWriter::new(buffer);
        let dbg = format!("{:?}", writer);
        assert!(dbg.contains("CountingWriter"), "Debug: {dbg}");
    }

    #[test]
    fn counting_writer_inner_mut() {
        let mut writer = CountingWriter::new(Vec::<u8>::new());
        writer.write_all(b"hello").unwrap();
        // Modify inner via inner_mut
        writer.as_mut().push(b'!');
        assert_eq!(writer.as_ref(), &b"hello!"[..]);
        // Byte counter unchanged by direct inner manipulation
        assert_eq!(writer.count(), 5);
    }

    #[test]
    fn counting_writer_empty_write() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);
        writer.write_all(b"").unwrap();
        assert_eq!(writer.count(), 0);
        let n = writer.write(b"").unwrap();
        assert_eq!(n, 0);
        assert_eq!(writer.count(), 0);
    }

    #[test]
    fn counting_writer_multiple_resets() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);
        writer.write_all(b"abc").unwrap();
        writer.reset();
        writer.reset();
        assert_eq!(writer.count(), 0);
        writer.write_all(b"de").unwrap();
        assert_eq!(writer.count(), 2);
    }

    #[test]
    fn counting_writer_accumulates_u64() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);
        // Write enough to test u64 accumulation (though not near overflow)
        for _ in 0..1000 {
            writer.write_all(b"x").unwrap();
        }
        assert_eq!(writer.count(), 1000);
    }

    #[test]
    fn counting_writer_multiple_flushes() {
        let mut buffer = Vec::new();
        let mut writer = CountingWriter::new(&mut buffer);
        writer.write_all(b"test").unwrap();
        writer.flush().unwrap();
        writer.flush().unwrap();
        writer.flush().unwrap();
        assert_eq!(writer.count(), 4);
    }

    #[test]
    fn counting_writer_into_inner_preserves_data() {
        let mut writer = CountingWriter::new(Vec::<u8>::new());
        writer.write_all(b"hello world").unwrap();
        let inner = writer.into_inner();
        assert_eq!(&inner, b"hello world");
    }

    #[test]
    fn counting_writer_initial_state() {
        let a = BufWriter::new(Vec::new());
        let buffer: Vec<u8> = Vec::new();
        let writer = CountingWriter::new(buffer);
        assert_eq!(writer.count(), 0);
        assert!(writer.as_ref().is_empty());
    }
}
