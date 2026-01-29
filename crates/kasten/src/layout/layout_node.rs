use super::{LayoutContext, render};
use crate::{
    Buffer, BufferIndex, Constraint, Constraints, Content, Edges, Node, Point, Position, Rect,
    Region, Row, Size,
};
use derive_more::Deref;

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
#[derive(Debug, Clone, Deref)]
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
        Self {
            node,
            bounds,
            children,
        }
    }

    /// Create a leaf layout node with no children.
    pub fn leaf(node: &'a Node, bounds: Rect) -> Self {
        Self::new(node, bounds, vec![])
    }
    
    /// Render a laid out node tree into a buffer.
    ///
    /// This is the third and final phase of rendering (after measure and layout).
    /// It recursively draws each node in the tree to the provided buffer,
    /// applying styles from the context and respecting the computed bounds.
    pub fn render(&self, buffer: &mut Buffer, context: &LayoutContext) {
        render(self, buffer, context);
    }
}
