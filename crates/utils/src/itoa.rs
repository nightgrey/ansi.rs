use std::io::Write;
use itoaaa::{Integer, Unsigned};


pub trait Itoa {
    fn itoa(self, w: &mut dyn Write) -> std::io::Result<()>;
}

/// Generate optimized implementations for one signed/unsigned pair.
///
/// `MAX_DIGITS` is computed at compile time from the type’s maximum value.
/// Requires `ilog10()` to be const (available since Rust 1.73).
macro_rules! impl_write_io {
       ($signed:ty, $unsigned:ty) => {
           impl Itoa for $signed {
               #[inline]
               fn itoa(self, w: &mut dyn Write) -> std::io::Result<()> {
                   // 1 byte for the optional '-' sign, plus digits for $unsigned::MAX
                   const MAX_LEN: usize = {
                       let digits = (<$unsigned>::MAX as u128).ilog10() as usize + 1;
                       digits + 1
                   };
                   let (neg, abs) = itoaaa::Integer::unsigned_abs(self);
                   let len = neg + itoaaa::Unsigned::dump_len(abs);
                   let mut buf = [0u8; MAX_LEN];
                   unsafe {
                       if neg != 0 {
                           *buf.get_unchecked_mut(0) = b'-';
                       }
                       itoaaa::Unsigned::unchecked_dump(abs, &mut buf[neg..len]);
                   }
                   w.write_all(&buf[..len])
               }
           }

           impl Itoa for $unsigned {
               #[inline]
               fn itoa(self, w: &mut dyn Write) -> std::io::Result<()> {
                   const MAX_LEN: usize = {
                       (<$unsigned>::MAX as u128).ilog10() as usize + 1
                   };
                   let abs = self;
                   let len = itoaaa::Unsigned::dump_len(abs);
                   let mut buf = [0u8; MAX_LEN];
                   unsafe {
                       itoaaa::Unsigned::unchecked_dump(abs, &mut buf[..len]);
                   }
                   w.write_all(&buf[..len])
               }
           }
       };
   }

impl_write_io!(i8,   u8);
impl_write_io!(i16,  u16);
impl_write_io!(i32,  u32);
impl_write_io!(i64,  u64);
impl_write_io!(i128, u128);