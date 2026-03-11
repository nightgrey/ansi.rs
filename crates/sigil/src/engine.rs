use crate::{Tree, TreeId, Direction, Element, ElementId, ElementKind, GraphemeArena, Layer, LayerId, NodeRef, NodeRefMut, Rasterizer, Secondary, Buffer, RootTree};
use geometry::{Rect};
use std::io::Write;
use crate::painter::Painter;

pub type ElementRef<'a> = NodeRef<'a, ElementId, Element>;
pub type ElementRefMut<'a> = NodeRefMut<'a, ElementId, Element>;
pub type LayerRef<'a> = NodeRef<'a, LayerId, Layer>;
pub type LayerRefMut<'a> = NodeRefMut<'a, LayerId, Layer>;

pub struct Engine {
    pub elements: RootTree<ElementId, Element>,
    pub layers: RootTree<LayerId, Layer>,
    pub layout: Secondary<ElementId, Rect>,
    pub arena: GraphemeArena,
    pub rasterizer: Rasterizer,
    pub front: Buffer,
    pub back: Buffer,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let mut elements = RootTree::new(Element::container(Direction::Vertical));
        let root = elements.root;

        let mut layers = RootTree::new(Layer::new(width, height));
        let layer_id = layers.root;

        elements[root].layer_id = Some(layer_id);

        Self {
            elements,
            layers,
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            arena: GraphemeArena::new(),
            rasterizer: Rasterizer::new(width, height),
            layout: Secondary::new(),
        }
    }

    pub fn root(&self) -> ElementId {
        self.elements.root
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    // Layering
    fn layer_element(&mut self, id: ElementId, layer_id: Option<LayerId>) {
        if let Some(mut element) = self.elements.get_mut(id) {
            // If element creates its own layer, make one; otherwise inherit
            let layer_id = if element.promotes() {
                    Some(self.layers
                        .insert(Layer::new(self.front.width, self.front.height)))
            } else {
                layer_id
            };

            element.layer_id = layer_id;

            // Recurse
            let children = element.children().collect::<Vec<_>>();

            for child in children {
                self.layer_element(child, layer_id);
            }
        }
    }

    pub fn layer(&mut self) {
            let root_layer = self.elements[self.root()].layer_id;
            self.layer_element(self.root(), root_layer);
    }

    // Layouting

    pub fn layout(&mut self) {
            self.layout_element(
                self.root(),
                Rect::bounds(0, 0, self.front.width, self.front.height),
            );
    }

    fn layout_element(&mut self, id: ElementId, bounds: Rect) {
        self.layout.insert(id, bounds);

        match &self.elements[id].kind {
            ElementKind::Container { direction } => {
                let direction = *direction;
                let children: Vec<_> = self.elements.children(id).collect();
                let child_count = children.len();

                if child_count == 0 {
                    return;
                }

                // Naive equal division
                match direction {
                    Direction::Vertical => {
                        let child_height = bounds.height() / child_count;
                        for (i, child) in children.into_iter().enumerate() {
                            let child_bounds = Rect::bounds(
                                bounds.x(),
                                bounds.y() + (i * child_height),
                                bounds.width(),
                                child_height,
                            );
                            self.layout_element(child, child_bounds);
                        }
                    }
                    Direction::Horizontal => {
                        let child_width = bounds.width() / child_count;
                        for (i, child) in children.into_iter().enumerate() {
                            let child_bounds = Rect::bounds(
                                bounds.x() + (i * child_width),
                                bounds.y(),
                                child_width,
                                bounds.height(),
                            );
                            self.layout_element(child, child_bounds);
                        }
                    }
                }
            }
            ElementKind::Text { .. } => {
                // Text is a leaf, already has bounds assigned
            }
        }
    }

    // Painting

    pub fn paint(&mut self) {
        // Clear all dirty layers
        for (_, layer) in &mut self.layers {
            if layer.is_dirty {
                layer.clear();
            }
        }

        // Paint all elements
        self.paint_element(self.root());

        // Mark layers clean
        for (_, layer) in &mut self.layers {
            layer.is_dirty = false;
        }
    }

    fn paint_element(&mut self, id: ElementId) {
        let element = &self.elements[id];
        let layer_id = element.layer_id.expect("element without layer");

        let mut painter = Painter::new(&mut self.layers[layer_id], &mut self.arena);

        match &element.kind {
            ElementKind::Text(content) => {
                painter.draw_text(0, 0, content, element.style);
            }
            ElementKind::Container { .. } => {
                // Containers don't paint themselves, just their children
            }
        }

        // Paint children
        let children: Vec<_> = self.elements.children(id).collect();
        for child in children {
            self.paint_element(child);
        }
    }

    // Compositing
    pub fn composite(&mut self) {
        let mut layers = self.layers.keys().collect::<Vec<_>>();
        layers.sort_by_key(|id| self.layers[*id].z_index);

        // Clear front buffer
        self.front.clear();

        // Composite back-to-front
        for layer_id in layers {
            let layer = &self.layers[layer_id];
            for (i, cell) in layer.iter().enumerate() {
                if !cell.is_empty() {
                    self.front[i].clone_from(cell);
                }
            }
        }
    }

    // Rendering
    pub fn render(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        self.rasterizer.render(&self.front, &self.arena);
        self.rasterizer.flush(out)?;
        self.swap();
        Ok(())
    }

    pub fn frame(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        self.layer();
        self.layout();
        self.paint();
        self.composite();
        self.render(out)?;
        Ok(())
    }
}
