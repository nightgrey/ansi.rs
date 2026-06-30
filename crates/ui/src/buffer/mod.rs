//! Terminal framebuffer — the 2D cell grid that backs every rendered frame.
//!
//! # Architecture
//!
//! The buffer subsystem is organised around a flat, row-major `Vec<Cell>` with
//! grid-aware indexing, styled text-writing primitives, and an efficient on-disk
//! representation for grapheme clusters.
//!
//! ## Core types
//!
//! | Type | Role |
//! |---|---|
//! | [`Buffer`] | Owned 2D cell grid; derefs to `Vec<Cell>` for slice-based access. |
//! | [`Cell`]   | A single grid cell: grapheme, display width, and [`Style`]. |
//! | [`Grapheme`] | 4-byte niche-packed handle: inline UTF-8 or arena slot. |
//! | [`Graphemes`] | Arena for extended grapheme clusters (>4 bytes). |
//! | [`Cells`] | Row-writing helpers and cell iteration. |
//!
//! ## Indexing
//!
//! The buffer accepts many index types — `usize`, [`Point`], [`PointLike`],
//! [`Row`], and all `std::ops::Range` variants — via a layered trait system:
//!
//! - [`BufferIndex`] — the foundation: converts an index into a [`SliceIndex`]
//!   for `[]` access.
//! - [`BufferIndexMany`] — extends single-cell indices to return `&[Cell]`
//!   slices.
//! - [`BufferIndexExt`] — geometry queries: `x()`, `y()`, `len()`, `within()`,
//!   conversions to `Point` and `Range<usize>`.
//! - [`BufferIndexIter`] — turns any index into a cell iterator, handling
//!   out-of-bounds gracefully with empty iterators.
//!
//! ## Grapheme lifecycle
//!
//! Short graphemes (≤4 UTF-8 bytes) are stored directly inside [`Grapheme`] —
//! zero allocation, zero indirection. Longer sequences (ZWJ emoji, complex
//! scripts) spill into a [`Graphemes`] arena. The arena uses best-fit reuse,
//! coalescing, and tail-truncation to stay compact across cell-level churn.
//!
//! # Example
//!
//! ```ignore
//! use ui::buffer::{Buffer, Cell, Cells, Graphemes};
//! use ansi::Style;
//!
//! let mut arena = Graphemes::new();
//! let mut buf = Buffer::new(80, 24);
//!
//! // Write a styled line using Cell-based helpers.
//! Cells::write(
//!     buf.get_many_mut((0, 0).row()..(80, 0).row()).unwrap(),
//!     "Hello, 世界!",
//!     Some(Style::None),
//!     &mut arena,
//! );
//! ```

mod buffer;
pub use buffer::*;

mod cell;
pub use cell::*;
mod cells;
pub use cells::*;

mod graphemes;
pub use graphemes::*;

pub mod index;
pub use index::*;
pub mod index_ext;
pub use index_ext::*;
pub mod index_many;
pub use index_many::*;
pub mod generation;
pub mod index_iter;
pub mod diff;

pub use index_iter::*;
