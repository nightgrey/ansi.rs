//! Compact grapheme-cluster storage — inline or arena-backed.
//!
//! # Motivation
//!
//! Terminal framebuffers store millions of cells; each cell's visible
//! character should be as small as possible while still supporting the full
//! Unicode repertoire. [`Grapheme`] solves this with a 4-byte niche-packed
//! handle:
//!
//! - **Inline** — graphemes of ≤4 UTF-8 bytes (ASCII, Latin, Cyrillic, CJK,
//!   most emoji) are stored directly inside the handle, with zero allocation
//!   and zero indirection.
//! - **Extended** — longer clusters (ZWJ sequences, complex scripts) live in
//!   a [`Graphemes`] arena and are referenced by a 24-bit slot.
//!
//! # Types
//!
//! | Type | Role |
//! |---|---|
//! | [`Grapheme`] | 4-byte niche-packed handle: inline UTF-8 or arena slot. |
//! | [`Graphemes`] | Arena for extended grapheme clusters with best-fit reuse. |
//! | [`GraphemesError`] | Errors from insertion, resolution, and release. |
//! | [`Source`] | Const-compatible conversion trait for `char`, `&str`,
//!   and arena-backed tuples. |
//! | [`IntoGraphemeWidth`] | Display-width measurement for grapheme sources. |

mod grapheme;
pub use grapheme::*;

mod graphemes;
pub use graphemes::*;

pub mod internals;
pub use internals::*;

pub mod sources;
pub use sources::*;


