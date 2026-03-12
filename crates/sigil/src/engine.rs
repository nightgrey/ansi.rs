use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut};
use crate::{Direction, Element, ElementId, ElementKind, Layer, LayerId, Renderer, Scene};
use crate::painter::Painter;
use geometry::Rect;
use grid::{Position, Spatial};

#[derive(Debug, Deref, DerefMut)]
pub struct Engine {
    #[deref]
    #[deref_mut]
    pub scene: Scene,
    pub renderer: Renderer,
    pub width: usize,
    pub height: usize,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            scene: Scene::new(width, height),
            renderer: Renderer::new(width, height),
            width,
            height,
        }
    }

    fn viewport(&self) -> Rect {
        Rect::bounds(0, 0, self.width, self.height)
    }

   fn layer(&mut self) {
        self.scene.layer(self.scene.root_id(), self.scene.root_layer_id());
    }

   fn layout(&mut self) {
        self.scene.layout(self.scene.root_id(), self.viewport());
    }

    fn paint(&mut self) {
        self.scene.layers.iter_mut().for_each(|(_, layer)| {
            layer.clear();
            layer.is_dirty = false;
        });

        self.paint_element(self.scene.root_id());
    }

    fn paint_element(&mut self, id: ElementId) {
        let element = &self.scene.get(id).unwrap();
        let kind = element.kind.clone();
        let style = element.style;
        let bounds = self.scene.layout[id];
        let layer_id = element.layer_id;

        {
            let layer = &mut self.scene.layers[layer_id];
            let mut painter = Painter::new(layer, &mut self.renderer.arena);
            painter.push(bounds);

            match &kind {
                ElementKind::Text(content) => {
                    if !style.is_empty() {
                        painter.fill(bounds, style);
                    }
                    painter.draw_text(bounds.y() as i32, bounds.x() as i32, content, style);
                }
                ElementKind::Container { .. } => {
                    if !style.is_empty() {
                        painter.fill(bounds, style);
                    }
                }
            }
        }

        for child in self.scene.elements.children(id).collect::<Vec<_>>() {
            self.paint_element(child);
        }
    }

    fn composite(&mut self) {
        self.renderer.front.clear();
        let layer_id = self.scene.layers.root_id();
        self.renderer.composite(&mut self.scene.layers, layer_id);
    }

    fn render(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        self.renderer.render(out)
    }

    pub fn frame(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        self.layer();
        self.layout();
        self.paint();
        self.composite();
        self.render(out)
    }


    pub fn resize(&mut self, width: usize, height: usize) {
        self.scene.resize(width, height);
        self.renderer.resize(width, height);
    }

}

impl Index<LayerId> for Engine {
    type Output = Layer;

    fn index(&self, index: LayerId) -> &Self::Output {
        &self.scene.layers[index]
    }
}

impl IndexMut<LayerId> for Engine {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        &mut self.scene.layers[index]
    }
}

#[cfg(test)]
mod tests {
    use ansi::{Color, Style};
    use crate::{Cell};
    use tree::{At};
    use super::*;

    #[test]
    fn layout_distributes_remainder_cells() {
        let mut engine = Engine::new(5, 4);
        let id = engine.scene.root_id();
        engine.scene[id].kind = ElementKind::Container { direction: Direction::Horizontal };

        let a = engine.scene.insert_at(Element::text("a".into()), At::Append(id));
        let b = engine.scene.insert_at(Element::text("b".into()), At::Append(id));
        let c = engine.scene.insert_at(Element::text("c".into()), At::Append(id));

        engine.scene.layout(id, engine.viewport());
        assert_eq!(engine.scene.layout[a], Rect::bounds(0, 0, 2, 4));
        assert_eq!(engine.scene.layout[b], Rect::bounds(2, 0, 2, 4));
        assert_eq!(engine.scene.layout[c], Rect::bounds(4, 0, 1, 4));
    }

    #[test]
    fn frame_paints_text_into_front_buffer() {
        let mut engine = Engine::new(5, 1);
        let id = engine.scene.root_id();
        engine.scene[id].kind = ElementKind::Text("Hi".into());

        let mut sink = Vec::new();
        engine.frame(&mut sink).unwrap();

        assert!(!engine.renderer.front[(0, 0)].is_empty());
        assert!(!engine.renderer.front[(0, 1)].is_empty());
    }

    #[test]
    fn composite_respects_child_layer_order() {
        let mut engine = Engine::new(3, 1);

        let root_layer_id = engine.layers.root_id();
        let mut root_layer = engine.layers.root_mut();
        root_layer.position = Position::ZERO;
        root_layer[(0, 0)] = Cell::from_char('a', Style::new().foreground(Color::Index(1)));

        let child_layer_id = engine.layers.insert_at(Layer::new(3, 1), At::Append(root_layer_id));
        let mut child_layer = engine.layers.get_ref_mut(child_layer_id).unwrap();
        child_layer.position = Position::ZERO;
        child_layer.z_index = 1;
        child_layer[(0, 0)] = Cell::from_char('b', Style::new().foreground(Color::Index(2)));

        engine.composite();

        assert_eq!(engine.renderer.front[(0, 0)].as_str(&engine.renderer.arena), "b");
        assert_eq!(engine.renderer.front[(0, 0)].style, Style::new().foreground(Color::Index(2)));
    }
}
