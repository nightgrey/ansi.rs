# Kasten Development Guide

Development guide for working on the Kasten terminal UI layout library.

## Architecture

### Module Organization

```
src/
├── lib.rs          # Re-exports public API
├── tree.rs         # Node types, layout, measure, render functions
├── geometry.rs     # Point, Size, Rect, basic geometry
├── position.rs     # Position, Region for buffer indexing
├── layout.rs       # Constraints, Edges, Alignment
└── buffer/
    ├── mod.rs      # Buffer module re-exports
    ├── buffer.rs   # Buffer type and operations
    ├── cell.rs     # Cell type for buffer storage
    ├── index.rs    # BufferIndex trait for flexible indexing
    └── range.rs    # Range utilities
```

### Core Responsibilities

- **tree.rs**: The heart of the layout engine
  - `Node` enum: Declarative UI tree structure
  - `Content` enum: Leaf node contents (Empty, Text, Fill)
  - `layout()`: Recursively compute bounds for the tree
  - `measure()`: Calculate natural size given constraints
  - `render()`: Draw layout tree into buffer with styling

- **geometry.rs**: Basic 2D primitives
  - `Point`: (x, y) coordinates
  - `Size`: (width, height) dimensions
  - `Rect`: Rectangle with min/max points
  - Operations: area, contains, shrink

- **position.rs**: Buffer addressing
  - `Position`: (row, col) buffer coordinates
  - `Region`: Rectangular region of buffer positions
  - `RegionIter`: Iterate positions row-by-row

- **layout.rs**: Layout constraints and spacing
  - `Constraint`: Size constraints (Auto, Min, Max, Fixed, Between, Fill)
  - `Constraints`: Width and height constraints
  - `Edges`: Padding/margins (top, right, bottom, left)
  - `Alignment`: Horizontal and vertical alignment (Start, Center, End)

