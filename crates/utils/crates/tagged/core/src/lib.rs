//! # tagged-union
//!
//! Derive macro for bit-packed tagged unions stored in a single integer.
//!
//! ## Quick start
//!
//! ```rust
//! use tagged::prelude::*;
//!
//! #[tagged(u32)]
//! enum Dimension {
//!     #[default]
//!     Auto,
//!     Length(u16),
//!     Percent(u30),   // arbitrary_int type: 30-bit uint
//! }
//!
//! let a = Dimension::AUTO;
//! assert!(a.is_auto());
//!
//! let l = Dimension::length(120);
//! assert_eq!(l.get_length(), 120);
//!
//! // Pseudo-types use arbitrary_int: accepts and returns u30
//! let p = Dimension::percent(u30::new(500_000));
//! assert_eq!(p.get_percent().value(), 500_000);
//!
//! // #[default] generates Default impl
//! assert_eq!(Dimension::default(), Dimension::AUTO);
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
//! ### Arbitrary-width types (`uN`)
//!
//! Field types like `u30` that aren't standard Rust integers are treated as
//! [`arbitrary_int`] types. Constructors, getters, and setters accept and
//! return the `arbitrary_int` type directly (e.g. `u30`), providing
//! compile-time bit-width safety.
//!
//! Standard types (`u8`, `u16`, `u32`, `u64`) are used as-is.

pub mod prelude;

pub use tagged_derive::tagged;

#[cfg(test)]
mod tests {
    use arbitrary_int::prelude::*;
    use super::*;

    #[tagged(u32)]
    enum Dimension {
        #[default]
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
        for raw in [0u32, 1, 1000, 500_000, (1 << 30) - 1] {
            let d = Dimension::percent(u30::new(raw));
            assert!(d.is_percent());
            assert_eq!(d.get_percent().value(), raw);
        }
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
        assert_eq!(format!("{:?}", Dimension::AUTO), "Dimension::Auto");
        assert_eq!(format!("{:?}", Dimension::length(120)), "Dimension::Length(120)");
        assert_eq!(format!("{:?}", Dimension::percent(u30::new(999))), "Dimension::Percent(999)");
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

        let l = Tiny::level(u6::new(63));
        assert!(l.is_level());
        assert_eq!(l.get_level().value(), 63);

        let l2 = Tiny::level(u6::new(0));
        assert_eq!(l2.get_level().value(), 0);
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

        let p = BigUnion::pointer(u62::new(0xDEAD_BEEF));
        assert!(p.is_pointer());
        assert_eq!(p.get_pointer().value(), 0xDEAD_BEEF);

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

        let e = Toggle::enabled(u31::new(100));
        assert!(e.is_enabled());
        assert!(!e.is_disabled());
        assert_eq!(e.get_enabled().value(), 100);

        let d = Toggle::disabled(u31::new(200));
        assert!(d.is_disabled());
        assert_eq!(d.get_disabled().value(), 200);
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

        let e = FiveWay::e(u13::new((1 << 13) - 1));
        assert!(e.is_e());
        assert_eq!(e.get_e().value(), (1 << 13) - 1);
    }

    // ── Constants exposed ───────────────────────────────────────────────

    #[test]
    fn constants_accessible() {
        assert_eq!(Dimension::TAG_WIDTH, 2);
        assert_eq!(Dimension::TAG_MASK, 0b11);
        assert_eq!(Dimension::PAYLOAD_WIDTH, 30);
    }

    // ── Setters ────────────────────────────────────────────────────────

    #[test]
    fn set_preserves_tag() {
        let mut d = Dimension::length(10);
        d.set_length(99);
        assert!(d.is_length());
        assert_eq!(d.get_length(), 99);

        let mut d = Dimension::percent(u30::new(100));
        d.set_percent(u30::new(200));
        assert!(d.is_percent());
        assert_eq!(d.get_percent().value(), 200);
    }

    // ── Default ─────────────────────────────────────────────────────────

    #[test]
    fn default_variant() {
        let d = Dimension::default();
        assert!(d.is_auto());
        assert_eq!(d, Dimension::AUTO);
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