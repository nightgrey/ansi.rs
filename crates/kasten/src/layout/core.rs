use geometry::{Point, Position, Rect, Region, Size};
use crate::{Buffer, Constraint, Constraints, Content, LayoutContext, LayoutNode, Node};
use crate::text::DisplayWidth;

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
pub fn layout(node: &Node, bounds: Rect, constraints: Constraints) -> LayoutNode {
    match node {
        Node::Base(_) => LayoutNode::leaf(node, bounds),

        Node::Style(_, child) => {
            let child_layout = Node::layout(child, bounds, constraints);
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
            let child_size =
               measure(child, Constraints::Max(bounds.width(), bounds.height()));
            let offset = alignment.offset(bounds.size(), child_size);
            let child_node = layout(
                child,
                Rect::new(
                    (bounds.min + offset),
                    Point::new(child_size.width, child_size.height),
                ),
                Constraints::Fixed(child_size.width, child_size.height),
            );
            LayoutNode::new(node, bounds, vec![child_node])
        }

        Node::Stack(children) => {
            let mut y = bounds.y();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_h = bounds.height().saturating_sub(y - bounds.y());
                let child_ct = Constraints::Max(bounds.width(), remaining_h);
                let size =measure(child, child_ct);
                let child_rect = Rect::new(
                    (bounds.x(), y),
                    (bounds.max.x, y.saturating_add(size.height)),
                );
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
                let size =measure(child, child_ct);
                let child_rect = Rect::new(
                    (x, bounds.y()),
                    (x.saturating_add(size.width), bounds.max.y),
                );
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
/// let size =measure(&node, Constraints::Max(100, 100));
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

        Node::Style(_, child) | Node::Align(_, child) =>measure(child, constraints),

        Node::Pad(edges, child) => {
            let inner =measure(child, constraints.shrink(edges));
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
                let size =measure(
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
                let size =measure(
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
                let size = measure(child, constraints);
                max_w = max_w.max(size.width);
                max_h = max_h.max(size.height);
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
pub fn render(layout_node: &LayoutNode, buffer: &mut Buffer, context: &LayoutContext) {
    let bounds = layout_node.bounds;
    let region = Region::from(bounds);

    match layout_node.node {
        Node::Base(Content::Empty) => {}

        Node::Base(Content::Text(s)) => {
            buffer.text(
                region.min..Position::new(region.min.row, region.max.col),
                s,
                &context.style,
            );
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
        Node::Base(Content::Fill(ch)) => {
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