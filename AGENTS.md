# AGENTS.md

This document provides essential information for AI coding agents working on this project.

## Project Overview

**Sigil** is a declarative, tree-based terminal UI framework written in Rust. It implements a constraint-based layout system with buffer-based rendering and ANSI escape code generation for styling terminal output.

The project name appears to be "fifi" in some configuration files (legacy), but the actual crate is named **sigil**.

## Technology Stack

- **Language**: Rust (Edition 2024)
- **Build Tool**: Cargo
- **Key Dependencies**:
  - `slotmap` - Efficient slot-map based storage for tree nodes
  - `indextree` - Alternative tree implementation (may be legacy)
  - `taffy` - Flexbox-like layout engine (future integration)
  - `compact_str` - Compact string representation for cells
  - `unicode-width` / `unicode-segmentation` - Unicode handling
  - `bitflags` - Bitflag attributes for ANSI styles
  - `derive_more` - Reduced boilerplate for derives
  - `rustix` - Safe Rust bindings to POSIX/Linux syscalls
  - `smol` - Async runtime (in terminal crate)
  - `notcurses` - Alternative terminal rendering backend (optional)

## Project Structure

This is a Cargo workspace with 5 crates:

```
crates/
├── ansi/       # ANSI escape code generation (colors, styles, attributes)
├── geometry/   # 2D primitives (Point, Rect, Size, Edges, Position)
├── terminal/   # Terminal I/O, events, and system integration
├── sigil/      # Main UI framework (layout engine, buffer, elements)
└── utils/      # Shared utility macros and helpers
```

### Crate Dependencies

```
sigil → ansi, geometry, terminal, utils
ansi → utils
terminal → ansi, geometry, utils
geometry → (no internal deps)
utils → (no internal deps)
```

## Build and Test Commands

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build --package sigil

# Run tests
cargo test --workspace

# Run specific test
cargo test --package sigil -- <test_name>

# Run the sandbox binary (demo application)
cargo run --package sigil --bin sandbox
```

## Architecture

### Rendering Pipeline

The UI follows a multi-stage rendering pipeline (see `docs/architecture.md`):

1. **Tree** - Element tree with parent-child relationships (`ui/tree/`)
2. **Layout** - Calculate bounds and positions (`Engine::layout()`)
3. **Layer** - Promoted nodes get their own compositing layer (`ui/layer.rs`)
4. **Paint** - Write cells into each layer's buffer (`Engine::paint()`)
5. **Composite** - Flatten layers back-to-front (`Engine::composite()`)
6. **Render/Diff** - Compare with previous frame, output ANSI codes (`Engine::render()`)

### Key Modules

#### `crates/sigil/src/ui/engine.rs`
The `Engine` struct is the main entry point:
- Manages element tree (`Tree<ElementId, Element>`)
- Manages layer tree (`Tree<LayerId, Layer>`)
- Executes the full rendering pipeline via `frame()`

#### `crates/sigil/src/ui/tree/`
Custom tree implementation using slotmap:
- `Tree<K, V>` - Generic tree with typed keys
- `Node<K, V>` - Tree node with embedded structural links (parent, siblings, children)
- `NodeRef`/`NodeRefMut` - Reference wrappers with tree navigation
- Multiple iterators: `Children`, `Descendants`, `Ancestors`, `Traverse`
- `Secondary<K, V>` - Secondary map for attaching computed data (e.g., layouts)

#### `crates/sigil/src/ui/element.rs`
UI element definitions:
- `ElementId` - Typed key for elements
- `Element` - UI element with kind, layer_id, and style
- `ElementKind::Container { direction }` - Layout container (Horizontal/Vertical)
- `ElementKind::Text(String)` - Text content

#### `crates/sigil/src/buffer/`
Terminal buffer implementation:
- `Buffer` - 2D grid of cells with various indexing methods
- `Cell` - Single terminal cell with grapheme content and style
- `BufferIndex` trait - Type-safe indexing (usize, Position, Point, Row, ranges)
- `DoubleBuffer` - Front/back buffer pair for differential rendering

#### `crates/ansi/src/`
ANSI terminal control:
- `Style` - Combined attributes, foreground/background/underline colors
- `Attribute` - Bitflags for text attributes (bold, italic, underline, etc.)
- `Color` - ANSI color support (basic, 256-color, RGB)
- `Escape` trait - For writing ANSI escape sequences

#### `crates/geometry/src/`
2D geometry primitives:
- `Point` - 2D point with x, y coordinates
- `Rect` - Axis-aligned rectangle (min/max points, half-open ranges)
- `Size` - Width/height dimensions
- `Edges` - Margin/padding insets (top, right, bottom, left)
- `Position` - Row/column grid position

## Code Style Guidelines

### Naming Conventions
- Use `PascalCase` for types and traits
- Use `snake_case` for functions, variables, and modules
- Use `SCREAMING_SNAKE_CASE` for constants
- Use `Id` suffix for key types (e.g., `ElementId`, `LayerId`)

### Key Macro Usage

The `key!` macro creates typed slotmap keys:
```rust
key!(
    pub struct ElementId;
);
```

### Feature Flags
The codebase uses several nightly Rust features:
- `slice_index_methods`
- `const_trait_impl`
- `const_cmp`, `const_range`
- `option_reference_flattening`
- `ascii_char`
- `bstr`
- `const_destruct`

### Error Handling
- Use `std::io::Result` for I/O operations
- Use `Option<T>` for optional values (especially with the `Key` trait's `option()` method)

## Testing Strategy

Tests are embedded in source files within `#[cfg(test)]` modules:

- `crates/ansi/src/attribute.rs` - 21 tests for ANSI attributes
- `crates/geometry/src/` - 41 tests for geometry primitives
- `crates/sigil/src/ui/tree/tree.rs` - 33 tests for tree operations
- `crates/sigil/src/buffer/index.rs` - Tests for buffer indexing
- `crates/sigil/tests/assertions.rs` - Custom test assertions

### Test Helpers

The `Test` struct in `tree.rs` provides a standard tree setup for tests:
```rust
let Test { root, a, b, c, tree } = Test::default();
// Creates: root -> [a, b, c]
```

## Known Issues

- There is a stack overflow issue in the `detach_middle` test in `crates/sigil/src/ui/tree/tree.rs` that causes test failures. This appears to be a runtime bug in the test setup, not the implementation.

## Key Files for Common Tasks

| Task | File |
|------|------|
| Add new element type | `crates/sigil/src/ui/element.rs` |
| Modify layout algorithm | `crates/sigil/src/ui/engine.rs` (`layout_element`) |
| Change rendering | `crates/sigil/src/ui/engine.rs` (`paint_element`, `render`) |
| Add ANSI attributes | `crates/ansi/src/attribute.rs` |
| Add buffer indexing | `crates/sigil/src/buffer/index.rs` |
| Tree operations | `crates/sigil/src/ui/tree/tree.rs` |

## Documentation

- `docs/architecture.md` - High-level rendering pipeline documentation
- `docs/notcurses-architecture.md` - Notes on notcurses integration (legacy)

## Notes

- The `sandbox` binary (`crates/sigil/src/bin/sandbox.rs`) is the primary demo/test application
- The project includes an unused `notcurses` module that was an experimental alternative backend
- The terminal crate is currently minimal (only terminal size detection)
- The UI tree uses an intrusive linked-list structure for efficient parent/sibling navigation
