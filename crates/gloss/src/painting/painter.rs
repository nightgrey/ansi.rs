use crate::{Document, NodeId, NodeKind, Style};
use maybe::Maybe;
use geometry::{Bounded, Point, Rect, Size};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use crate::drawing_context::DrawingContext;

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Painter<B: DrawingContext>(pub B);

impl<B: DrawingContext> Painter<B> {
    pub fn paint(&mut self, document: &Document<'_>) {
        self.resize(document.size(document.root));
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
        let bounds = document.bounds(id);
        let content_bounds = document.content_bounds(id);
        let style = node.style.inherit(parent_style);

        // Snapshot the current state
        self.save();

        // Apply this node's style and clip
        self
            .translate(bounds.min)
            .clip(bounds.size().into())
            .style(style)
            .border_style(node.border);

        // Normalize content bounds to be relative to the node's border box.
        let content_bounds = content_bounds - bounds.min;

        // Paint
        if style.background.is_some() { self.rect(content_bounds); }
        if style.border.is_some() { self.border(content_bounds); }

        match &node.kind {
            NodeKind::Span(text) => {
                self.text(content_bounds.min, text);
            }
            NodeKind::Div => {
            }
        }

        // Recurse — clip to *content* area so children don't paint over padding/borders.
        // Don't translate — child bounds from taffy are relative to the parent's
        // border box, so translating to the content area would double-count padding.
        self.save();
        self.clip(content_bounds);

        for child in document.children(id) {
            self.paint_node(document, child, style);
        }

        self.restore();

        self.restore();
    }
}
