use crate::{Direction, Element, ElementId, ElementKind, GraphemeArena, Layer, LayerId, TreeNodeRef, TreeNodeRefMut, Rasterizer, Secondary, Buffer, RootTree, TreeId, Tree};
use geometry::{Rect};
use std::io::Write;

pub type ElementRef<'a> = TreeNodeRef<'a, ElementId, Element>;
pub type ElementRefMut<'a> = TreeNodeRefMut<'a, ElementId, Element>;
pub type LayerRef<'a> = TreeNodeRef<'a, LayerId, Layer>;
pub type LayerRefMut<'a> = TreeNodeRefMut<'a, LayerId, Layer>;

pub struct Engine {
    pub front: Buffer,
    pub back: Buffer,
    arena: GraphemeArena,
    pub elements: RootTree<ElementId, Element>,
    pub layers: RootTree<LayerId, Layer>,
    pub layout: Secondary<ElementId, Rect>,
    rasterizer: Rasterizer,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let mut elements = RootTree::new(Element::container(Direction::Vertical));
        let root = elements.root;

        let mut layers = RootTree::new(Layer::new(width, height));

        elements[root].layer_id = layers.root;

        let mut layout = Secondary::new();
        layout.insert(root, Rect::bounds(0, 0, width, height));

        Self {
            elements,
            layers,
            layout,
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            arena: GraphemeArena::new(),
            rasterizer: Rasterizer::new(width, height),
        }
    }

    fn layer_element(&mut self, id: ElementId, layer_id: Option<LayerId>) {
        if self.elements.contains(id) {
            let element = &mut self.elements[id];
            
            // // If element creates its own layer, make one; otherwise inherit
            // let layer_id = if element.promotes() {
            //         self.layers
            //             .insert(Layer::new(self.front.width, self.front.height))
            // } else {
            //     layer_id.unwrap_or(LayerId::none())
            // };

            element.layer_id = layer_id.unwrap_or_default();

            let layer_id = element.layer_id.as_option();
            for child in self.elements.children(id).collect::<Vec<_>>() {
                self.layer_element(child, layer_id);
            }
        }
    }

    // Layouting
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
                        for (i, child) in children.iter().enumerate() {
                            let child_bounds = Rect::bounds(
                                bounds.x(),
                                bounds.y() + (i * child_height),
                                bounds.width(),
                                child_height,
                            );
                            self.layout_element(*child, child_bounds);
                        }
                    }
                    Direction::Horizontal => {
                        let child_width = bounds.width() / child_count;
                        for (i, child) in children.iter().enumerate() {
                            let child_bounds = Rect::bounds(
                                bounds.x() + (i * child_width),
                                bounds.y(),
                                child_width,
                                bounds.height(),
                            );
                            self.layout_element(*child, child_bounds);
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
    fn paint_element(&mut self, id: ElementId) {
        let element = &self.elements[id];
        let layer_id = element.layer_id;

        match &element.kind {
            ElementKind::Text(content) => {
                // TODO
            }
            ElementKind::Container { .. } => {
                // TODO
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
                    self.front[i] = *cell;
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

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn frame(&mut self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        let root_element = self.elements.root();
        let root_layer = self.layers.root();
        let root_layout = self.layout[root_element];

        // Layering
        self.layer_element(root_element, root_layer.as_option());

        // Layouting
        self.layout_element(
            root_element,
            root_layout,
        );

        // Painting

        for (_, layer) in &mut self.layers {
            if layer.is_dirty {
                layer.clear();
            }
        }

        self.paint_element(root_element);

        for (_, layer) in &mut self.layers {
            layer.is_dirty = false;
        }

        // Compositing

        self.composite();

        // Rendering

        self.render(out)?;
        Ok(())
    }
}
