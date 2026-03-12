use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut};
use crate::{Element, ElementId, ElementKind, Layer, LayerId, Renderer, Document};
use crate::painter::Painter;

#[derive(Debug, Deref, DerefMut)]
pub struct Orchestrator {
    #[deref]
    #[deref_mut]
    pub document: Document,
    pub renderer: Renderer,
}

impl Orchestrator {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            document: Document::new(width, height),
            renderer: Renderer::new(width, height),
        }
    }

    fn layer(&mut self) {
        self.document.compute_layers(self.document.root_id(), self.document.root_layer_id());
    }

    fn layout(&mut self) {
        self.document.compute_layout();
    }

    fn paint(&mut self) {
        self.document.layers.iter_mut().for_each(|(_, layer)| {
            layer.clear();
            layer.is_dirty = false;
        });

        self.paint_element(self.document.root_id());
    }

    fn paint_element(&mut self, id: ElementId) {
        let element = &self.document.get(id).unwrap();
        let kind = element.kind.clone();
        let style = element.style;
        let bounds = self.document.layout[id];
        let layer_id = element.layer_id;

        {
            let layer = &mut self.document.layers[layer_id];
            let mut painter = Painter::new(layer, &mut self.document.arena);
            painter.push(bounds);

            match &kind {
                ElementKind::Span(content) => {
                    if !style.is_empty() {
                        painter.fill(bounds, style);
                    }
                    painter.draw_text(bounds.y() as i32, bounds.x() as i32, content, style);
                }
                ElementKind::Div => {
                    if !style.is_empty() {
                        painter.fill(bounds, style);
                    }
                }
            }
        }

        for child in self.document.elements.children(id).collect::<Vec<_>>() {
            self.paint_element(child);
        }
    }

    fn composite(&mut self) {
        self.renderer.front.clear();
        let layer_id = self.document.layers.root_id();
        Renderer::composite(&mut self.renderer.front, &self.document.layers, layer_id);
    }

    /// Insert an element as a child of the root element.
    pub fn insert(&mut self, element: Element) -> ElementId {
        let root = self.document.root_id();
        self.document.insert_at(element, tree::At::Child(root))
    }

    /// Insert an element as a child of `parent`.
    pub fn insert_at(&mut self, element: Element, parent: ElementId) -> ElementId {
        self.document.insert_at(element, tree::At::Child(parent))
    }

    pub fn raster(&mut self) -> std::io::Result<()> {
        self.renderer.raster(&self.document.arena);
        Ok(())
    }

    /// Run the full frame pipeline and write output.
    pub fn render(&mut self) -> std::io::Result<()> {
        self.layer();
        self.layout();
        self.paint();
        self.composite();
        self.raster()?;
        Ok(())
    }

    pub fn flush(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        self.renderer.flush(&self.document.arena, out)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.document.resize(width, height);
        self.renderer.resize(width, height);
    }
}

impl Index<LayerId> for Orchestrator {
    type Output = Layer;

    fn index(&self, index: LayerId) -> &Self::Output {
        &self.document.layers[index]
    }
}

impl IndexMut<LayerId> for Orchestrator {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        &mut self.document.layers[index]
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Color, Style};
    use crate::Cell;
    use grid::Position;
    use tree::At;
    use super::*;

    #[test]
    fn layout_distributes_remainder_cells() {
        let mut orchestrator = Orchestrator::new(5, 4);
        let id = orchestrator.document.root_id();
        orchestrator.document[id].kind = ElementKind::Div;
        orchestrator.document[id].layout.flex_direction = taffy::FlexDirection::Row;

        let a = orchestrator.document.insert_at(Element::Span("a".into()), At::Child(id));
        let b = orchestrator.document.insert_at(Element::Span("b".into()), At::Child(id));
        let c = orchestrator.document.insert_at(Element::Span("c".into()), At::Child(id));

        orchestrator.document.compute_layout();
        // Taffy distributes evenly with flex_grow: 3 children in 5 cols
        // Each gets floor(5/3)=1 with rounding; taffy may round differently
        let a_rect = orchestrator.document.layout[a];
        let b_rect = orchestrator.document.layout[b];
        let c_rect = orchestrator.document.layout[c];
        // All children should span the full height
        assert_eq!(a_rect.height(), 4);
        assert_eq!(b_rect.height(), 4);
        assert_eq!(c_rect.height(), 4);
        // Total width should equal viewport width
        assert_eq!(a_rect.width() + b_rect.width() + c_rect.width(), 5);
        // Children should be contiguous
        assert_eq!(a_rect.x(), 0);
        assert_eq!(b_rect.x(), a_rect.width());
        assert_eq!(c_rect.x(), a_rect.width() + b_rect.width());
    }

    #[test]
    fn frame_paints_text_into_front_buffer() {
        let mut orchestrator = Orchestrator::new(5, 1);
        let id = orchestrator.document.root_id();
        orchestrator.document[id].kind = ElementKind::Span("Hi".into());

        let mut sink = Vec::new();
        orchestrator.render().unwrap();
        orchestrator.flush(&mut sink).unwrap();

        assert!(!orchestrator.renderer.front[(0, 0)].is_empty());
        assert!(!orchestrator.renderer.front[(0, 1)].is_empty());
    }

    #[test]
    fn composite_respects_child_layer_order() {
        let mut orchestrator = Orchestrator::new(3, 1);

        let root_layer_id = orchestrator.layers.root_id();
        let mut root_layer = orchestrator.layers.root_mut();
        root_layer.position = Position::ZERO;
        root_layer[(0, 0)] = Cell::from_char('a', Style::new().foreground(Color::Index(1)));

        let child_layer_id = orchestrator.layers.insert_at(Layer::new(3, 1), At::Child(root_layer_id));
        let mut child_layer = orchestrator.layers.get_ref_mut(child_layer_id).unwrap();
        child_layer.position = Position::ZERO;
        child_layer.z_index = 1;
        child_layer[(0, 0)] = Cell::from_char('b', Style::new().foreground(Color::Index(2)));

        orchestrator.composite();

        assert_eq!(orchestrator.renderer.front[(0, 0)].as_str(&orchestrator.document.arena), "b");
        assert_eq!(orchestrator.renderer.front[(0, 0)].style, Style::new().foreground(Color::Index(2)));
    }
}
