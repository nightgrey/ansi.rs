use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut};
use geometry::{Rect};
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
        self.document.compute_layers();
    }

    fn layout(&mut self) {
        self.document.compute_layouts();
    }

    fn paint(&mut self) {
        self.document.layers.values_mut().for_each(|layer| {
            layer.clear();
            layer.is_dirty = false;
        });

        self.paint_element(self.document.root_id());
    }

    fn paint_element(&mut self, id: ElementId) {
        let mut computation = self.document.layouts[id].final_computation;
        let bounds = Rect::bounds(computation.content_box_x() as usize, computation.content_box_y() as usize, computation.content_box_width() as usize, computation.content_box_height() as usize);
        let style = self.document.elements[id].style;
        let kind = self.document[id].kind.clone();

        let mut painter = Painter::new(&mut self.document.layers[id]);
         painter.push(bounds);

        match &kind {
            ElementKind::Span(content) => {
                if !style.is_empty() {
                    painter.fill(bounds, style);
                }
                painter.text(bounds.min.y as i32, bounds.min.x as i32, &content, style, &mut self.document.arena);
            }   
            ElementKind::Div => {
                if !style.is_empty() {
                    painter.fill(bounds, style);
                }
            }
        }

        for child in self.document.elements.children(id).collect::<Vec<_>>() {
            self.paint_element(child);
        }
    }

    fn composite(&mut self) {
        self.renderer.front.clear();
        Renderer::composite(&mut self.renderer.front, &self.document, self.document.root_id());
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
        &self.document.layers.get_direct(index).unwrap()
    }
}

impl IndexMut<LayerId> for Orchestrator {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        self.document.layers.get_direct_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Color, Style};
    use crate::Cell;
    use tree::layout::prelude::*;
    use tree::At;
    use super::*;

    #[test]
    fn layout_distributes_remainder_cells() {
        let mut orchestrator = Orchestrator::new(5, 4);
        let mut root = orchestrator.root_mut();
        root.layout.flex_direction = FlexDirection::Row;
        let a = orchestrator.insert_with_layout(Element::Span("abc".into()), Layout { flex_grow: 1.0, ..Layout::default() });
        let b = orchestrator.insert_with_layout(Element::Span("b".into()), Layout { flex_grow: 1.0, ..Layout::default() });
        let c = orchestrator.insert_with_layout(Element::Span("c".into()), Layout { flex_grow: 1.0, ..Layout::default() });

        orchestrator.layout();

        // Taffy distributes evenly with flex_grow: 3 children in 5 cols
        // Each gets floor(5/3)=1 with rounding; taffy may round differently
        let a_rect = orchestrator.get_computation(a).unwrap();
        let b_rect = orchestrator.get_computation(b).unwrap();
        let c_rect = orchestrator.get_computation(c).unwrap();

        // All children should span the full height
        assert_eq!(a_rect.size.height, 4.0);
        assert_eq!(b_rect.size.height, 4.0);
        assert_eq!(c_rect.size.height, 4.0);
        // Total width should equal viewport width
        assert_eq!(a_rect.size.width + b_rect.size.width + c_rect.size.width, 5.0);
        // Children should be contiguous
        assert_eq!(a_rect.location.x, 0.0);
        assert_eq!(b_rect.location.x, a_rect.size.width);
        assert_eq!(c_rect.location.x, a_rect.size.width + b_rect.size.width);
    }

    #[test]
    fn frame_paints_text_into_front_buffer() {
        let mut orchestrator = Orchestrator::new(5, 1);
        let mut root = orchestrator.root_mut();
        root.kind = ElementKind::Span("Hi".into());

        let mut sink = Vec::new();
        orchestrator.render().unwrap();

        orchestrator.flush(&mut sink).unwrap();

        assert!(!orchestrator.renderer.front[(0, 0)].is_empty());
        assert!(!orchestrator.renderer.front[(0, 1)].is_empty());
    }

    #[test]
    fn composite_respects_child_layer_order() {
        let mut orchestrator = Orchestrator::new(3, 1);

        let mut root = orchestrator.root_mut();
        root.kind = ElementKind::Div;
        let layer = root.layer_mut();
        layer[(0, 0)] = Cell::from_char('a', Style::new().foreground(Color::Index(1)));

        let child_id = orchestrator.insert(Element::Div());
        orchestrator.layers.insert(child_id, Layer::new(3, 1));
        let child_layer = orchestrator.layers.get_mut(child_id).unwrap();
        child_layer.z_index = 1;
        child_layer[(0, 0)] = Cell::from_char('b', Style::new().foreground(Color::Index(2)));

        orchestrator.composite();

        assert_eq!(orchestrator.renderer.front[(0, 0)].as_str(&orchestrator.document.arena), "b");
        assert_eq!(orchestrator.renderer.front[(0, 0)].style, Style::new().foreground(Color::Index(2)));
    }
}
