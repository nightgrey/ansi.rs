use std::ops::BitOr;
use derive_more::Deref;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use ansi::{Color, Escape, Style};
use ansi::io::Write;
use crate::{Align, Region, Alignment, Buffer, BufferIndex, Constraint, Constraints, Edges, Point, Rect, Size, Row, Position};
use crate::runes::DisplayWidth;

/// Content types for leaf nodes in the UI tree.
///
/// Content represents the actual visual elements that will be rendered,
/// as opposed to layout nodes which only affect positioning and styling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    /// Empty content that takes no space.
    ///
    /// Useful for placeholder nodes or conditional rendering.
    Empty,

    /// Text content to be displayed.
    ///
    /// The text will be measured using Unicode width and rendered
    /// with any styles applied by parent [`Node::Style`] nodes.
    Text(String),

    /// Fill a region with a repeated character.
    ///
    /// Useful for backgrounds, separators, or decorative elements.
    /// The character will be written to every cell in the node's bounds.
    Fill(char),
}

/// A node in the UI tree, representing either content or a layout operation.
///
/// Nodes are the building blocks of Kasten UIs. They form a tree structure where:
/// - Leaf nodes ([`Node::Base`]) contain the actual content to render
/// - Container nodes ([`Node::Stack`], [`Node::Row`], [`Node::Layer`]) arrange children
/// - Modifier nodes ([`Node::Style`], [`Node::Pad`], etc.) transform their child
///
/// ## Example
///
/// ```rust
/// use kasten::{Node, Content, Edges};
/// use ansi::{Style, Color};
///
/// let ui = Node::Style(
///     Style::new().bold(),
///     Box::new(Node::Pad(
///         Edges::all(2),
///         Box::new(Node::Stack(vec![
///             Node::Base(Content::Text("Line 1".into())),
///             Node::Base(Content::Text("Line 2".into())),
///         ])),
///     )),
/// );
/// ```
#[derive(Clone, Debug)]
pub enum Node {
    /// Leaf node containing actual content.
    ///
    /// This is the only node type that directly produces visual output.
    /// See [`Content`] for available content types.
    Base(Content),

    /// Apply ANSI styling to a child node.
    ///
    /// Styles are composable - nested Style nodes accumulate attributes.
    /// Later styles override conflicting attributes (e.g., colors).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content}; use ansi::{Style, Color};
    /// Node::Style(
    ///     Style::new().foreground(Color::Red).bold(),
    ///     Box::new(Node::Base(Content::Text("Error!".into()))),
    /// )
    /// # ;
    /// ```
    Style(Style, Box<Node>),

    /// Add padding (internal spacing) around a child node.
    ///
    /// Padding reduces the available space for the child by the specified
    /// [`Edges`] amounts.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content, Edges};
    /// Node::Pad(
    ///     Edges::all(2),  // 2 cells of padding on all sides
    ///     Box::new(Node::Base(Content::Text("Padded".into()))),
    /// )
    /// # ;
    /// ```
    Pad(Edges, Box<Node>),

    /// Apply size constraints to a child node.
    ///
    /// The constraints override the child's natural size, clamping it
    /// to the specified dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content, Constraints, Constraint};
    /// Node::Size(
    ///     Constraints::new(Constraint::Fixed(20), Constraint::Max(10)),
    ///     Box::new(Node::Base(Content::Text("Constrained".into()))),
    /// )
    /// # ;
    /// ```
    Size(Constraints, Box<Node>),

    /// Align a child node within the available space.
    ///
    /// The child is measured, then positioned according to the alignment.
    /// Any space not used by the child remains empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content, Alignment, Align};
    /// Node::Align(
    ///     Alignment { x: Align::Center, y: Align::Center },
    ///     Box::new(Node::Base(Content::Text("Centered!".into()))),
    /// )
    /// # ;
    /// ```
    Align(Alignment, Box<Node>),

    /// Arrange children vertically (top to bottom).
    ///
    /// Children are laid out in order, each starting immediately below
    /// the previous child. If children exceed available height, later
    /// children may be clipped.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content};
    /// Node::Stack(vec![
    ///     Node::Base(Content::Text("First".into())),
    ///     Node::Base(Content::Text("Second".into())),
    ///     Node::Base(Content::Text("Third".into())),
    /// ])
    /// # ;
    /// ```
    Stack(Vec<Node>),

