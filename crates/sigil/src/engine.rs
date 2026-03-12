use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut, Index, IndexMut};
use crate::{Buffer, Direction, Element, ElementId, ElementKind, GraphemeArena, Layer, LayerId, Rasterizer, RootTree, Secondary, TreeNodeRef, TreeNodeRefMut};
use crate::painter::Painter;
use geometry::Rect;
use grid::{Position, Spatial};

pub type ElementRef<'a> = TreeNodeRef<'a, ElementId, Element>;
pub type ElementRefMut<'a> = TreeNodeRefMut<'a, ElementId, Element>;
pub type LayerRef<'a> = TreeNodeRef<'a, LayerId, Layer>;
pub type LayerRefMut<'a> = TreeNodeRefMut<'a, LayerId, Layer>;

pub type Elements = RootTree<ElementId, Element>;
pub type Layers = RootTree<LayerId, Layer>;
pub type Layout = Secondary<ElementId, Rect>;

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
pub struct Engine {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
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

    pub fn resize(&mut self, width: usize, height: usize) {
        self.scene.resize(width, height);
        self.renderer.resize(width, height);
    }

   fn layer(&mut self) {
        self.scene.layer(self.root_element(), self.root_layer());
    }

   fn layout(&mut self) {
        self.scene.layout(self.root_element(), self.viewport());
    }

    fn paint(&mut self) {
        self.scene.layers.iter_mut().for_each(|(_, layer)| {
            layer.clear();
            layer.is_dirty = false;
        });

        self.paint_element(self.root_element());
    }

    fn paint_element(&mut self, id: ElementId) {
        let element = &self.scene.elements[id];
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

        let layer_id = self.scene.root_layer();
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

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
pub struct Scene {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    pub elements: Elements,
    pub layers: Layers,
    pub layout: Layout,
}

impl Scene {
    pub fn new(width: usize, height: usize) -> Self {
        let layers = RootTree::new(Layer::new(width, height));
        let elements = RootTree::new(Element::container(Direction::Vertical).on(layers.root()));
        let mut layout = Secondary::new();
        layout.insert(elements.root(), Rect::bounds(0, 0, width, height));
        Self { elements, layers, layout }
    }

    pub fn root_element(&self) -> ElementId {
        self.elements.root()
    }

    pub fn root_layer(&self) -> LayerId {
        self.layers.root()
    }

    fn layer(
        &mut self,
        id: ElementId,
        layer_id: LayerId,
    ) {
        self.elements[id].layer_id = layer_id;

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            let child = &self.elements[child_id];
            let next_layer_id = if child.is_promoting() {
                let size = &self.layers[layer_id].size();
                let next_layer_id = self.layers.insert_detached(Layer::new(size.width, size.height));
                self.layers.append_child(layer_id, next_layer_id);
                next_layer_id
            } else {
                layer_id
            };

            self.layer(child_id, next_layer_id);
        }
    }

    fn layout(
        &mut self,
        id: ElementId,
        bounds: Rect,
    ) {
        self.layout.insert(id, bounds);

        match &self.elements[id].kind {
            ElementKind::Container { direction } => {
                let children: Vec<_> = self.elements.children(id).collect();
                let child_count = children.len();
                if child_count == 0 {
                    return;
                }

                match direction {
                    Direction::Vertical => {
                        let base = bounds.height() / child_count;
                        let remainder = bounds.height() % child_count;
                        let mut y = bounds.y();

                        for (index, child) in children.into_iter().enumerate() {
                            let child_height = base + usize::from(index < remainder);
                            let child_rect = Rect::bounds(bounds.x(), y, bounds.width(), child_height);
                            y += child_height;
                            self.layout(child, child_rect);
                        }
                    }
                    Direction::Horizontal => {
                        let base = bounds.width() / child_count;
                        let remainder = bounds.width() % child_count;
                        let mut x = bounds.x();

                        for (index, child) in children.into_iter().enumerate() {
                            let child_width = base + usize::from(index < remainder);
                            let child_rect = Rect::bounds(x, bounds.y(), child_width, bounds.height());
                            x += child_width;
                            self.layout(child, child_rect);
                        }
                    }
                }
            }
            ElementKind::Text(_) => {}
        }
    }


    pub fn resize(&mut self, width: usize, height: usize) {
        self.layers.get_root_node_mut().resize(width, height);
        self.layout.insert(self.elements.root(), Rect::bounds(0, 0, width, height));
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
        self.layout.clear();
    }
}

#[derive(Debug)]
pub struct Renderer {
    pub front: Buffer,
    pub back: Buffer,
    pub arena: GraphemeArena,
    pub rasterizer: Rasterizer,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            arena: GraphemeArena::new(),
            rasterizer: Rasterizer::new(width, height),
        }
    }

    fn composite(&mut self, layers: &mut Layers, id: LayerId) {
        let layer = &layers[id];
        for row in 0..layer.height {
            let front_row = layer.position.row + row;
            if front_row >= self.front.height {
                continue;
            }

            for col in 0..layer.width {
                let front_col = layer.position.col + col;
                if front_col >= self.front.width {
                    continue;
                }

                let cell = layer[(row, col)];
                if !cell.is_empty() {
                    self.front[(front_row, front_col)] = cell;
                }
            }
        }

        let mut children: Vec<_> = layers.children(id).collect();
        children.sort_by_key(|child| layers[*child].z_index);

        for child in children {
            self.composite(layers, child);
        }
    }

    fn render(&mut self, output: &mut impl std::io::Write) -> std::io::Result<()> {
        self.rasterizer.render(&self.front, &self.arena);
        self.rasterizer.flush(output)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.resize_inner(width, height);
        self.back.resize_inner(width, height);
        self.rasterizer.resize(width, height);
    }

    pub fn clear(&mut self) {
        self.front.clear();
        self.back.clear();
        self.rasterizer.clear();
        self.arena.clear();
    }
}


