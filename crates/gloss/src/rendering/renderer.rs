use crate::{Backend, Node};
use crate::{Document, NodeId, NodeKind, Style};

use derive_more::{AsMut, AsRef, Deref, DerefMut};
use geometry::{Bounded, Point, Rect};

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Renderer<B: Backend>(pub B);

impl<B: Backend> Renderer<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }

    pub fn render(&mut self, doc: &Document<'_>) -> Result<(), B::Error> {
        self.resize(doc.size(doc.root))?;
        
        self.render_node(doc, doc.root, Style::DEFAULT)?;
        self.finish()
    }

    fn render_node(
        &mut self,
        doc: &Document<'_>,
        id: NodeId,
        inherited: Style,
    ) -> Result<(), B::Error> {
        let  node = doc.node(id);
        let bounds = doc.bounds(id);
        let content_bounds = doc.content_bounds(id);
        let style = node.style.inherit(inherited);

        // Snapshot the current state
        self.save()?;

        // Apply this node's style and clip
        self.translate(bounds.min)?;
        self.clip(bounds.size().into())?;
        self.set_style(style);

        // Paint
        if style.has_background() { self.fill(None, None, Some(' ')) }
        if style.has_border() { self.stroke(None, None) }

        match &node.kind {
            NodeKind::Span(text) => {
                self.draw_text(text, Some(content_bounds.min - bounds.min), None);
            }
            NodeKind::Div => {
            }
        }

        // Recurse — clip to *content* area so children don't paint over padding/borders
        self.save()?;
        self.translate(content_bounds.min)?;
        self.clip(content_bounds.size().into())?;

        for child in doc.children(id) {
            self.render_node(doc, child, style)?;
        }

        self.restore()?;

        self.restore()
    }
}
