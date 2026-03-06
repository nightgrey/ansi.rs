use super::{Key, Tree};
use crate::{
    Direction, DoubleBuffer, Element, ElementId, ElementKind, Layer, LayerId, NodeRef,
    NodeRefMut, Rasterizer, Secondary,
};
use geometry::{Rect};
use std::io::Write;

pub type ElementRef<'a> = NodeRef<'a, ElementId, Element>;
pub type ElementRefMut<'a> = NodeRefMut<'a, ElementId, Element>;
pub type LayerRef<'a> = NodeRef<'a, LayerId, Layer>;
pub type LayerRefMut<'a> = NodeRefMut<'a, LayerId, Layer>;

pub struct Engine {
    pub elements: Tree<ElementId, Element>,
    pub layers: Tree<LayerId, Layer>,
    pub layout: Secondary<ElementId, Rect>,
    pub screen: DoubleBuffer,
    pub rasterizer: Rasterizer,
    pub root: ElementId,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let mut elements = Tree::new();
        let root = elements.insert(Element::container(Direction::Vertical));

        let mut layers = Tree::new();
        let layer_id = layers.insert(Layer::new(width, height));

        elements[root].layer_id = Some(layer_id);

        Self {
            elements,
            layers,
            screen: DoubleBuffer::new(width, height),
            rasterizer: Rasterizer::new(width, height),
            layout: Secondary::new(),
            root,
        }
    }

    pub fn root(&self) -> Option<ElementId> {
        self.root.option()
    }

    // Layering

    fn layer_element(&mut self, id: ElementId, layer_id: Option<LayerId>) {
        if let Some(mut element) = self.elements.get_mut(id) {
            // If element creates its own layer, make one; otherwise inherit
            let layer_id = if element.promotes() {
                Some(
                    self.layers
                        .insert(Layer::new(self.screen.width, self.screen.height)),
                )
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
        if let Some(root) = self.root() {
            let root_layer = self.elements[root].layer_id;
            self.layer_element(root, root_layer);
        }
    }

    // Layouting

    pub fn layout(&mut self) {
        if let Some(root) = self.root() {
            self.layout_element(
                root,
                Rect::bounds(0, 0, self.screen.width, self.screen.height),
            );
        }
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
        if let Some(root) = self.root() {
            self.paint_element(root);
        }

        // Mark layers clean
        for (_, layer) in &mut self.layers {
            layer.is_dirty = false;
        }
    }

    fn paint_element(&mut self, id: ElementId) {
        let rect = self.layout[id];
        let element = &self.elements[id];
        let layer_id = element.layer_id.expect("element without layer");

        match &element.kind {
            ElementKind::Text(content) => {
                let layer = &mut self.layers[layer_id];
                // layer.text(0.., content, Style::EMPTY);
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
        self.screen.front.clear();

        // Composite back-to-front
        for layer_id in layers {
            let layer = &self.layers[layer_id];
            for (i, cell) in layer.iter().enumerate() {
                if !cell.is_empty() {
                    self.screen.front[i].clone_from(cell);
                }
            }
        }
    }

    // Rendering
    pub fn render(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        self.rasterizer.render(&self.screen.front);
        self.rasterizer.flush(out)?;
        self.screen.swap();
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
