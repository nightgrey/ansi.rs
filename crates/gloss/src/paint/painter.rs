use crate::{Document, NodeId, NodeKind, Style};
use maybe::Maybe;
use geometry::{Bounded, Point, Rect, Size};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use crate::drawing_context::DrawingContext;

#[derive(Default, Deref, DerefMut, AsRef, AsMut, Debug)]
pub struct Painter<B: DrawingContext>(pub B);

impl<B: DrawingContext> Painter<B> {
    pub fn paint(&mut self, document: &Document<'_>) {
        self.resize(document.border_bounds(document.root));
        self.paint_node(document, document.root, Style::DEFAULT);
        self.finish();
    }

    fn paint_node(
        &mut self,
        document: &Document<'_>,
        id: NodeId,
        parent_style: Style,
    ) {
        let node = document.node(id);
        let border_bounds = document.border_bounds(id);
        let content_bounds = document.content_bounds(id);
        let style = node.style.inherit(parent_style);

        // Snapshot the current state
        self.save();

        if style.border.is_some() { self.border(border_bounds); }

        // Apply this node's style and clip
        self
            .translate(border_bounds.min)
            .clip(border_bounds.size().into())
            .style(style)
            .border_style(node.border);

        // Normalize content bounds to be relative to the node's border box.
        let normalized_bounds = content_bounds - border_bounds.min;

        // Paint
        if style.background.is_some() { self.rect(normalized_bounds); }

        match &node.kind {
            NodeKind::Span(text) => {
                self.text(normalized_bounds.min, text);
            }
            NodeKind::Div => {
            }
        }

        // Recurse — clip to *content* area so children don't paint over padding/borders.
        // Use normalized bounds: origin is already at border_bounds.min, and child
        // layouts from taffy are border-box-relative, so the clip must be too.
        self.save();
        self.clip(normalized_bounds);

        for child in document.children(id) {
            self.paint_node(document, child, style);
        }

        self.restore();

        self.restore();
    }
}