    /// Arrange children horizontally (left to right).
    ///
    /// Children are laid out in order, each starting immediately after
    /// the previous child. If children exceed available width, later
    /// children may be clipped.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content};
    /// Node::Row(vec![
    ///     Node::Base(Content::Text("Left".into())),
    ///     Node::Base(Content::Fill(' ')),
    ///     Node::Base(Content::Text("Right".into())),
    /// ])
    /// # ;
    /// ```
    Row(Vec<Node>),

    /// Overlay children in Z-order (first is bottom, last is top).
    ///
    /// All children share the same bounds and are rendered in order.
    /// Later children can overlap earlier ones.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use kasten::{Node, Content, Alignment, Align};
    /// Node::Layer(vec![
    ///     Node::Base(Content::Fill('█')),  // Background
    ///     Node::Align(
    ///         Alignment { x: Align::Center, y: Align::Center },
    ///         Box::new(Node::Base(Content::Text("Overlay".into()))),
    ///     ),
    /// ])
    /// # ;
    /// ```
    Layer(Vec<Node>),
}


/// The result of laying out a node tree with resolved bounds.
///
/// A `LayoutNode` represents a node from the UI tree along with its computed
/// rectangular bounds and laid out children. This forms a parallel tree structure
/// where each node has been assigned a position and size.
///
/// The `bounds` field contains the resolved [`Rect`] for this node, which has been
/// computed based on:
/// - The node's type and constraints
/// - Available space from the parent
/// - Measured sizes of children (for containers)
///
/// ## Derefs to Node
///
/// `LayoutNode` implements `Deref<Target = Node>`, so you can access the original
/// node's methods directly:
///
/// ```rust
/// # use kasten::*;
/// # let node = Node::Base(Content::Empty);
/// # let layout = LayoutNode::leaf(&node, Rect::ZERO);
/// // Access the underlying node through deref
/// match &*layout {
///     Node::Base(content) => { /* ... */ },
///     _ => {}
/// }
/// ```
#[derive(Debug, Deref)]
pub struct LayoutNode<'a> {
    /// The original node from the UI tree.
    #[deref]
    pub node: &'a Node,

    /// The computed rectangular bounds for this node.
    pub bounds: Rect,

    /// Laid out children with their own computed bounds.
    pub children: Vec<LayoutNode<'a>>,
}

impl<'a> LayoutNode<'a> {
    /// Create a new layout node with children.
    pub fn new(node: &'a Node, bounds: Rect, children: Vec<LayoutNode<'a>>) -> Self {
        Self { node, bounds, children }
    }

