# AGENTS.md

## Repository Overview

This repository is a Rust workspace (`Cargo.toml` at the root) with these members:

- `crates/sigil` — main rendering engine crate
- `crates/ansi` — ANSI colors, styles, escape sequences
- `crates/geometry` — geometric primitives like `Point`, `Rect`, `Size`, `Edges`
- `crates/grid` — generic 2D grid storage plus spatial/location helpers
- `crates/terminal` — terminal-facing helpers
- `crates/utils` — small shared utilities

There are also architecture notes in `docs/architecture.md`, `docs/possible-architecture.md`, and `docs/notcurses-architecture.md`.

## Toolchain and Build Assumptions

This workspace uses Rust 2024 edition throughout.

Multiple crate roots enable unstable features via `#![feature(...)]` attributes, including:

- `crates/sigil/src/lib.rs`
- `crates/geometry/src/lib.rs`
- `crates/ansi/src/lib.rs`

Use a nightly toolchain. No `rust-toolchain` or `rust-toolchain.toml` file is present, so the exact nightly is not pinned in-repo.

## Essential Commands

Run from the repository root.

### Build

```bash
cargo build --workspace
cargo build -p sigil
```

### Test

```bash
cargo test --workspace
cargo test -p sigil
cargo test -p geometry
cargo test -p grid
cargo test -p ansi
cargo test -p terminal
cargo test -p utils
```

### Benchmarks

Observed benchmark targets:

```bash
cargo bench -p sigil --bench rasterizer
cargo bench -p grid --bench step
```

### Run

Observed binary target:

```bash
cargo run -p sigil --bin sandbox
```

## Code Organization

### `crates/sigil`

`crates/sigil/src/lib.rs` re-exports the major subsystems:

- `buffer`
- `layout`
- `rasterizer`
- `painter`
- `engine`

Representative files:

- `crates/sigil/src/engine.rs` — `Engine` owns front/back buffers, grapheme arena, element/layer trees, layout storage, and rasterizer
- `crates/sigil/src/buffer/buffer.rs` — `Buffer` is a `Grid<Cell>` wrapper with terminal-style line/cell insertion and deletion helpers
- `crates/sigil/src/buffer/cell.rs` — terminal cell representation, grapheme storage rules, wide-character behavior
- `crates/sigil/src/layout/tree/*` — slotmap-backed tree implementation with custom ID trait and traversal helpers
- `crates/sigil/src/rasterizer/rasterizer.rs` — diffing/rasterization tests and terminal output behavior

### `crates/grid`

`Grid<T>` is a row-major contiguous `Vec<T>` with explicit `width` and `height`. The crate also contains:

- `area`
- `location` / `locations`
- `spatial`
- feature modules like `contains`, `intersect`, `sides`

### `crates/geometry`

Geometric primitives are split into focused files:

- `point.rs`
- `rect.rs`
- `size.rs`
- `edges.rs`

`Rect` is documented as half-open: `[min, max)`.

### `crates/ansi`

Style and color handling live under:

- `color/`
- `style/`
- `sequences/`
- `escape.rs`

`Style` uses builder-style methods like `.foreground(...)`, `.background(...)`, `.bold()`, `.underline()`, etc.

## Architecture Notes

`docs/architecture.md` and `docs/possible-architecture.md` describe the intended pipeline as:

```text
Tree -> Layout -> Layers -> Paint -> Composite -> Diff -> Terminal
```

That matches the main `Engine` shape in `crates/sigil/src/engine.rs`:

- element tree
- layer tree
- layout storage
- front/back buffers
- rasterizer

`docs/notcurses-architecture.md` is a detailed reference for the rendering model this project is borrowing from.

## Conventions and Patterns

## Naming and Coordinate Systems

There are two coordinate vocabularies in use:

- geometry types like `Rect` use `x` / `y`
- grid and buffer code use `row` / `col`

Be careful when moving between `geometry` and `grid`/`buffer` code.

## Data Structure Patterns

Observed recurring patterns:

- heavy use of `derive_more` derives for wrappers and small types
- many wrapper/newtype APIs expose `EMPTY` constants and `new()` constructors
- module roots often re-export submodules for a flat public API
- tree IDs are slotmap keys wrapped by the `tree_id!` macro in `crates/sigil/src/layout/tree/id.rs`
- custom ID helpers mimic `Option`-style behavior (`none()`, `is_none()`, `as_option()`, `or_else()`, etc.)

## Performance-Oriented Style

The codebase favors low-level, allocation-aware operations:

- `Grid<T>` is contiguous and row-major
- buffer mutation methods use `copy_within` and slice fills instead of element-by-element reallocation
- `Cell` is `#[repr(C)]` and documented with memory layout goals
- grapheme storage distinguishes inline and arena-backed content

Follow existing local patterns before introducing new abstractions.

## Testing Approach

Tests are colocated with implementation in `#[cfg(test)]` modules. Representative examples:

- `crates/sigil/src/layout/tree/tree.rs`
- `crates/sigil/src/rasterizer/rasterizer.rs`
- `crates/sigil/src/buffer/cell.rs`
- `crates/grid/src/area.rs`

When changing behavior, prefer adding or updating nearby unit tests in the same file.

Benchmarks are crate-local under `benches/`.

## Important Gotchas

- Nightly Rust is required; the repo does not pin a nightly version.
- No `Makefile`, `justfile`, `Taskfile`, GitHub Actions workflow, `rustfmt.toml`, or `clippy.toml` were found. Use Cargo directly and match surrounding style.
- `crates/sigil/src/engine.rs` still contains commented-out work and `TODO` markers around layering and painting. Treat that area as incomplete/WIP.
- `Cell`/grapheme code is stateful with respect to `GraphemeArena`; when changing cell replacement logic, preserve release semantics and wide-character continuation handling from `crates/sigil/src/buffer/cell.rs`.
- `Buffer` is indexed by `(row, col)` tuples in helpers like `from_chars`, even when nearby geometry code talks in `x`/`y`.
- `Rect` semantics are half-open; width and height calculations use saturating subtraction for inverted rectangles.
- As observed locally, `cargo test --workspace` currently aborts in `sigil` with a stack overflow in `layout::tree::tree::tests::detach_middle`; do not assume the full workspace test command is green before your change.

## Useful Files to Read First

Start here before making non-trivial changes:

- `Cargo.toml`
- `docs/architecture.md`
- `docs/notcurses-architecture.md`
- `crates/sigil/src/lib.rs`
- `crates/sigil/src/engine.rs`
- `crates/sigil/src/buffer/buffer.rs`
- `crates/sigil/src/buffer/cell.rs`
- `crates/sigil/src/layout/tree/tree.rs`
- `crates/grid/src/grid.rs`
- `crates/geometry/src/rect.rs`
- `crates/ansi/src/style/mod.rs`

## What Was Not Found

These were explicitly looked for and not found:

- existing `AGENTS.md`
- `.cursor/rules/*.md`
- `.cursorrules`
- `.github/copilot-instructions.md`
- `claude.md` / `CLAUDE.md` in the working tree
- root-level task runner files or CI configs
