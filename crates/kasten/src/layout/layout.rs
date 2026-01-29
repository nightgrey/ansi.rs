use crate::{Align, Alignment, Buffer, Constraints, Content, Edges, LayoutNode, Node, Rect};
use ansi::Style;
use derive_more::{Deref, DerefMut};
use std::ops::BitOr;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Layout<'a> {
    #[deref]
    #[deref_mut]
    pub root: LayoutNode<'a>,
    pub bounds: Rect,
    pub constraints: Constraints,
    pub context: LayoutContext,
}

impl<'a> Layout<'a> {
    pub const EMPTY: Layout<'static> = Layout {
        root: LayoutNode {
            node: &Node::Base(Content::Empty),
            children: vec![],
            bounds: Rect::ZERO,
        },
        bounds: Rect::ZERO,
        constraints: Constraints::Auto(),
        context: LayoutContext::EMPTY,
    };

    pub fn new(root: &'a Node, bounds: Rect) -> Self {
        let constraints = Constraints::Fixed(bounds.width(), bounds.height());
        Self {
            root: root.layout(bounds, constraints),
            bounds,
            constraints,
            context: LayoutContext::EMPTY,
        }
    }

    pub fn replace(&mut self, root: &'a Node) {
        self.root = root.layout(self.bounds, self.constraints);
    }

    /// Render a laid out node tree into a buffer.
    ///
    /// This is the third and final phase of rendering (after measure and layout).
    /// It recursively draws each node in the tree to the provided buffer,
    /// applying styles from the context and respecting the computed bounds.
    ///
    /// # Arguments
    ///
    /// * `layout` - The laid out node tree (from [`layout()`])
    /// * `buffer` - The buffer to render into (will be mutated)
    /// * `ctx` - The rendering context (tracks accumulated styles)
    ///
    /// # Rendering Rules
    ///
    /// Different node types render differently:
    ///
    /// - **Empty**: Renders nothing
    /// - **Text**: Writes text to buffer with current style
    /// - **Fill**: Fills the bounds with the specified character
    /// - **Style**: Updates context with new style, renders children
    /// - **Pad/Size/Align**: Renders children with their bounds
    /// - **Stack/Row/Layer**: Renders all children in order
    ///
    /// For Style nodes, if a background color is specified, it fills the entire
    /// bounds before rendering children.
    ///
    /// # Safety
    ///
    /// This function uses `unsafe` buffer access for performance. It assumes that
    /// the layout phase has computed valid bounds that fit within the buffer.
    pub fn render(&self, buffer: &mut Buffer) {
        self.root.render(buffer, &self.context);
    }
}

/// Rendering context that accumulates styles as the tree is traversed.
///
/// The context tracks the current style state, which is composed from
/// all ancestor [`Node::Style`] nodes. Styles are merged using bitwise OR,
/// with later styles overriding conflicting attributes.
///
/// Create a default context for rendering:
///
/// ```rust
/// # use kasten::*;
/// let ctx = Context::default();
/// // ... use in render() ...
/// ```
#[derive(Clone, Default, Debug)]
pub struct LayoutContext {
    pub(crate) style: Style,
}

impl LayoutContext {
    pub const EMPTY: Self = Self {
        style: Style::EMPTY,
    };

    pub const fn empty() -> Self {
        Self::EMPTY
    }
    /// Add a style to the context, creating a new context with merged styles.
    ///
    /// Styles are composed using bitwise OR. Later styles override conflicting
    /// attributes (e.g., if both specify a foreground color, the new one wins).
    pub fn compose(&self, style: &Style) -> Self {
        Self {
            style: style.bitor(self.style),
        }
    }
}

#[cfg(test)]
mod tests {
    use ansi::Color;
    use super::*;
    use crate::{layout::macros::*, text, Align, Alignment, Cell, Edges, Row};

