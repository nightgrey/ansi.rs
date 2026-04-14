# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Experimental TUI (Terminal User Interface) and ANSI libraries written in Rust. This is a workspace of multiple crates that together provide low-level ANSI escape code handling, geometry primitives, and a higher-level document-based terminal rendering system.

## Build Commands

```bash
cargo build                    # build all crates
cargo test                     # run all tests
cargo test -p ansi             # test a single crate
cargo test -p ansi -- test_name  # run a single test
cargo bench -p gloss           # run benchmarks (gloss)
cargo run --bin gloss          # run the gloss demo binary (or: mise run gloss)
```

## Toolchain

Requires **Rust nightly** (configured via `mise.toml`). The codebase uses many nightly features: `const_trait_impl`, `derive_const`, `const_cmp`, `const_destruct`, `ascii_char`, `bstr`, `new_range_api`, and others. Uses Cargo resolver v3 (edition 2024).

## Workspace Architecture

### `ansi` — ANSI escape code primitives
- **Color**: 4-bit, 8-bit, and 24-bit RGB color representation with a compact bit-packed layout (`Color` type uses `bilge` for bitfield packing)
- **Style**: `Attribute` flags (bold, italic, underline, etc.) and `Style` combining foreground/background colors with attributes
- **Sequences**: CSI/SGR escape sequence generation — cursor movement, scrolling, erasure, terminal modes
- Color and Attribute types implement bitwise ops (`BitOr`, `BitAnd`) with `None` acting as identity/zero

### `geometry` — Geometric primitives
- `Point`, `Size`, `Rect`, `Edges`, `Sides` — core spatial types
- `Row`, `Column`, `Position` — terminal grid indexing
- Feature system (`features/`) for transforms and mapping over geometric types
- Custom numeric traits (`num/`) including checked operations and float constants

### `gloss` — Document-based terminal renderer (highest-level crate)
- **Document model**: `Document` holds a `Tree<NodeId, Node>` with `Node::Div` (flex container) and `Node::Span` (text leaf)
- **Layout**: Uses `taffy` for flexbox layout computation. `LayoutContext` bridges the document tree to taffy's layout engine via custom `measure_node` for text measurement
- **Buffer**: 2D cell grid (`Buffer`) with `Cell` containing grapheme + style. Supports diffing (`diff.rs`) for efficient terminal updates
- **Rasterizer**: Converts laid-out document into buffer cells — handles borders, text wrapping, background fills
- **Style system**: CSS-like properties (`BorderStyle`, `TextDecoration`, `FontWeight`, `Dimension`, `Display`), box drawing symbols, and block/shade characters

### `terminal` — Terminal capability detection
- Queries terminal capabilities (color support, dimensions)

### `utils` — Shared utilities
- **`tree`**: Arena-based tree data structure with `Tree`, `Secondary` (parallel storage keyed by tree IDs), node traversal iterators
- **`tagged`/`tagged-derive`**: Derive macro for tagged union patterns
- **`maybe`/`maybe-derive`**: Derive macro for optional/nullable wrapper types
- `SeparateBy` iterator adapter

## Key Patterns

- The `tree` crate's `id!` macro generates strongly-typed ID newtypes (e.g., `NodeId`) for arena indexing
- `Document` uses a `Tree` for the node hierarchy and `Secondary` for parallel layout data
- `Node` derefs to `Style`, so style properties are set directly on nodes
- Constructor-style associated functions use PascalCase (`Node::Div()`, `Node::Span(...)`) to resemble enum-like construction