    /// Create a leaf layout node with no children.
    pub fn leaf(node: &'a Node, bounds: Rect) -> Self {
        Self::new(node, bounds, vec![])
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
#[derive(Clone, Default)]
#[derive(Debug)]
pub struct Context {
    style: Style,
}

impl Context {
    /// Add a style to the context, creating a new context with merged styles.
    ///
    /// Styles are composed using bitwise OR. Later styles override conflicting
    /// attributes (e.g., if both specify a foreground color, the new one wins).
    fn add(&self, style: &Style) -> Self {
        Self { style: style.bitor(self.style) }
    }
}

/// Layout a node tree, assigning bounds to each node.
///
/// This is the second phase of rendering (after measure, before render).
/// It recursively computes the position and size of every node in the tree,
/// creating a [`LayoutNode`] tree with resolved bounds.
///
/// # Arguments
///
/// * `node` - The root of the UI tree to layout
/// * `bounds` - The rectangular region available for this node
/// * `constraints` - Size constraints to apply during layout
///
/// # Returns
///
/// A [`LayoutNode`] tree with bounds assigned to every node.
///
/// # Example
///
/// ```rust
/// use kasten::{layout, Node, Content, Rect, Constraints};
///
/// let node = Node::Stack(vec![
///     Node::Base(Content::Text("Hello".into())),
///     Node::Base(Content::Text("World".into())),
/// ]);
///
/// let bounds = Rect::new((0, 0), (80, 24));
/// let layout_tree = layout(&node, bounds, Constraints::Max(80, 24));
///
/// // layout_tree now contains bounds for each child
/// assert_eq!(layout_tree.children.len(), 2);
/// ```
///
/// # How Layout Works
///
/// Different node types handle layout differently:
///
/// - **Base**: Uses the provided bounds directly (already measured)
/// - **Style/Pad/Size/Align**: Modifies bounds/constraints for child
/// - **Stack**: Arranges children vertically, top to bottom
/// - **Row**: Arranges children horizontally, left to right
/// - **Layer**: All children share the same bounds (overlapping)
///
/// For containers (Stack/Row), layout measures each child to determine its size,
/// then assigns it a position within the container's bounds.
pub fn layout<'a>(node: &'a Node, bounds: Rect, constraints: Constraints) -> LayoutNode<'a> {
    match node {
        Node::Base(_) => LayoutNode::leaf(node, bounds),

        Node::Style(_, child) => {
            let child_layout = layout(child, bounds, constraints);
            LayoutNode::new(node, bounds, vec![child_layout])
        }

        Node::Pad(edges, child) => {
            let inner_rect = bounds.shrink(edges);
            let inner_ct = constraints.shrink(edges);
            let child_layout = layout(child, inner_rect, inner_ct);
            LayoutNode::new(node, bounds, vec![child_layout])
        }

        Node::Size(node_constraints, child) => {
            let new_ct = node_constraints.constrain(constraints);
            let child_node = layout(child, bounds, new_ct);
            LayoutNode::new(node, bounds, vec![child_node])
        }

        Node::Align(alignment, child) => {
            let child_size = measure(child, Constraints::Max(bounds.width(), bounds.height()));
            let offset = alignment.offset(bounds.size(), child_size);
            let child_node = layout(child, Rect::new((bounds.min + offset), Point::new(child_size.width, child_size.height)), Constraints::Fixed(child_size.width, child_size.height));
            LayoutNode::new(node, bounds, vec![child_node])
        }

        Node::Stack(children) => {
            let mut y = bounds.y();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_h = bounds.height().saturating_sub(y - bounds.y());
                let child_ct = Constraints::Max(bounds.width(), remaining_h);
                let size = measure(child, child_ct);
                let child_rect = Rect::new((bounds.x(), y), (bounds.max.x, y.saturating_add(size.height)));
                laid_out.push(layout(child, child_rect, child_ct));
                y = y.saturating_add(size.height);
            }

            LayoutNode::new(node, bounds, laid_out)
        }

        Node::Row(children) => {
            let mut x = bounds.x();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_w = bounds.width().saturating_sub(x - bounds.x());
                let child_ct = Constraints::Max(remaining_w, bounds.height());
                let size = measure(child, child_ct);
                let child_rect = Rect::new((x, bounds.y()), (x.saturating_add(size.width), bounds.max.y));
                laid_out.push(layout(child, child_rect, child_ct));
                x = x.saturating_add(size.width);
            }

            LayoutNode::new(node, bounds, laid_out)
        }

        Node::Layer(children) => {
            let laid_out = children
                .iter()
                .map(|child| layout(child, bounds, constraints))
                .collect();
            LayoutNode::new(node, bounds, laid_out)
        }
    }
}