    fn render_and_layout(node: &Node, width: usize, height: usize) -> (Buffer, Layout)   {
        let bounds = Rect::bounds(0, 0, width, height);
        let layout = Layout::new(&node, bounds);
        let mut buffer = Buffer::new(bounds);
        layout.render(&mut buffer);
        (buffer, layout)
    }
    fn render(node: &Node, width: usize, height: usize) -> Buffer {
        let bounds = Rect::bounds(0, 0, width, height);
        let layout = Layout::new(&node, bounds);
        let mut buffer = Buffer::new(bounds);
        layout.render(&mut buffer);
        buffer
    }

    fn layout(node: &Node, width: usize, height: usize) -> Layout {
        let bounds = Rect::bounds(0, 0, width, height);
        let layout = Layout::new(&node, bounds);
        let mut buffer = Buffer::new(bounds);
        layout.render(&mut buffer);
        layout
    }

    macro_rules! row {
        ($buffer:expr, $row:expr) => {
            $buffer[Row::from($row)].iter().collect::<String>()
        };
    }

    macro_rules! assert_row {
        ($buffer:expr, $row:expr, $expected:expr) => {
            assert_eq!(
                row!($buffer, $row),
                $expected,
                "Row {row:?} should be {expected:?}",
                row = $row,
                expected = $expected
            );
        };
    }

    macro_rules! assert_row_trimmed {
        ($buffer:expr, $row:expr, $expected:expr) => {{
            let actual = if ($buffer[Row::from($row)].iter().any(|cell| cell.content() != Cell::SPACE)) {
                $buffer[Row::from($row)].iter().collect::<String>()
            } else {
                "".to_string()
            };

            assert_eq!(
                actual,
                $expected,
                "Row({row:?}) contains {actual:?}.",
                actual = actual,
                row = $row,
            );
        }};
    }

    macro_rules! assert_row_empty {
        ($buffer:expr, $row:expr) => {
            assert_eq!(
                row!($buffer, $row).trim(),
                "",
                "Row {row:?} should be empty",
                row = $row,
            );
        };
    }

    macro_rules! assert_text {
        ($buffer:expr, $expected:expr) => {
            assert_eq!(
                &$buffer.lines().map(|line| line.trim().to_string()).collect::<Vec<_>>().join("\n"),
                $expected,
                "Expected buffer text to be {expected:?}",
                expected = $expected,
            );
        };
    }

    macro_rules! assert_rect {
        ($rect:expr) => {
            assert!($rect.min.x <= $rect.max.x, "Inverted x: {:?}", $rect);
            assert!($rect.min.y <= $rect.max.y, "Inverted y: {:?}", $rect);
        };
    }

    // === Layout Tests ===

    #[test]
    fn test_layout_stack_basic() {
        let node = Node::Stack(vec![text!("A"), text!("B"), text!("C")]);

        let bounds = Rect::bounds(0, 0, 10, 10);
        let layout_tree = Layout::new(&node, bounds);

        assert_eq!(layout_tree.children.len(), 3);
        // Stack should position children vertically
        assert_eq!(layout_tree.children[0].bounds.y(), 0);
        assert_eq!(layout_tree.children[1].bounds.y(), 1);
        assert_eq!(layout_tree.children[2].bounds.y(), 2);
    }

    #[test]
    fn test_layout_stack_with_padding() {
        let node = Node::Pad(
            Edges::all(1),
            Box::new(Node::Stack(vec![text!("A"), text!("B")])),
        );

        let bounds = Rect::bounds(0, 0, 10, 10);
        let layout_tree = Layout::new(&node, bounds);

        // Padding should reduce inner bounds
        let inner = &layout_tree.children[0];
        assert_eq!(inner.bounds.x(), 1);
        assert_eq!(inner.bounds.y(), 1);
        assert_eq!(inner.bounds.width(), 8); // 10 - 2 (left + right padding)
        assert_eq!(inner.bounds.height(), 8); // 10 - 2 (top + bottom padding)
    }

