//! # tagged-union
//!
//! Derive macro for bit-packed tagged unions stored in a single integer.
//!
//! ## Quick start
//!
//! ```rust
//! use tagged::tagged;
//!
//! #[tagged(u32)]
//! enum Dimension {
//!     Auto,
//!     Length(u32),
//!     Percent(u30),   // pseudo-type: 30-bit uint, mapped to u32
//! }
//!
//! let a = Dimension::AUTO;
//! assert!(a.is_auto());
//!
//! let l = Dimension::length(120);
//! assert_eq!(l.get_length(), 120);
//!
//! let p = Dimension::percent(500_000);
//! assert_eq!(p.get_percent(), 500_000);
//!
//! // Everything fits in a single u32
//! assert_eq!(size_of::<Dimension>(), 4);
//! ```
//!
//! ## How it works
//!
//! The macro turns an enum into a `#[repr(transparent)]` newtype around the
//! chosen backing integer. The lowest bits store the **tag** (discriminant)
//! and the remaining upper bits store the **payload**.
//!
//! ```text
//! MSB ←────────────── backing integer ──────────────→ LSB
//! ┌──────────────────────────────┬──────────────────────┐
//! │         payload              │        tag           │
//! │      (PAYLOAD_WIDTH)          │     (TAG_WIDTH)       │
//! └──────────────────────────────┴──────────────────────┘
//! ```
//!
//! ### Pseudo-types (`uN`)
//!
//! Field types like `u30` that aren't standard Rust integers are interpreted
//! as "N-bit unsigned integer" and mapped to the smallest containing standard
//! type (`u32` for `u30`). Values are masked on both construction and extraction.
//!
//! Standard types (`u8`, `u32`, `u32`, `u64`) are used as-is.

pub mod prelude;

pub use tagged_derive::tagged;


#[cfg(test)]
mod tests {
    use arbitrary_int::prelude::*;
    use super::*;


    #[tagged(u32)]
    enum Dimension {
        Auto,
        Length(u16),
        Percent(u30),
    }

    #[test]
    fn dimension_size() {
        assert_eq!(size_of::<Dimension>(), 4);
    }

    #[test]
    fn dimension_auto() {
        let d = Dimension::AUTO;
        assert!(d.is_auto());
        assert!(!d.is_length());
        assert!(!d.is_percent());
        assert_eq!(d.tag(), 0);
        assert_eq!(d.bits(), 0);
    }

    #[test]
    fn dimension_length_roundtrip() {
        for val in [0u16, 1, 42, 255, 1000, u16::MAX] {
            let d = Dimension::length(val);
            assert!(d.is_length());
            assert!(!d.is_auto());
            assert_eq!(d.get_length(), val);
        }
    }

    #[test]
    fn dimension_percent_roundtrip() {
        let max_30bit: u32 = (1 << 30) - 1;
        for val in [0u32, 1, 1000, 500_000, max_30bit] {
            let d = Dimension::percent(val);
            assert!(d.is_percent());
            assert_eq!(d.get_percent(), val);
        }
    }

    #[test]
    fn dimension_percent_overflow_masked() {
        // If someone passes a value exceeding 30 bits, it gets silently masked
        let over = 1u32 << 30; // bit 30 set = exceeds u30
        let d = Dimension::percent(over);
        assert_eq!(d.get_percent(), 0); // masked away
    }

    #[test]
    fn dimension_equality() {
        assert_eq!(Dimension::AUTO, Dimension::AUTO);
        assert_eq!(Dimension::length(42), Dimension::length(42));
        assert_ne!(Dimension::length(42), Dimension::length(43));
        assert_ne!(Dimension::AUTO, Dimension::length(0));
    }

    #[test]
    fn dimension_debug() {
        let s = format!("{:?}", Dimension::AUTO);
        assert_eq!(s, "Auto");

        let s = format!("{:?}", Dimension::length(120));
        assert_eq!(s, "Length(120)");

        let s = format!("{:?}", Dimension::percent(999));
        assert_eq!(s, "Percent(999)");
    }

