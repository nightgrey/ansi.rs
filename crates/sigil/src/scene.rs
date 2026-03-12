use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut};
use crate::{Direction, Element, ElementId, ElementKind, Layer, LayerId};
use tree::{RootTree, SecondaryTree, At, NodeRef, NodeRefMut, Node};
use geometry::Rect;
use grid::{Spatial};

pub type ElementNode = Node<ElementId, Element>;
pub type ElementRef<'a> = NodeRef<'a, ElementId, Element>;
pub type ElementRefMut<'a> = NodeRefMut<'a, ElementId, Element>;
pub type LayerNode = Node<LayerId, Layer>;
pub type LayerRef<'a> = NodeRef<'a, LayerId, Layer>;
pub type LayerRefMut<'a> = NodeRefMut<'a, LayerId, Layer>;

pub type Elements = RootTree<ElementId, Element>;
pub type Layers = RootTree<LayerId, Layer>;
pub type Layout = SecondaryTree<ElementId, Rect>;

#[derive(Debug, Deref, DerefMut)]
pub struct Scene {
    #[deref]
    #[deref_mut]
    pub elements: Elements,
    pub layers: Layers,
    pub layout: Layout,
}

impl Scene {
    pub fn new(width: usize, height: usize) -> Self {
        let layers = RootTree::new(Layer::new(width, height));
        let elements = RootTree::new(Element::container(Direction::Vertical).on(layers.root_id()));
        let mut layout = SecondaryTree::new();
        layout.insert(elements.root_id(), Rect::bounds(0, 0, width, height));
        Self { elements, layers, layout }
    }

    pub fn root_id(&self) -> ElementId {
        self.elements.root_id()
    }

    pub fn root_layer_id(&self) -> LayerId {
        self.layers.root_id()
    }
    
    pub fn get(&self, id: ElementId) -> Option<&ElementNode> {
        self.elements.get(id)
    }

    pub fn layer(
        &mut self,
        id: ElementId,
        layer_id: LayerId,
    ) {
        self.elements[id].layer_id = layer_id;

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            let child = &self.elements[child_id];
            let next_layer_id = if child.is_promoting() {
                let size = &self.layers[layer_id].size();
                let next_layer_id = self.layers.insert(Layer::new(size.width, size.height));
                self.layers.move_to(layer_id, At::Append(next_layer_id));
                next_layer_id
            } else {
                layer_id
            };

            self.layer(child_id, next_layer_id);
        }
    }

    pub  fn layout(
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
        self.layers.root_mut().resize(width, height);
        self.layout.insert(self.elements.root_id(), Rect::bounds(0, 0, width, height));
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
        self.layout.clear();
    }
}

impl Index<ElementId> for Scene {
    type Output = Element;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.elements[index]
    }
}

impl IndexMut<ElementId> for Scene {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        &mut self.elements[index]
    }
}

impl Index<LayerId> for Scene {
    type Output = LayerNode;

    fn index(&self, index: LayerId) -> &Self::Output {
        &self.layers[index]
    }
}

impl IndexMut<LayerId> for Scene {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        &mut self.layers[index]
    }
}