    #[test]
    fn test_layout_stack_overflow() {
        // Create stack with more content than available space
        let node = Node::Stack(vec![
            text!("Line 1"),
            text!("Line 2"),
            text!("Line 3"),
            text!("Line 4"),
            text!("Line 5"),
        ]);

        let bounds = Rect::bounds(0, 0, 10, 3); // Only 3 rows available
        let layout_tree = Layout::new(&node, bounds);

        assert_eq!(layout_tree.children.len(), 5);
        // All children should be laid out, even if they overflow
        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
        }
    }

    #[test]
    fn test_layout_stack_empty() {
        let node = Node::Stack(vec![]);
        let bounds = Rect::bounds(0, 0, 10, 10);
        let layout_tree = Layout::new(&node, bounds);

        assert_eq!(layout_tree.children.len(), 0);
    }

    #[test]
    fn test_layout_stack_inverted_bounds_prevented() {
        // Regression test: Stack should not create inverted rectangles
        let node = Node::Stack(vec![
            text!("A".repeat(100).as_str()),
            text!("B".repeat(100).as_str()),
        ]);

        let bounds = Rect::bounds(0, 0, 10, 1); // Very limited space
        let layout_tree = Layout::new(&node, bounds);

        // Verify no child has inverted bounds
        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
            assert!(child.bounds.width() <= 10);
        }
    }

    #[test]
    fn test_layout_row_basic() {
        let node = Node::Row(vec![text!("A"), text!("B"), text!("C")]);

        let bounds = Rect::bounds(0, 0, 10, 10);
        let layout_tree = Layout::new(&node, bounds);

        assert_eq!(layout_tree.children.len(), 3);
        // Row should position children horizontally
        assert_eq!(layout_tree.children[0].bounds.x(), 0);
        assert_eq!(layout_tree.children[1].bounds.x(), 1);
        assert_eq!(layout_tree.children[2].bounds.x(), 2);
    }

    #[test]
    fn test_layout_row_overflow() {
        let node = Node::Row(vec![text!("Word1"), text!("Word2"), text!("Word3")]);

        let bounds = Rect::bounds(0, 0, 10, 5); // Limited width
        let layout_tree = Layout::new(&node, bounds);

        // All children should have valid bounds
        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
        }
    }

    #[test]
    fn test_layout_row_inverted_bounds_prevented() {
        // Regression test: Row should not create inverted rectangles
        let node = Node::Row(vec![
            text!("LongWord1"),
            text!("LongWord2"),
            text!("LongWord3"),
        ]);

        let bounds = Rect::bounds(0, 0, 5, 10); // Very limited width
        let layout_tree = Layout::new(&node, bounds);

        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
        }
    }

    #[test]
    fn test_layout_layer_overlapping() {
        let node = Node::Layer(vec![text!("Background"), text!("Foreground")]);

        let bounds = Rect::bounds(0, 0, 20, 10);
        let layout_tree = Layout::new(&node, bounds);

        assert_eq!(layout_tree.children.len(), 2);
        // Both children should share the same bounds
        assert_eq!(layout_tree.children[0].bounds, bounds);
        assert_eq!(layout_tree.children[1].bounds, bounds);
    }

    #[test]
    fn test_layout_align_center() {
        let node = Node::Align(
            Alignment {
                x: Align::Center,
                y: Align::Center,
            },
            Box::new(text!("Hi")),
        );

        let bounds = Rect::bounds(0, 0, 10, 10);
        let layout_tree = Layout::new(&node, bounds);

        let child = &layout_tree.children[0];
        // "Hi" is 2 chars wide, 1 tall, should be centered in 10x10
        assert_eq!(child.bounds.x(), 4); // (10 - 2) / 2
        assert_eq!(child.bounds.y(), 4); // (10 - 1) / 2 (rounds down)
    }

    #[test]
    fn test_layout_align_edges() {
        let node = Node::Align(
            Alignment {
                x: Align::End,
                y: Align::Start,
            },
            Box::new(text!("Text")),
        );

        let bounds = Rect::bounds(0, 0, 20, 10);
        let layout_tree = Layout::new(&node, bounds);

        let child = &layout_tree.children[0];
        // "Text" is 4 chars wide
        assert_eq!(child.bounds.x(), 16); // 20 - 4 (aligned to end)
        assert_eq!(child.bounds.y(), 0); // Start alignment
    }

    #[test]
    fn test_layout_pad_shrinks_bounds() {
        let node = Node::Pad(Edges::new(1, 2, 3, 4), Box::new(text!("Test")));

        let bounds = Rect::bounds(0, 0, 20, 20);
        let layout_tree = Layout::new(&node, bounds);

        let child = &layout_tree.children[0];
        assert_eq!(child.bounds.x(), 4); // left padding
        assert_eq!(child.bounds.y(), 1); // top padding
        assert_eq!(child.bounds.width(), 14); // 20 - 4 - 2
        assert_eq!(child.bounds.height(), 16); // 20 - 1 - 3
    }

    #[test]
    fn test_layout_size_constraints() {
        let ui = size!(
            Constraints::Fixed(15, 8) => text!("Constrained")
        );
        let layout = layout(&ui, 100, 100);

        // The Size node passes constraints to its child, but the child's bounds
        // are still determined by the parent's provided space
        // The measured size of "Constrained" is 11 chars × 1 row
        let child = &layout.children[0];
        // With Fixed(15, 8) constraints and large parent bounds,
        // the child will use the parent's full bounds
        assert_eq!(child.bounds.width(), 100); // Uses parent's width
    }

    #[test]
    fn test_render_stack() {
        let node = Node::Stack(vec![text!("First"), text!("Second"), text!("Third")]);

        let bounds = Rect::bounds(0, 0, 4, 3);
        let layout_tree = Layout::new(&node, bounds);

        let mut buffer = Buffer::new(bounds);
        layout_tree.render(&mut buffer);

        assert_row!(&buffer, 0, "Firs");
        assert_row!(&buffer, 1, "Seco");
        assert_row!(&buffer, 2, "Thir");
    }

    #[test]
    fn test_render_text() {
        let node = Node::Base(Content::Text("Out of bounds\nMore".into()));

        let buffer = render(&node, 4, 2);

        assert_row!(&buffer, 0, "Out ");
        assert_row_empty!(&buffer, 1);
    }


    #[test]
    fn test_render_align() {
        let ui = align!(
            Alignment::CENTER => text!("Text")
        );
        let buffer = render(&ui, 10, 10);

        let lines = buffer.lines();
        assert_row_trimmed!(&buffer, 0, "");
        assert_row_trimmed!(&buffer, 1, "");
        assert_row_trimmed!(&buffer, 2, "");
        assert_row_trimmed!(&buffer, 3, "");
        assert_row_trimmed!(&buffer, 4, "   Text   ");
        assert_row_trimmed!(&buffer, 5, "");
        assert_row_trimmed!(&buffer, 6, "");
        assert_row_trimmed!(&buffer, 7, "");
        assert_row_trimmed!(&buffer, 8, "");
        assert_row_trimmed!(&buffer, 9, "");
    }

    #[test]
    fn test_render_size_constrained_stack_item_with_aligned_text() {
        let ui = stack![
            size!(
                Constraints::Fixed(20, 5) => align!(Alignment::CENTER => text!("Hello Ay! 👋")           )
            ),
            text!("After")
        ];

        let (buffer, layout) = render_and_layout(&ui, 20, 20);

        // This here is 1 because it takes the text's size as bounds.
        assert_eq!(layout.children[0].children[0].bounds.height(), 5);

        assert_row_empty!(&buffer, 0);
        assert_row_empty!(&buffer, 1);
        assert_row_trimmed!(&buffer, 2, "    Hello Ay! 👋     ");
        assert_row_empty!(&buffer, 3);
    }
}
