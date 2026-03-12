use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut};
use crate::{Direction, Element, ElementId, GraphemeArena, Layer, LayerId, Layout};
use tree::{RootTree, At, NodeRef, NodeRefMut, Node};
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
pub type Layouts = taffy::TaffyTree<Rect>;
#[derive(Debug, Deref, DerefMut)]
pub struct Document {
    #[deref]
    #[deref_mut]
    pub elements: Elements,
    pub layers: Layers,
    pub layouts: Layouts,

    pub arena: GraphemeArena,
}

impl Document {
    pub fn new(width: usize, height: usize) -> Self {
        let layers = RootTree::new(Layer::new(width, height));
        let mut layout = taffy::TaffyTree::<Rect>::new();

        let mut root_element = Element::Div(Direction::Vertical).on(layers.root_id());
        let taffy_node = layout.new_leaf_with_context(root_element.layout.clone(), Rect::bounds(0, 0, width, height)).unwrap();
        root_element.layout_id = taffy_node;

        let elements = RootTree::new(root_element);
        Self { elements, layers, arena: GraphemeArena::new(), layouts: layout }
    }
    
    pub fn root_id(&self) -> ElementId {
        self.elements.root_id()
    }
    
    pub fn root(&self) -> &ElementNode {
        self.elements.root()
    }
    
    pub fn root_mut(&mut self) -> &mut ElementNode {
        self.elements.root_mut()
    }
    
    pub fn root_layer_id(&self) -> LayerId {
        self.layers.root_id()
    }
    
    pub fn root_layer(&self) -> &LayerNode {
        self.layers.root()
    }
    
    pub fn root_layer_mut(&mut self) -> &mut LayerNode {
        self.layers.root_mut()
    }

    pub fn viewport(&self) -> Rect {
        let root = self.layers.root();
        Rect::bounds(0, 0, root.width, root.height)
    }
    
    pub fn get(&self, id: ElementId) -> Option<&ElementNode> {
        self.elements.get(id)
    }

    pub fn get_layer(&self, id: LayerId) -> Option<&LayerNode> {
        self.layers.get(id)
    }

    pub fn get_bounds(&self, id: ElementId) -> Rect {
        self.layouts.get_node_context(self.elements[id].layout_id).copied().unwrap_or_default()
    }
    
    fn set_bounds(&mut self, id: ElementId, rect: Rect) {
        self.layouts.set_node_context(self.elements[id].layout_id, Some(rect)).unwrap();
    }

    /// Insert an element as a child of the root. Creates a corresponding taffy node.
    pub fn insert(&mut self, element: Element) -> ElementId {
        self.insert_at(element, At::Child(self.elements.root_id()))
    }

    /// Insert an element at the given position. Creates a corresponding taffy node.
    pub fn insert_at(&mut self, mut element: Element, at: At<ElementId>) -> ElementId {
        let taffy_node = self.layouts.new_leaf_with_context(element.layout.clone(), Rect::ZERO).unwrap();
        element.layout_id = taffy_node;
        self.elements.insert_at(element, at)
    }

    pub fn insert_layer_at(&mut self, layer: Layer, at: At<LayerId>) -> LayerId {
        self.insert_layer_at(layer, at)
    }

    pub fn compute_layers(
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
                self.layers.move_to(next_layer_id, At::Child(layer_id));
                next_layer_id
            } else {
                layer_id
            };

            self.compute_layers(child_id, next_layer_id);
        }
    }

    /// Sync the element tree into taffy, compute layout, and read back results.
    pub fn compute_layout(&mut self) {
        let viewport = self.viewport();
        let root_id = self.elements.root_id();

        // Sync element tree → taffy tree
        self.layout_element(root_id);

        // Compute layout
        let layout_id = self.elements[root_id].layout_id;
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(viewport.width() as f32),
            height: taffy::AvailableSpace::Definite(viewport.height() as f32),
        };
        self.layouts.compute_layout(layout_id, available).unwrap();

        self.layout_bounds(root_id, 0.0, 0.0);
    }

    fn layout_element(&mut self, id: ElementId) {
        let element = &self.elements[id];

        // Update or verify taffy node style
        self.layouts.set_style(element.layout_id, element.layout.clone()).unwrap();

        // Collect children and sync recursively
        let children: Vec<_> = self.elements.children(id).collect();
        for &child_id in &children {
            self.layout_element(child_id);
        }

        // Set taffy children
        let taffy_children: Vec<_> = children.iter().map(|&c| self.elements[c].layout_id).collect();
        self.layouts.set_children(self.elements[id].layout_id, &taffy_children).unwrap();
    }

    fn layout_bounds(&mut self, id: ElementId, offset_x: f32, offset_y: f32) {
        let taffy_layout = self.layouts.layout(self.elements[id].layout_id).unwrap();
        let x = offset_x + taffy_layout.location.x;
        let y = offset_y + taffy_layout.location.y;
        let w = taffy_layout.size.width;
        let h = taffy_layout.size.height;

        self.set_bounds(id, Rect::bounds(x as usize, y as usize, w as usize, h as usize));

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            self.layout_bounds(child_id, x, y);
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.layers.root_mut().resize(width, height);
        let layout_id = self.elements.root().layout_id;
        self.layouts.set_node_context(layout_id, Some(Rect::bounds(0, 0, width, height))).unwrap();
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
        self.arena.clear();
        // Re-sync: clear taffy and re-add root node
        self.layouts.clear();

        let viewport = self.viewport();
        let root = self.elements.root_mut();
        root.layout_id = self.layouts.new_leaf_with_context(root.layout.clone(), viewport).unwrap();
    }
}

impl Index<ElementId> for Document {
    type Output = Element;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.elements[index]
    }
}

impl IndexMut<ElementId> for Document {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        &mut self.elements[index]
    }
}

impl Index<LayerId> for Document {
    type Output = LayerNode;

    fn index(&self, index: LayerId) -> &Self::Output {
        &self.layers[index]
    }
}

impl IndexMut<LayerId> for Document {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        &mut self.layers[index]
    }
}