    #[test]
    fn dimension_copy_clone() {
        let a = Dimension::length(7);
        let b = a; // Copy
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn dimension_raw_roundtrip() {
        let d = Dimension::length(42);
        let raw = d.bits();
        let d2 = unsafe { Dimension::from_bits(raw) };
        assert_eq!(d, d2);
        assert_eq!(d2.get_length(), 42);
    }

    // ── u8 backing ──────────────────────────────────────────────────────

    #[tagged(u8)]
    enum Tiny {
        Off,
        On,
        Level(u6),
    }

    #[test]
    fn tiny_u8() {
        assert_eq!(core::mem::size_of::<Tiny>(), 1);

        assert!(Tiny::OFF.is_off());
        assert!(Tiny::ON.is_on());

        let max_6bit = (1u8 << 6) - 1; // 63
        let l = Tiny::level(max_6bit);
        assert!(l.is_level());
        assert_eq!(l.get_level(), max_6bit);

        // Overflow masked
        let l2 = Tiny::level(255);
        assert_eq!(l2.get_level(), 63);
    }

    // ── u64 backing ─────────────────────────────────────────────────────

    #[tagged(u64)]
    enum BigUnion {
        Empty,
        Pointer(u62),
        Counter(u32),
    }

    #[test]
    fn big_union_u64() {
        assert_eq!(size_of::<BigUnion>(), 8);

        assert!(BigUnion::EMPTY.is_empty());

        let p = BigUnion::pointer(0xDEAD_BEEF);
        assert!(p.is_pointer());
        assert_eq!(p.get_pointer(), 0xDEAD_BEEF);

        let c = BigUnion::counter(u32::MAX);
        assert!(c.is_counter());
        assert_eq!(c.get_counter(), u32::MAX);
    }

    // ── Two variants = 1 tag bit ────────────────────────────────────────

    #[tagged(u32)]
    enum Toggle {
        Enabled(u31),
        Disabled(u31),
    }

    #[test]
    fn two_variants_one_tag_bit() {
        assert_eq!(Toggle::TAG_WIDTH, 1);
        assert_eq!(Toggle::PAYLOAD_WIDTH, 31);

        let e = Toggle::enabled(100);
        assert!(e.is_enabled());
        assert!(!e.is_disabled());
        assert_eq!(e.get_enabled(), 100);

        let d = Toggle::disabled(200);
        assert!(d.is_disabled());
        assert_eq!(d.get_disabled(), 200);
    }

    // ── Single variant = 0 tag bits ─────────────────────────────────────

    #[tagged(u32)]
    enum Mono {
        Value(u32),
    }

    #[test]
    fn single_variant_no_tag() {
        assert_eq!(Mono::TAG_WIDTH, 0);
        assert_eq!(Mono::PAYLOAD_WIDTH, 32);

        let m = Mono::value(0x12345678);
        assert!(m.is_value());
        assert_eq!(m.get_value(), 0x12345678);
    }

    // ── All-unit enum ───────────────────────────────────────────────────

    #[tagged(u8)]
    enum Direction {
        North,
        South,
        East,
        West,
    }

    #[test]
    fn all_unit_variants() {
        assert_eq!(Direction::TAG_WIDTH, 2);
        assert!(Direction::NORTH.is_north());
        assert!(Direction::SOUTH.is_south());
        assert!(Direction::EAST.is_east());
        assert!(Direction::WEST.is_west());
        assert_ne!(Direction::NORTH, Direction::SOUTH);
    }

    // ── Five variants = 3 tag bits ──────────────────────────────────────

    #[tagged(u16)]
    enum FiveWay {
        A,
        B,
        C,
        D,
        E(u13),
    }

    #[test]
    fn five_variants_three_tag_bits() {
        assert_eq!(FiveWay::TAG_WIDTH, 3);
        assert_eq!(FiveWay::PAYLOAD_WIDTH, 13);

        let e = FiveWay::e((1 << 13) - 1);
        assert!(e.is_e());
        assert_eq!(e.get_e(), (1 << 13) - 1);
    }

    // ── Constants exposed ───────────────────────────────────────────────

    #[test]
    fn constants_accessible() {
        assert_eq!(Dimension::TAG_WIDTH, 2);
        assert_eq!(Dimension::TAG_MASK, 0b11);
        assert_eq!(Dimension::PAYLOAD_WIDTH, 30);
    }

    // ── Hash works ──────────────────────────────────────────────────────

    #[test]
    fn hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Dimension::AUTO);
        set.insert(Dimension::length(42));
        set.insert(Dimension::length(42)); // duplicate
        assert_eq!(set.len(), 2);
    }

}