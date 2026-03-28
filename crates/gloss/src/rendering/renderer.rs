use crate::Backend;
use crate::{Document, NodeId, NodeKind, Style};

use derive_more::{AsMut, AsRef, Deref, DerefMut};
#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Renderer<B: Backend>(pub B);

impl<B: Backend> Renderer<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }

    pub fn render(&mut self, doc: &Document<'_>) -> Result<(), B::Error> {
        self.resize(doc.size(doc.root))?;
        self.render_node(doc, doc.root, crate::Style::DEFAULT)?;
        self.finish()
    }

    fn render_node(
        &mut self,
        doc: &Document<'_>,
        id: NodeId,
        inherited: Style,
    ) -> Result<(), B::Error> {
        let mut node = doc.node(id);
        let bounds = doc.bounds(id);
        let content_bounds = doc.content_bounds(id);
        let style = node.style.inherit(inherited);


        if style.has_background_color() { self.fill_style(content_bounds.clone(), style) }
        if style.has_border() { self.stroke(bounds, style) }

        // Leaf text.
        match &node.kind {
            NodeKind::Span(text) => {
                // MVP: left-aligned only in actual paint pass.
                // Center/right can be added once line layout is shared with measurement.
                self.draw_text(content_bounds.min, text, style);
            }
            _ => {}
        }

        for child in doc.children(id) {
            self.render_node(doc, child, style)?;
        }

        self.finish()?;

        Ok(())
    }
}