/// Measure a node's natural size given constraints.
///
/// This is the first phase of rendering (before layout and render).
/// It recursively calculates how much space a node wants to occupy,
/// respecting the provided constraints.
///
/// # Arguments
///
/// * `node` - The node to measure
/// * `constraints` - Size constraints to respect (max, min, fixed, etc.)
///
/// # Returns
///
/// The desired [`Size`] (width and height) of the node.
///
/// # Example
///
/// ```rust
/// use kasten::{measure, Node, Content, Constraints, Constraint};
///
/// let node = Node::Base(Content::Text("Hello".into()));
/// let size = measure(&node, Constraints::Max(100, 100));
///
/// // Text "Hello" is 5 columns wide, 1 row tall
/// assert_eq!(size.width, 5);
/// assert_eq!(size.height, 1);
/// ```
///
/// # Measurement Rules
///
/// Different node types measure differently:
///
/// - **Empty**: Returns zero size
/// - **Text**: Width is Unicode width of string, height is 1
/// - **Fill**: Expands to fill max constraints
/// - **Stack**: Sum of children heights, max of children widths
/// - **Row**: Sum of children widths, max of children heights
/// - **Layer**: Max of all children's widths and heights
/// - **Pad**: Child size plus padding edges
/// - **Style/Align**: Delegates to child
/// - **Size**: Applies node's constraints to child's measurement
///
/// Measurements are clamped to satisfy constraints using [`Constraints::clamp`].
pub fn measure(node: &Node, constraints: Constraints) -> Size {
    match node {
        Node::Base(Content::Empty) => {
            Size::ZERO
        }

        Node::Base(Content::Text(string)) => {
            constraints.clamp(string.display_width(), 1)
        }

        // Node::Base(Primitive::TextWrap(tw)) => {
        //     let lines = wrap_text(&tw.content, constraints.max_w);
        //     let h = lines.len() as u16;
        //     let w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
        //     let (w, h) = constraints.clamp(w, h);
        //     Size::new(w, h)
        // }

        Node::Base(Content::Fill(_)) => constraints.max(),

        Node::Style(_, child) | Node::Align(_, child) => measure(child, constraints),

        Node::Pad(edges, child) => {
            let inner = measure(child, constraints.shrink(edges));
            Size::new(
                inner.width + edges.horizontal(),
                inner.height + edges.vertical(),
            )
        }

        Node::Size(inner_constraints, child) => {
            measure(child, inner_constraints.constrain(constraints))
        }

        Node::Stack(children) => {
            let mut total_h = 0;
            let mut max_w = 0;

            for child in children {
                let size = measure(child, Constraints {
                    height: Constraint::Max(constraints.height.max_or(0).saturating_sub(total_h)),
                    ..constraints
                });
                total_h = total_h.saturating_add(size.height as usize);
                max_w = max_w.max(size.width as usize);
            }

            constraints.clamp(max_w, total_h)
        }

        Node::Row(children) => {
            let mut total_w = 0;
            let mut max_h = 0;

            for child in children {
                let size = measure(child, Constraints {
                    width: Constraint::Max(constraints.width.max_or(0).saturating_sub(total_w)),
                    ..constraints
                });
                total_w = total_w.saturating_add(size.width as usize);
                max_h = max_h.max(size.height as usize);
            }

            constraints.clamp(total_w, max_h)
        }

        Node::Layer(children) => {
            let mut max_w = 0;
            let mut max_h = 0;

            for child in children {
                let size = measure(child, constraints);
                max_w = max_w.max(size.width as usize);
                max_h = max_h.max(size.height as usize);
            }

            constraints.clamp(max_w, max_h)
        }
    }
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
pub fn render(layout: &LayoutNode<'_>, buffer: &mut Buffer, ctx: &Context) {
    let bounds = layout.bounds;
    let region = Region::from(bounds);

    match layout.node {
        Node::Base(Content::Empty) => {}

        Node::Base(Content::Text(s)) => {
            buffer.text(region.min..Position::new(region.min.row, region.max.col), s, &ctx.style);
        }

        // Node::Base(Primitive::TextWrap(tw)) => {
        //     let lines = wrap_text(&tw.content, rect.width());
        //
        //     for (i, line) in lines.iter().enumerate() {
        //         let y = rect.y + i as u16;
        //         if y >= rect.y + rect.h {
        //             break;
        //         }
        //
        //         let line_w = display_width(line);
        //         let x = rect.x + match tw.align {
        //             AlignX::Start => 0,
        //             AlignX::Center => (rect.w.saturating_sub(line_w)) / 2,
        //             AlignX::End => rect.w.saturating_sub(line_w),
        //         };
        //
        //         canvas.text(x, y, line, ctx.style);
        //     }
        // }

        Node::Base(Content::Fill(ch)) =>  {
            for pos in Region::from(bounds) {
                unsafe { buffer.get_unchecked_mut(pos) }.set_char(*ch);
            }
        }

        Node::Style(style, _) => {
            let new_ctx = ctx.add(style);

            if style.bg.is_some() {
                for pos in Region::from(bounds) {
                    unsafe { buffer.get_unchecked_mut(pos) }.style.bg = style.bg;
                }
            }

            for child in &layout.children {
                render(child, buffer, &new_ctx);
            }
        }

        Node::Pad(_, _) | Node::Size(_, _) | Node::Align(_, _) => {
            for child in &layout.children {
                render(child, buffer, ctx);
            }
        }

        Node::Stack(_) | Node::Row(_) | Node::Layer(_) => {
            for child in &layout.children {
                render(child, buffer, ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: usize, y: usize, w: usize, h: usize) -> Rect {
        Rect::new((x, y), (x + w, y + h))
    }

    fn text(s: &str) -> Node {
        Node::Base(Content::Text(s.into()))
    }

    macro_rules! row {
        ($buffer:expr, $row:expr) => {
            &$buffer[Row::from($row)].iter().collect::<String>()
        };
    }

    macro_rules! assert_row {
        ($buffer:expr, $row:expr, $expected:expr) => {
            assert_eq!(row!($buffer, $row), $expected);
        };
    }


    macro_rules! assert_row_empty {
        ($buffer:expr, $row:expr) => {
            assert_eq!(row!($buffer, $row).trim(), "");
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
        let node = Node::Stack(vec![
            text("A"),
            text("B"),
            text("C"),
        ]);

        let bounds = rect(0, 0, 10, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 10));

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
            Box::new(Node::Stack(vec![text("A"), text("B")])),
        );

        let bounds = rect(0, 0, 10, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 10));

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
            text("Line 1"),
            text("Line 2"),
            text("Line 3"),
            text("Line 4"),
            text("Line 5"),
        ]);

        let bounds = rect(0, 0, 10, 3); // Only 3 rows available
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 3));

        assert_eq!(layout_tree.children.len(), 5);
        // All children should be laid out, even if they overflow
        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
        }
    }

    #[test]
    fn test_layout_stack_empty() {
        let node = Node::Stack(vec![]);
        let bounds = rect(0, 0, 10, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 10));

        assert_eq!(layout_tree.children.len(), 0);
    }

    #[test]
    fn test_layout_stack_inverted_bounds_prevented() {
        // Regression test: Stack should not create inverted rectangles
        let node = Node::Stack(vec![
            text("A".repeat(100).as_str()),
            text("B".repeat(100).as_str()),
        ]);

        let bounds = rect(0, 0, 10, 1); // Very limited space
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 1));

        // Verify no child has inverted bounds
        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
            assert!(child.bounds.width() <= 10);
        }
    }

    #[test]
    fn test_layout_row_basic() {
        let node = Node::Row(vec![text("A"), text("B"), text("C")]);

        let bounds = rect(0, 0, 10, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 10));

        assert_eq!(layout_tree.children.len(), 3);
        // Row should position children horizontally
        assert_eq!(layout_tree.children[0].bounds.x(), 0);
        assert_eq!(layout_tree.children[1].bounds.x(), 1);
        assert_eq!(layout_tree.children[2].bounds.x(), 2);
    }

    #[test]
    fn test_layout_row_overflow() {
        let node = Node::Row(vec![
            text("Word1"),
            text("Word2"),
            text("Word3"),
        ]);

        let bounds = rect(0, 0, 10, 5); // Limited width
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 5));

        // All children should have valid bounds
        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
        }
    }

    #[test]
    fn test_layout_row_inverted_bounds_prevented() {
        // Regression test: Row should not create inverted rectangles
        let node = Node::Row(vec![
            text("LongWord1"),
            text("LongWord2"),
            text("LongWord3"),
        ]);

        let bounds = rect(0, 0, 5, 10); // Very limited width
        let layout_tree = layout(&node, bounds, Constraints::Max(5, 10));

        for child in &layout_tree.children {
            assert_rect!(&child.bounds);
        }
    }

    #[test]
    fn test_layout_layer_overlapping() {
        let node = Node::Layer(vec![
            text("Background"),
            text("Foreground"),
        ]);

        let bounds = rect(0, 0, 20, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(20, 10));

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
            Box::new(text("Hi")),
        );

        let bounds = rect(0, 0, 10, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(10, 10));

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
            Box::new(text("Text")),
        );

        let bounds = rect(0, 0, 20, 10);
        let layout_tree = layout(&node, bounds, Constraints::Max(20, 10));

        let child = &layout_tree.children[0];
        // "Text" is 4 chars wide
        assert_eq!(child.bounds.x(), 16); // 20 - 4 (aligned to end)
        assert_eq!(child.bounds.y(), 0); // Start alignment
    }

    #[test]
    fn test_layout_pad_shrinks_bounds() {
        let node = Node::Pad(Edges::new(1, 2, 3, 4), Box::new(text("Test")));

        let bounds = rect(0, 0, 20, 20);
        let layout_tree = layout(&node, bounds, Constraints::Max(20, 20));

        let child = &layout_tree.children[0];
        assert_eq!(child.bounds.x(), 4); // left padding
        assert_eq!(child.bounds.y(), 1); // top padding
        assert_eq!(child.bounds.width(), 14); // 20 - 4 - 2
        assert_eq!(child.bounds.height(), 16); // 20 - 1 - 3
    }

    #[test]
    fn test_layout_size_constraints() {
        let node = Node::Size(
            Constraints::Fixed(15, 8),
            Box::new(text("Constrained")),
        );

        let bounds = rect(0, 0, 100, 100);
        let layout_tree = layout(&node, bounds, Constraints::Max(100, 100));

        // The Size node passes constraints to its child, but the child's bounds
        // are still determined by the parent's provided space
        // The measured size of "Constrained" is 11 chars × 1 row
        let child = &layout_tree.children[0];
        // With Fixed(15, 8) constraints and large parent bounds,
        // the child will use the parent's full bounds
        assert_eq!(child.bounds.width(), 100); // Uses parent's width
    }

    // === Measure Tests ===

    #[test]
    fn test_measure_text() {
        let node = text("Hello");
        let size = measure(&node, Constraints::Max(100, 100));

        assert_eq!(size.width, 5); // "Hello" is 5 chars
        assert_eq!(size.height, 1); // Single line
    }

    #[test]
    fn test_measure_fill() {
        let node = Node::Base(Content::Fill('X'));
        let size = measure(&node, Constraints::Max(10, 5));

        assert_eq!(size.width, 10); // Fills to max width
        assert_eq!(size.height, 5); // Fills to max height
    }

    #[test]
    fn test_measure_empty() {
        let node = Node::Base(Content::Empty);
        let size = measure(&node, Constraints::Max(100, 100));

        assert_eq!(size.width, 0);
        assert_eq!(size.height, 0);
    }

    #[test]
    fn test_measure_stack() {
        let node = Node::Stack(vec![
            text("Short"),
            text("LongerText"),
            text("X"),
        ]);

        let size = measure(&node, Constraints::Max(100, 100));

        // Stack width is max of children
        assert_eq!(size.width, 10); // "LongerText" is longest
        // Stack height is sum of children
        assert_eq!(size.height, 3); // 3 children, each 1 tall
    }

    #[test]
    fn test_measure_row() {
        let node = Node::Row(vec![text("A"), text("BC"), text("DEF")]);

        let size = measure(&node, Constraints::Max(100, 100));

        // Row width is sum of children
        assert_eq!(size.width, 6); // 1 + 2 + 3
        // Row height is max of children
        assert_eq!(size.height, 1); // All text is 1 tall
    }

    #[test]
    fn test_render_stack() {
        let node = Node::Stack(vec![text("First"), text("Second"), text("Third")]);

        let bounds = rect(0, 0, 4, 3);
        let layout_tree = layout(&node, bounds, Constraints::Max(bounds.width(), bounds.height()));

        let mut buffer = Buffer::new(bounds);
        render(&layout_tree, &mut buffer, &Context::default());

        assert_row!(&buffer, 0, "Firs");
        assert_row!(&buffer, 1, "Seco");
        assert_row!(&buffer, 2, "Thir");
    }

    #[test]
    fn test_render_text() {
        let node = Node::Base(Content::Text("Out of bounds\nMore".into()));

        let bounds = rect(0, 0, 4, 2);
        let layout_tree = layout(&node, bounds, Constraints::Max(bounds.width(), bounds.height()));

        let mut buffer = Buffer::new(bounds);
        render(&layout_tree, &mut buffer, &Context::default());

        assert_row!(&buffer, 0, "Out ");
        assert_row_empty!(&buffer, 1);
    }
}
