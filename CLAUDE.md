# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Sigil is a TUI rendering engine inspired by notcurses and the DOM. It provides a scene-graph-based pipeline for compositing styled text onto terminal buffers.

## Build & Test

Requires **nightly Rust** (heavy use of unstable features like `const_trait_impl`, `step_trait`, `new_range_api`, `slice_index_methods`, etc.).

```bash
cargo build                          # build all crates
cargo test                           # run all tests
cargo test -p geometry               # test single crate
cargo test -p sigil -- cell          # run tests matching "cell" in sigil
cargo bench -p sigil                 # buffer benchmarks (criterion)
cargo bench -p geometry              # region iteration benchmarks
cargo run -p sigil --bin sandbox     # run the sandbox binary
```

## Workspace Crates

- **sigil** — Main crate. Framebuffer (`Buffer`, `Cell`, `Grapheme`, `GraphemePool`), scene graph (`Tree`, `Engine`, `Layer`, `Element`), text rendering, and the render pipeline.
- **geometry** — 2D spatial primitives: `Point`, `Size`, `Rect`, `Position`, `Bounds`, `Region`. Grid iteration via `SpatialIter`/`Cursor`. Half-open `[min, max)` range semantics throughout.
- **ansi** — Terminal styling: `Color` (None/Default/Index/RGB), `Attribute` (bitflags), `Style` (fg/bg/underline/attributes), ANSI escape sequence generation.
- **terminal** — Terminal size queries and signal handling.
- **utils** — Shared utilities: string packing, segmentation, separator macros.

## Rendering Pipeline

```
Tree → Layout → Layers → Paint → Composite → Diff → Terminal
```

Tree holds the scene graph (slotmap-based). Layout computes bounds/constraints. Layers promotes nodes to their own buffers. Paint writes cells into layers (skipping clean ones). Composite flattens back-to-front. Diff compares against the previous frame and emits minimal ANSI output.

## Key Types & Memory Layout

- **Cell** (16 bytes, `repr(C)`): grapheme handle + display width + style. Cache-line optimized.
- **Grapheme** (4 bytes): dual-mode encoding — inline for ≤4 UTF-8 bytes, 24-bit pool offset for extended graphemes (emoji ZWJ sequences).
- **GraphemePool**: arena allocator for extended graphemes (16 MiB max via 24-bit offset).
- **Bounds**: half-open `[min, max)` rectangular region with saturating arithmetic.
- **Tree\<K: Key, V\>**: generic slotmap-backed tree with O(1) insert/remove and stable keys.

## Patterns

- Heavy use of `derive_more` for operator implementations.
- Const-friendly design: many geometric operations are `const fn` via nightly const trait features.
- Saturating arithmetic everywhere to prevent panics on malformed/inverted regions.
- `Grid` trait for 2D array access (implemented by `Bounds`, `Buffer`).
- Builder/fluent API for `Style` construction.
- Row-major iteration order for spatial types.
