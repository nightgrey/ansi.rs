use derive_more::{Deref};
use crate::{Region,Buffer, BufferIndex, Constraint, Constraints, Edges, Point, Rect, Size, Row, Position, Node, Content};
use crate::layout::layout::LayoutContext;

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
        Self { node, bounds, children }
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
    pub fn render(&self, buffer: &mut Buffer, context: &LayoutContext) {
        let layout_node = self;
        let bounds = layout_node.bounds;
        let region = Region::from(bounds);

        match layout_node.node {
            Node::Base(Content::Empty) => {}

            Node::Base(Content::Text(s)) => {
                buffer.text(region.min..Position::new(region.min.row, region.max.col), s, &context.style);
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
                let new_ctx = context.compose(style);

                if style.bg.is_some() {
                    for pos in Region::from(bounds) {
                        unsafe { buffer.get_unchecked_mut(pos) }.style.bg = style.bg;
                    }
                }

                for child in &layout_node.children {
                    child.render(buffer, &new_ctx);
                }
            }

            Node::Pad(_, _) | Node::Size(_, _) | Node::Align(_, _) => {
                for child in &layout_node.children {
                    child.render(buffer, context);
                }
            }

            Node::Stack(_) | Node::Row(_) | Node::Layer(_) => {
                for child in &layout_node.children {
                    child.render(buffer, context);
                }
            }
        }
    }

}


