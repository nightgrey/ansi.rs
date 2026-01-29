use crate::text::DisplayWidth;
use crate::{
    Align, Alignment, Buffer, BufferIndex, Constraint, Constraints, Edges, LayoutNode, Point,
    Position, Rect, Region, Row, Size,
};
use ansi::io::Write;
use ansi::{Color, Escape, Style};
use derive_more::Deref;
use std::ops::BitOr;
use std::process::Child;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

impl Node {
    /// Lays out a node, assigning bounds to each node.
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
    /// use kasten::{constraints, Node, Content, Rect, Constraints};
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
    pub fn layout(&self, bounds: Rect, constraints: Constraints) -> LayoutNode {
        match self {
            Node::Base(_) => LayoutNode::leaf(self, bounds),

            Node::Style(_, child) => {
                let child_layout = Self::layout(child, bounds, constraints);
                LayoutNode::new(self, bounds, vec![child_layout])
            }

            Node::Pad(edges, child) => {
                let inner_rect = bounds.shrink(edges);
                let inner_ct = constraints.shrink(edges);
                let child_layout = Self::layout(child, inner_rect, inner_ct);
                LayoutNode::new(self, bounds, vec![child_layout])
            }

            Node::Size(node_constraints, child) => {
                let new_ct = node_constraints.constrain(constraints);
                let child_node = Self::layout(child, bounds, new_ct);
                LayoutNode::new(self, bounds, vec![child_node])
            }

            Node::Align(alignment, child) => {
                let child_size =
                    Self::measure(child, Constraints::Max(bounds.width(), bounds.height()));
                let offset = alignment.offset(bounds.size(), child_size);
                let child_node = Self::layout(
                    child,
                    Rect::new(
                        (bounds.min + offset),
                        Point::new(child_size.width, child_size.height),
                    ),
                    Constraints::Fixed(child_size.width, child_size.height),
                );
                LayoutNode::new(self, bounds, vec![child_node])
            }

            Node::Stack(children) => {
                let mut y = bounds.y();
                let mut laid_out = Vec::with_capacity(children.len());

                for child in children {
                    let remaining_h = bounds.height().saturating_sub(y - bounds.y());
                    let child_ct = Constraints::Max(bounds.width(), remaining_h);
                    let size = Self::measure(child, child_ct);
                    let child_rect = Rect::new(
                        (bounds.x(), y),
                        (bounds.max.x, y.saturating_add(size.height)),
                    );
                    laid_out.push(Self::layout(child, child_rect, child_ct));
                    y = y.saturating_add(size.height);
                }

                LayoutNode::new(self, bounds, laid_out)
            }

            Node::Row(children) => {
                let mut x = bounds.x();
                let mut laid_out = Vec::with_capacity(children.len());

                for child in children {
                    let remaining_w = bounds.width().saturating_sub(x - bounds.x());
                    let child_ct = Constraints::Max(remaining_w, bounds.height());
                    let size = Self::measure(child, child_ct);
                    let child_rect = Rect::new(
                        (x, bounds.y()),
                        (x.saturating_add(size.width), bounds.max.y),
                    );
                    laid_out.push(Self::layout(child, child_rect, child_ct));
                    x = x.saturating_add(size.width);
                }

                LayoutNode::new(self, bounds, laid_out)
            }

            Node::Layer(children) => {
                let laid_out = children
                    .iter()
                    .map(|child| Self::layout(child, bounds, constraints))
                    .collect();
                LayoutNode::new(self, bounds, laid_out)
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
    /// let size = Self::measure(&node, Constraints::Max(100, 100));
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
    pub fn measure(&self, constraints: Constraints) -> Size {
        match self {
            Node::Base(Content::Empty) => Size::ZERO,

            Node::Base(Content::Text(string)) => constraints.clamp(string.display_width(), 1),

            // Node::Base(Primitive::TextWrap(tw)) => {
            //     let lines = wrap_text(&tw.content, constraints.max_w);
            //     let h = lines.len() as u16;
            //     let w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
            //     let (w, h) = constraints.clamp(w, h);
            //     Size::new(w, h)
            // }
            Node::Base(Content::Fill(_)) => constraints.max(),

            Node::Style(_, child) | Node::Align(_, child) => Self::measure(child, constraints),

            Node::Pad(edges, child) => {
                let inner = Self::measure(child, constraints.shrink(edges));
                Size::new(
                    inner.width + edges.horizontal(),
                    inner.height + edges.vertical(),
                )
            }

            Node::Size(inner_constraints, child) => {
                Self::measure(child, inner_constraints.constrain(constraints))
            }

            Node::Stack(children) => {
                let mut total_h = 0;
                let mut max_w = 0;

                for child in children {
                    let size = Self::measure(
                        child,
                        Constraints {
                            height: Constraint::Max(
                                constraints.height.max_or(0).saturating_sub(total_h),
                            ),
                            ..constraints
                        },
                    );
                    total_h = total_h.saturating_add(size.height as usize);
                    max_w = max_w.max(size.width as usize);
                }

                constraints.clamp(max_w, total_h)
            }

            Node::Row(children) => {
                let mut total_w = 0;
                let mut max_h = 0;

                for child in children {
                    let size = Self::measure(
                        child,
                        Constraints {
                            width: Constraint::Max(
                                constraints.width.max_or(0).saturating_sub(total_w),
                            ),
                            ..constraints
                        },
                    );
                    total_w = total_w.saturating_add(size.width as usize);
                    max_h = max_h.max(size.height as usize);
                }

                constraints.clamp(total_w, max_h)
            }

            Node::Layer(children) => {
                let mut max_w = 0;
                let mut max_h = 0;

                for child in children {
                    let size = Self::measure(child, constraints);
                    max_w = max_w.max(size.width);
                    max_h = max_h.max(size.height);
                }

                constraints.clamp(max_w, max_h)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Node {
        Node::Base(Content::Text(s.into()))
    }

    // === Measure Tests ===

    #[test]
    fn test_measure_text() {
        let node = text("Hello");
        let size = node.measure(Constraints::Max(100, 100));

        assert_eq!(size.width, 5); // "Hello" is 5 chars
        assert_eq!(size.height, 1); // Single line
    }

    #[test]
    fn test_measure_fill() {
        let node = Node::Base(Content::Fill('X'));
        let size = node.measure(Constraints::Max(10, 5));

        assert_eq!(size.width, 10); // Fills to max width
        assert_eq!(size.height, 5); // Fills to max height
    }

    #[test]
    fn test_measure_empty() {
        let node = Node::Base(Content::Empty);
        let size = node.measure(Constraints::Max(100, 100));

        assert_eq!(size.width, 0);
        assert_eq!(size.height, 0);
    }

    #[test]
    fn test_measure_stack() {
        let node = Node::Stack(vec![text("Short"), text("LongerText"), text("X")]);

        let size = node.measure(Constraints::Max(100, 100));

        // Stack width is max of children
        assert_eq!(size.width, 10); // "LongerText" is longest
        // Stack height is sum of children
        assert_eq!(size.height, 3); // 3 children, each 1 tall
    }

    #[test]
    fn test_measure_row() {
        let node = Node::Row(vec![text("A"), text("BC"), text("DEF")]);

        let size = node.measure(Constraints::Max(100, 100));

        // Row width is sum of children
        assert_eq!(size.width, 6); // 1 + 2 + 3
        // Row height is max of children
        assert_eq!(size.height, 1); // All text is 1 tall
    }
}
