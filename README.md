# ansi.rs

> ⚠️ **Experimental / work in progress.** APIs change without notice, features are incomplete, and nothing here is stable. Not recommended for production use.

A Rust workspace of experimental TUI and ANSI libraries. It provides low-level ANSI escape code primitives, geometry types, and a higher-level document-based terminal renderer.

## Crates

| Crate | Description |
| --- | --- |
| [`ansi`](crates/ansi) | ANSI escape code primitives: 4/8/24-bit colors, SGR attributes, CSI sequences (cursor movement, scrolling, erasure, modes). Uses a bit-packed `Color` type. |
| [`geometry`](crates/geometry) | Spatial primitives — `Point`, `Size`, `Rect`, `Edges`, `Sides` — plus terminal grid indexing (`Row`, `Column`, `Position`) and custom numeric traits. |
| [`gloss`](crates/gloss) | Highest-level crate: a document-based terminal renderer. Flexbox layout via [`taffy`](https://crates.io/crates/taffy), CSS-like styles, a diffable cell buffer, and a rasterizer for borders, text wrapping, and fills. |
| [`terminal`](crates/terminal) | Terminal capability detection (color support, dimensions). |
| [`utils`](crates/utils) | Shared utilities: an arena-based `tree`, `tagged`/`maybe` derive macros, and iterator adapters. |

## Toolchain

Requires **Rust nightly** (pinned via [`mise.toml`](mise.toml)). The codebase relies on several nightly features (`const_trait_impl`, `derive_const`, `const_cmp`, `const_destruct`, `ascii_char`, `bstr`, `new_range_api`, …) and Cargo resolver v3 (edition 2024).

## Building

```bash
cargo build                      # build all crates
cargo test                       # run all tests
cargo test -p ansi               # test a single crate
cargo test -p ansi -- test_name  # run a single test
cargo bench -p gloss             # run gloss benchmarks
cargo run --bin gloss            # run the gloss demo (or: mise run gloss)
```

## Status

This is a personal experiment. Expect churn: crates are being renamed, restructured, and rewritten frequently. Documentation is sparse and tests cover only part of the surface area. Feedback and ideas are welcome, but please don't depend on anything here yet.