- **buffer/**: Terminal rendering
  - `Buffer`: 2D grid of cells with ANSI escape generation
  - `Cell`: Individual cell with character, foreground, background
  - `BufferIndex`: Trait for indexing buffer by Position, usize, Range, etc.

## Development Patterns

### Adding New Node Types

To add a new node variant:

1. **Add to `Node` enum** in `tree.rs`:
   ```rust
   pub enum Node {
       // existing variants...
       MyNewNode(MyNodeData, Box<Node>),
   }
   ```

2. **Implement `measure()`**:
   ```rust
   Node::MyNewNode(data, child) => {
       let child_size = measure(child, constraints);
       // Apply your node's sizing logic
       constraints.clamp(child_size.width, child_size.height)
   }
   ```

3. **Implement `layout()`**:
   ```rust
   Node::MyNewNode(data, child) => {
       // Compute child bounds based on your node's layout rules
       let child_layout = layout(child, child_bounds, child_constraints);
       LayoutNode::new(node, bounds, vec![child_layout])
   }
   ```

4. **Implement `render()`**:
   ```rust
   Node::MyNewNode(data, _) => {
       // Render any custom visuals, then render children
       for child in &layout.children {
           render(child, buffer, ctx);
       }
   }
   ```

5. **Add tests**: See "Testing Patterns" below

### Extending Constraints

To add a new constraint variant:

1. Add to `Constraint` enum in `layout.rs`
2. Update `clamp()` method to handle new variant
3. Update `constrain()` for composition behavior
4. Update `shrink()` if applicable
5. Add tests for the new constraint

### Building UI Trees

Common patterns for composing nodes:

```rust
// Wrapper helper for Style nodes
fn styled(style: Style, child: Node) -> Node {
    Node::Style(style, Box::new(child))
}

// Wrapper for Pad nodes
fn padded(edges: Edges, child: Node) -> Node {
    Node::Pad(edges, Box::new(child))
}

// Centering helper
fn centered(child: Node) -> Node {
    Node::Align(
        Alignment { x: Align::Center, y: Align::Center },
        Box::new(child),
    )
}

// Build complex trees
let ui = padded(
    Edges::all(2),
    Node::Stack(vec![
        styled(Style::new().bold(), Node::Base(Content::Text("Title".into()))),
        Node::Base(Content::Fill('-')),
        Node::Row(vec![
            Node::Base(Content::Text("Left".into())),
            Node::Base(Content::Fill(' ')),
            Node::Base(Content::Text("Right".into())),
        ]),
    ]),
);
```

### Custom Rendering

For low-level buffer manipulation:

```rust
// Access cells directly
buffer[Position::new(row, col)].set_char('X');
buffer[Position::new(row, col)].fg = Some(Color::Red);

// Fill region
for pos in region {
    buffer[pos].set_char('█');
}

// Write styled text
buffer.text(
    Position::new(0, 0)..Position::new(0, 10),
    "Hello",
    &Style::new().bold().foreground(Color::Blue),
);
```

## Gotchas & Edge Cases

### Inverted Rectangles

**Issue**: If `min > max`, operations like `width()` and `height()` would underflow.

**Solution**: Use `saturating_sub()` everywhere:
```rust
pub const fn width(&self) -> usize {
    self.max.x.saturating_sub(self.min.x)  // Returns 0 if inverted
}
```

**Why this matters**: Stack and Row layouts use `saturating_add()` for positioning, which can create inverted rects when children overflow available space. The recent bug fix added `saturating_sub()` when computing `child_rect` in Stack/Row to prevent this.

### Zero-Sized Bounds

**Issue**: Empty rects (width or height = 0) can cause issues in:
- Division by zero in alignment calculations
- Empty iterators when rendering

**Solution**: Check bounds before operations:
```rust
if rect.width() == 0 || rect.height() == 0 {
    return;  // Skip rendering
}
```

### Multi-Width Characters

**Issue**: Unicode grapheme clusters can be 0, 1, or 2+ cells wide (e.g., emoji, CJK).

**Handling**:
- Use `unicode-width` crate for text measurement
- `buffer.text()` handles grapheme positioning
- Cell contains the full grapheme even if multi-width

**Not yet implemented**: Proper handling of multi-cell characters that span buffer boundaries

### Overflow Handling

**Stack behavior**: Children that exceed available height are still laid out, but their bounds may be clipped:
```rust
let remaining_h = bounds.height().saturating_sub(y - bounds.y());
// remaining_h becomes 0 if we've exceeded bounds
```

This allows overflow to be rendered if the buffer is large enough, but prevents underflow.

### Style Composition

Styles are composed with bitwise OR:
```rust
let ctx = ctx.add(&style);  // Merges styles
```

**Implication**: Nested Style nodes accumulate attributes. Later styles override conflicting attributes (e.g., colors).

## Testing Guide

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test --lib tree

# Run with output
cargo test -- --nocapture

# Run tests in release mode (catch optimization issues)
cargo test --release
```

### Test Organization

```
src/
├── tree.rs
│   └── #[cfg(test)] mod tests { ... }
├── geometry.rs
│   └── #[cfg(test)] mod tests { ... }
└── buffer/
    └── index.rs
        └── #[cfg(test)] mod tests { ... }

tests/
├── test_utils/
│   ├── mod.rs
│   ├── builders.rs
│   └── assertions.rs
└── integration/
    ├── layout_integration.rs
    └── rendering_integration.rs
```

### Testing Patterns

**Unit tests** in the same file as the code:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_stack_basic() {
        let node = Node::Stack(vec![
            Node::Base(Content::Text("A".into())),
            Node::Base(Content::Text("B".into())),
        ]);

        let bounds = Rect::new((0, 0), (10, 10));
        let layout = layout(&node, bounds, Constraints::Max(10, 10));

        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.children[0].bounds.y(), 0);
        assert_eq!(layout.children[1].bounds.y(), 1);
    }
}
```

**Integration tests** in `tests/`:
```rust
// tests/integration/layout_integration.rs
use kasten::*;

#[test]
fn test_complex_nested_layout() {
    // Build complex tree and verify end-to-end behavior
}
```

**Test builders** in `tests/test_utils/builders.rs`:
```rust
pub fn rect(x: usize, y: usize, w: usize, h: usize) -> Rect {
    Rect::new((x, y), (x + w, y + h))
}

pub fn text_node(s: &str) -> Node {
    Node::Base(Content::Text(s.into()))
}
```

**Custom assertions** in `tests/test_utils/assertions.rs`:
```rust
pub fn assert_rect_valid(rect: &Rect) {
    assert!(rect.min.x <= rect.max.x);
    assert!(rect.min.y <= rect.max.y);
}
```

### Regression Tests

The recent inverted rectangle bug should have a regression test:

```rust
#[test]
fn test_stack_inverted_bounds_prevented() {
    // Create a stack where children exceed available height
    let node = Node::Stack(vec![
        Node::Base(Content::Text("A".repeat(100))),
        Node::Base(Content::Text("B".repeat(100))),
    ]);

    let bounds = Rect::new((0, 0), (10, 1)); // Only 1 row available
    let layout = layout(&node, bounds, Constraints::Max(10, 1));

    // Verify no child has inverted bounds (min > max)
    for child in &layout.children {
        assert_rect_valid(&child.bounds);
        assert!(child.bounds.width() <= 10);
    }
}
```

### Coverage Tools

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

**Target**: 80%+ coverage for core modules (tree, buffer, geometry, position).

## Common Development Tasks

### Add a new example

1. Create `examples/my_example.rs`
2. Add `[[example]]` section to `Cargo.toml` if needed
3. Run with `cargo run --example my_example`

### Debug layout issues

```rust
// Add debug prints in layout()
eprintln!("Node: {:?}, Bounds: {:?}", node, bounds);

// Visualize layout tree
fn print_layout(layout: &LayoutNode, depth: usize) {
    let indent = "  ".repeat(depth);
    eprintln!("{}Node: {:?}, Bounds: {:?}", indent, layout.node, layout.bounds);
    for child in &layout.children {
        print_layout(child, depth + 1);
    }
}
```

### Profile performance

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin sandbox

# Open flamegraph.svg
```

## Todo List

### Performance
- [ ] Benchmark layout performance with deep trees (100+ nodes)
- [ ] Optimize buffer allocation (pool reused buffers?)
- [ ] Cache measurement results (memoization for pure measure functions)
- [ ] Profile ANSI escape generation

### Features
- [ ] Text wrapping support (currently commented out in tree.rs)
- [ ] Scrolling regions (viewport into larger buffer)
- [ ] Border/box drawing helpers (common pattern)
- [ ] Animation/transitions (interpolate between layouts)
- [ ] Custom render hooks (user-defined rendering for nodes)
- [ ] Layout debugging mode (visualize bounds)

### Testing
- [ ] Property-based testing with proptest (fuzz constraints, sizes)
- [ ] Fuzzing for buffer operations (AFL or libfuzzer)
- [ ] Reach 80%+ coverage on core modules
- [ ] Benchmark suite (criterion)
- [ ] Visual regression tests (screenshot comparison)

### Documentation
- [ ] Tutorial series (beginner to advanced)
- [ ] Video walkthrough
- [ ] Comparison with other TUI libraries (ratatui, tui-rs, termwiz)
- [ ] Architecture decision records (ADRs)
- [ ] Contributing guide

### API Design
- [ ] Builder pattern for Node construction?
- [ ] Macro for ergonomic tree building (like `html!` in Yew)?
- [ ] Separate layout and style into different concerns?
- [ ] Event handling integration points?
- [ ] Diff/patch for efficient re-renders?

### Cleanup
- [ ] Remove commented-out TextWrap code or implement it
- [ ] Audit all `unsafe` usage (currently in buffer rendering)
- [ ] Consistent error handling (Results vs panics)
- [ ] Stabilize public API (semantic versioning)

## Code Style

### Conventions
- Use `saturating_*` arithmetic for all size/position math
- Prefer `const fn` where possible
- Document panics in doc comments
- Mark `unsafe` with SAFETY comments
- Keep functions small (< 50 lines)

### Naming
- Types: `PascalCase`
- Functions: `snake_case`
- Constructors: Associated functions like `new()`, `from_*()`, or `Fixed()` (for Constraints)
- Avoid Hungarian notation

### Comments
- Explain "why", not "what"
- Use `// SAFETY:` for unsafe code
- Use `// TODO:` for known issues
- Use `// NOTE:` for important clarifications

## Debugging Tips

### Visualize buffer contents

```rust
// Debug print buffer
for row in 0..buffer.height() {
    for col in 0..buffer.width() {
        let cell = &buffer[Position::new(row, col)];
        print!("{}", cell.grapheme);
    }
    println!();
}
```

### Check constraint satisfaction

```rust
fn verify_constraints(size: Size, constraints: Constraints) {
    if let Some(min_w) = constraints.width.min() {
        assert!(size.width >= min_w);
    }
    if let Some(max_w) = constraints.width.max() {
        assert!(size.width <= max_w);
    }
    // Similar for height...
}
```

### Validate layout tree

```rust
fn validate_layout(layout: &LayoutNode, parent_bounds: Rect) {
    // Child bounds should be within parent bounds (or we're overflowing)
    // This might fail during overflow, which is okay

    for child in &layout.children {
        validate_layout(child, layout.bounds);
    }
}
```

## Resources

- [Terminal Escape Sequences](https://en.wikipedia.org/wiki/ANSI_escape_code)
- [Unicode Width](https://unicode.org/reports/tr11/)
- [Ratatui](https://github.com/ratatui-org/ratatui) - Similar TUI library
- [Cassowary](https://github.com/dylanede/cassowary-rs) - Constraint solver
