# CLAUDE.md

## Project Overview

The Rust project implements a declarative, tree-based terminal UI system using constraint-based layouts and buffer-based rendering.

## Build and Run Commands

```bash
# Build entire workspace
cargo build

# Run specific package
cargo run --package sigil --bin sandbox

# Run tests
cargo test

# Build specific crate
cargo build --package sigil --package sigil
```

## Architecture

### Workspace Structure

- `crates/ansi` - ANSI escape code generation (attributes, colors, styles)
- `crates/geometry` - 2D primitives (Point, Rect, Size, Edges)
- `crates/utils` - Shared utility functions
- `crates/terminal` - Terminal-based functionality (planned: events, inputs, 
  and more)
- `crates/sigil` - Layout engine

### Layouting

1. **Measure**: Calculate natural sizes of nodes based on content
2. **Layout**: Assign positions using constraints
3. **Render**: Draw to buffer with ANSI styling applied

## Key

- `crates/sigil/src/layout/constraints.rs` - Constraint system
- `crates/sigil/src/layout/layout.rs` - Layout engine
- `crates/sigil/src/layout/node.rs` - Node tree definition
- `crates/sigil/src/layout/macros.rs` - Procedural macros (`text!`, `stack!`, 
  `row!`, etc.)
- `crates/sigil/src/buffer/buffer.rs` - Buffer rendering with escape codes
