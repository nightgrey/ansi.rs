//! # Kasten
//!
//! A declarative terminal UI layout library for Rust.
//!
//! Kasten provides a composable, tree-based API for building terminal user interfaces
//! with constraint-based layouts, ANSI styling, and buffer-based rendering.
//!
//! ## Features
//!
//! - **Declarative UI Trees**: Build interfaces by composing tree structures
//! - **Constraint-Based Layout**: Flexible sizing with various constraint types
//! - **Three-Phase Rendering**: Clean separation between measure, layout, and render
//! - **ANSI Styling**: First-class support for terminal colors and text attributes
//! - **Buffer-Based Rendering**: Efficient cell-by-cell buffer manipulation
//!
//! ## Quick Example
//!
//! ```rust
//! use ansi::{Color, Style};
//! use kasten::{layout, render, Buffer, Context, Constraints, Edges, Node, Content, Rect};
//!
//! // Build a UI tree
//! let ui = Node::Style(
//!     Style::new().bold().foreground(Color::Blue),
//!     Box::new(Node::Pad(
//!         Edges::all(1),
//!         Box::new(Node::Stack(vec![
//!             Node::Base(Content::Text("Hello".into())),
//!             Node::Base(Content::Text("World".into())),
//!         ])),
//!     )),
//! );
//!
//! // Create buffer and render
//! let mut buffer = Buffer::new(Rect::new((0, 0), (80, 24)));
//! let tree = layout(&ui, buffer.bounds, Constraints::Fixed(80, 24));
//! let ctx = Context::default();
//! render(&tree, &mut buffer, &ctx);
//!
//! // Output to terminal
//! # /*
//! use ansi::io::Write;
//! let stdout = std::io::stdout();
//! stdout.lock().write_escape(&buffer).unwrap();
//! # */
//! ```
//!
//! ## Core Concepts
//!
//! ### Node Tree
//!
//! UIs are built using the [`Node`] enum with variants for different layout and content types:
//! - [`Node::Base`] - Leaf nodes with content (text, fill, empty)
//! - [`Node::Style`] - Apply ANSI styling
//! - [`Node::Pad`] - Add padding/margins
//! - [`Node::Stack`] - Vertical layout
//! - [`Node::Row`] - Horizontal layout
//! - [`Node::Layer`] - Overlapping layout
//! - [`Node::Align`] - Alignment within available space
//! - [`Node::Size`] - Apply size constraints
//!
//! ### Three-Phase Workflow
//!
//! 1. **Measure** - Calculate natural sizes: [`measure()`]
//! 2. **Layout** - Assign positions and bounds: [`layout()`]
//! 3. **Render** - Draw to buffer: [`render()`]
//!
//! ### Constraints
//!
//! The [`Constraint`] type controls sizing behavior:
//! - `Auto` - Use natural size
//! - `Min(n)` - At least n units
//! - `Max(n)` - At most n units
//! - `Fixed(n)` - Exactly n units
//! - `Between(min, max)` - Within range
//! - `Fill` - Expand to available space
//!
//! ## Module Organization
//!
//! - [`tree`] - Node types and core layout/measure/render functions
//! - [`geometry`] - Point, Size, Rect primitives
//! - [`position`] - Position and Region for buffer indexing
//! - [`layout`] - Constraints, Edges, Alignment types
//! - [`buffer`] - Buffer and Cell for terminal rendering

#![feature(slice_index_methods)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_range)]

mod tree;
mod layout;
mod buffer;
mod geometry;
mod position;

pub use tree::*;
pub use layout::*;
pub use buffer::*;
pub use geometry::*;
pub use position::*;