#[cfg(test)]
mod tests {
    use ansi::{Color, Style};
    use crate::Cell;
    use super::*;

    #[test]
    fn layout_distributes_remainder_cells() {
        let mut engine = Engine::new(5, 4);
        let id = engine.root_element();
        engine[id].kind = ElementKind::Container { direction: Direction::Horizontal };

        let a = engine.append_root(Element::text("a".into()));
        let b = engine.append_root(Element::text("b".into()));
        let c = engine.append_root(Element::text("c".into()));

        engine.scene.layout(id, engine.viewport());
        assert_eq!(engine.scene.layout[a], Rect::bounds(0, 0, 2, 4));
        assert_eq!(engine.scene.layout[b], Rect::bounds(2, 0, 2, 4));
        assert_eq!(engine.scene.layout[c], Rect::bounds(4, 0, 1, 4));
    }

    #[test]
    fn frame_paints_text_into_front_buffer() {
        let mut engine = Engine::new(5, 1);
        let id = engine.root_element();
        engine[id].kind = ElementKind::Text("Hi".into());

        let mut sink = Vec::new();
        engine.frame(&mut sink).unwrap();

        assert!(!engine.renderer.front[(0, 0)].is_empty());
        assert!(!engine.renderer.front[(0, 1)].is_empty());
    }

    #[test]
    fn composite_respects_child_layer_order() {
        let mut engine = Engine::new(3, 1);

        let mut root_layer = engine.layers.get_root_mut();
        root_layer.position = Position::ZERO;
        root_layer[(0, 0)] = Cell::from_char('a', Style::new().foreground(Color::Index(1)));

        let child_layer_id = engine.layers.append_root(Layer::new(3, 1));
        let mut child_layer = engine.layers.get_mut(child_layer_id).unwrap();
        child_layer.position = Position::ZERO;
        child_layer.z_index = 1;
        child_layer[(0, 0)] = Cell::from_char('b', Style::new().foreground(Color::Index(2)));

        engine.composite();

        assert_eq!(engine.renderer.front[(0, 0)].as_str(&engine.renderer.arena), "b");
        assert_eq!(engine.renderer.front[(0, 0)].style, Style::new().foreground(Color::Index(2)));
    }
}
