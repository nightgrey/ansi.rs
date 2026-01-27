# Kasten

A declarative terminal UI layout library for Rust.

## Overview

Kasten is a lightweight, composable library for building terminal user interfaces with a declarative, tree-based API similar to React or Flutter. It provides a constraint-based layout system, buffer-based rendering, and ANSI styling integration.

The name "kasten" comes from German for "box", reflecting the library's focus on rectangular layout regions.

## Key Features

- **Declarative UI Trees**: Build interfaces by composing tree structures rather than imperatively drawing to a canvas
- **Constraint-Based Layout**: Flexible sizing with Auto, Min, Max, Fixed, Between, and Fill constraints
- **Three-Phase Rendering**: Clean separation between measure, layout, and render phases
- **ANSI Styling**: First-class support for terminal colors, bold, italic, and other text attributes
- **Buffer-Based Rendering**: Efficient cell-by-cell buffer manipulation with Unicode support
- **Zero Dependencies** (aside from workspace deps): Minimal footprint for terminal applications

## Quick Start

```rust
use ansi::{Color, Style};
use kasten::{layout, render, Buffer, Context, Constraints, Edges, Node, Content, Rect};

fn main() {
    // 1. Build a UI tree
    let ui = Node::Style(
        Style::new().bold().background(Color::Blue).foreground(Color::White),
        Box::new(Node::Pad(
            Edges::all(1),
            Box::new(Node::Stack(vec![
                Node::Base(Content::Text("Header".into())),
                Node::Base(Content::Text("Body text".into())),
                Node::Base(Content::Fill('.')),
            ])),
        )),
    );

    // 2. Create a buffer for the terminal
    let mut buffer = Buffer::new(Rect::new((0, 0), (80, 24)));

    // 3. Layout the UI tree
    let tree = layout(
        &ui,
        buffer.bounds,
        Constraints::Fixed(buffer.bounds.width(), buffer.bounds.height()),
    );

    // 4. Render to the buffer
    let ctx = Context::default();
    render(&tree, &mut buffer, &ctx);

    // 5. Write to terminal
    use ansi::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_escape(&buffer).unwrap();
}
```

## Core Concepts

### Node Tree Structure

UIs are built by composing `Node` variants:

- **`Base(Content)`**: Leaf nodes containing actual content
  - `Content::Empty`: No content
  - `Content::Text(String)`: Text to display
  - `Content::Fill(char)`: Fill region with a character

- **`Style(Style, Box<Node>)`**: Apply ANSI styling to child
- **`Pad(Edges, Box<Node>)`**: Add padding around child
- **`Size(Constraints, Box<Node>)`**: Apply size constraints to child
- **`Align(Alignment, Box<Node>)`**: Align child within available space

- **`Stack(Vec<Node>)`**: Stack children vertically
- **`Row(Vec<Node>)`**: Arrange children horizontally
- **`Layer(Vec<Node>)`**: Overlay children (Z-ordering)

### Three-Phase Workflow

1. **Measure**: Determine the natural size of each node given constraints
   ```rust
   let size = measure(&node, Constraints::Max(80, 24));
   ```

2. **Layout**: Assign positions and bounds to each node
   ```rust
   let layout_tree = layout(&node, bounds, constraints);
   ```

3. **Render**: Draw the layout tree into a buffer
   ```rust
   render(&layout_tree, &mut buffer, &context);
   ```

### Constraints System

Constraints control how nodes size themselves:

```rust
// Take exactly 20 columns and 5 rows
Constraint::Fixed(20)

// At most 80 columns
Constraint::Max(80)

// At least 10 rows
Constraint::Min(10)

// Between 5 and 50 columns
Constraint::Between(5, 50)

// Expand to fill available space
Constraint::Fill

// Use natural size
Constraint::Auto
```

Constraints can be composed:

```rust
let constraints = Constraints::new(
    Constraint::Fixed(40),  // width
    Constraint::Max(20),    // height
);
```

### Geometry Types

- **`Point`**: 2D coordinate (x, y)
- **`Size`**: Dimensions (width, height)
- **`Rect`**: Rectangle defined by min and max points
- **`Edges`**: Padding/margin (top, right, bottom, left)
- **`Alignment`**: Positioning (Start, Center, End for x and y)

### Buffer and Cell System

The `Buffer` is a 2D grid of `Cell` objects:

```rust
let mut buffer = Buffer::new(Rect::new((0, 0), (80, 24)));

// Access cells by position
buffer[Position::new(5, 10)].set_char('X');

// Write text with styling
buffer.text(
    Position::new(0, 0)..Position::new(1, 10),
    "Hello",
    &Style::new().bold(),
);

// Output as ANSI escape sequences
use ansi::io::Write;
stdout.write_escape(&buffer)?;
```

## Examples

See the [examples](examples/) directory:

- `sandbox.rs`: Basic usage demonstration
- More examples coming soon...

Run examples with:

```bash
cargo run --bin sandbox
```

## Layout Patterns

### Centering Content

```rust
Node::Align(
    Alignment { x: Align::Center, y: Align::Center },
    Box::new(Node::Base(Content::Text("Centered!".into()))),
)
```

### Vertical Stack with Padding

```rust
Node::Pad(
    Edges::all(2),
    Box::new(Node::Stack(vec![
        Node::Base(Content::Text("Line 1".into())),
        Node::Base(Content::Text("Line 2".into())),
        Node::Base(Content::Text("Line 3".into())),
    ])),
)
```

### Horizontal Row

```rust
Node::Row(vec![
    Node::Base(Content::Text("Left".into())),
    Node::Base(Content::Fill(' ')),
    Node::Base(Content::Text("Right".into())),
])
```

### Layered Content (Overlays)

```rust
Node::Layer(vec![
    Node::Base(Content::Fill('█')),  // Background
    Node::Align(
        Alignment { x: Align::Center, y: Align::Center },
        Box::new(Node::Base(Content::Text("Overlay".into()))),
    ),
])
```

### Styled Text

```rust
Node::Style(
    Style::new().foreground(Color::Yellow).bold(),
    Box::new(Node::Base(Content::Text("Warning!".into()))),
)
```

## Architecture

Kasten is organized into several modules:

- `tree`: Node types and core layout/measure/render functions
- `geometry`: Point, Size, Rect primitives
- `position`: Position and Region for buffer indexing
- `layout`: Constraints, Edges, Alignment types
- `buffer`: Buffer and Cell for terminal rendering

## API Reference

Full API documentation is available on [docs.rs](https://docs.rs/kasten) (once published).

Generate documentation locally:

```bash
cargo doc --open --no-deps
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. New public APIs have documentation comments

## License

[License information to be added]

## Comparison with Other TUI Libraries

Kasten focuses on being a low-level layout primitive. It differs from libraries like `ratatui` in that:

- Kasten provides the layout engine, not a full TUI framework
- No built-in widgets, event handling, or terminal management
- Lower-level API for building custom rendering systems
- Smaller scope and lighter weight

Use Kasten when you want fine-grained control over your terminal rendering pipeline, or as a foundation for building your own TUI framework.
