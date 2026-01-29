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
use crate::layout::core::{layout, measure};

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
    pub fn layout(&self, bounds: Rect, constraints: Constraints) -> LayoutNode {
        layout(self, bounds, constraints)
    }

    /// Measure a node's natural size given constraints.
    ///
    /// This is the first phase of rendering (before layout and render).
    /// It recursively calculates how much space a node wants to occupy,
    /// respecting the provided constraints.
    pub fn measure(&self, constraints: Constraints) -> Size {
       measure(self, constraints)
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
