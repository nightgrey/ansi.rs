use std::ops::{Index, IndexMut};
use derive_more::{Deref, DerefMut};
use crate::{Element, ElementId, GraphemeArena, Layer};
use tree::{RootTree,  Node, layout::prelude::*, id, Secondary, LayoutNode, LayoutContext, Tree, Map};
use geometry::{Rect, Size};
use grid::{Spatial};

pub type ElementNode = Node<ElementId, Element>;

id!(pub struct LayerId);
#[derive(Debug, Deref, DerefMut)]
pub struct Document {
    #[deref]
    #[deref_mut]
    pub elements: RootTree<ElementId, Element>,
    pub layouts:  Secondary<ElementId, LayoutNode>,
    pub bounds:   Secondary<ElementId, Rect>,
    pub layer_ids: Secondary<ElementId, LayerId>,
    pub layers:   Map<LayerId, Layer>,
    pub arena: GraphemeArena,
}

impl Document {
    pub fn new(width: usize, height: usize) -> Self {
        let mut elements = RootTree::new(Element::Div());
        let mut layouts = Secondary::new();
        let mut bounds = Secondary::new();

        let mut layers = Map::new();
        let id = layers.insert(Layer::new(width, height));
        let mut layer_ids = Secondary::new();
        layer_ids.insert(elements.root_id(), id);

        Self { elements, layouts, layer_ids, layers, bounds, arena: GraphemeArena::new() }
    }
    
    pub fn root_id(&self) -> ElementId {
        self.elements.root_id()
    }

    pub fn root_layer_id(&self) -> LayerId {
        *self.layer_ids.get(self.root_id()).unwrap()
    }
    
    pub fn root(&self) -> &ElementNode {
        self.elements.root()
    }
    
    pub fn root_mut(&mut self) -> &mut ElementNode {
        self.elements.root_mut()
    }

    pub fn viewport(&self) -> Size {
        self.get_layer(self.root_id()).unwrap().size()
    }

    pub fn get(&self, id: ElementId) -> Option<&ElementNode> {
        self.elements.get(id)
    }

    pub fn get_layer_id(&self, id: ElementId) -> Option<LayerId> {
        self.layer_ids.get(id).copied()
    }

    pub fn get_layer(&self, id: ElementId) -> Option<&Layer> {
        self.layer_ids.get(id).map(|&id|self.layers.get(id)).flatten()
    }

    pub fn get_layer_mut(&mut self, id: ElementId) -> Option<&mut Layer> {
        self.layer_ids.get(id).map(|&id|self.layers.get_mut(id)).flatten()
    }

    pub fn set_layer_id(&mut self, id: ElementId, layer_id: LayerId) {
        self.layer_ids.insert(id, layer_id);
    }

    pub fn get_bounds(&self, id: ElementId) -> Option<&Rect> {
        self.bounds.get(id)
    }

    pub fn set_bounds(&mut self, id: ElementId, rect: Rect) {
        self.bounds.insert(id, rect);
    }

    pub fn compute_layers(
        &mut self,
        id: ElementId,
        layer_id: LayerId,
    ) {
        let layer_id = self.get_layer_id(id).unwrap_or(layer_id);

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            let child = &self.elements[child_id];
            let next_layer_id = if child.is_promoting() {
                match self.get_layer_id(child_id) {
                    Some(layer_id) => layer_id,
                    None => {
                        let size = self.layers.get(layer_id).unwrap().size();
                        let layer_id = self.layers.insert(Layer::new(size.width, size.height));
                        self.set_layer_id(child_id, layer_id);
                        layer_id
                    }
                }
            } else {
                layer_id
            };

            self.compute_layers(child_id, next_layer_id);
        }
    }

    pub fn compute_layout(&mut self) {
        let viewport = self.viewport();
        let root_id = self.elements.root_id();

        // Compute layout
        LayoutTree::compute_layout(self, root_id, LayoutSize {
            width: AvailableSpace::Definite(viewport.width as f32),
            height: AvailableSpace::Definite(viewport.height as f32),
        });

        self.compute_bounds(root_id, 0.0, 0.0);
    }

    fn compute_bounds(&mut self, id: ElementId, offset_x: f32, offset_y: f32) {
        let computation = self.layouts[id].final_computation;
        let x = offset_x + computation.location.x;
        let y = offset_y + computation.location.y;
        let w = computation.size.width;
        let h = computation.size.height;

        self.set_bounds(id, Rect::bounds(x as usize, y as usize, w as usize, h as usize));

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            self.compute_bounds(child_id, x, y);
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let root_id = self.root_id();
        let layer = self.get_layer_mut(root_id).unwrap();
        layer.resize(width, height);
        self.set_bounds(root_id, Rect::bounds(0, 0, width, height));
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layer_ids.clear();
        self.arena.clear();
        // Re-sync: clear taffy and re-add root node
        self.layouts.clear();

        let viewport = self.viewport();
        self.set_bounds(self.root_id(), Rect::bounds(0, 0, viewport.width, viewport.height));
    }
}


impl LayoutTree<ElementId, Element, Rect> for Document {
    fn as_context<MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, ElementId, Option<&mut Rect>, &Layout) -> LayoutSize<f32>>(&mut self, measure: MeasureFunction) -> LayoutContext<'_, ElementId, Element, Rect, MeasureFunction> {
        LayoutContext {
            tree: &mut self.elements,
            layouts: &mut self.layouts,
            contexts: &mut self.bounds,
            measure_function: measure,
        }
    }

    fn use_rounding(&self) -> bool {
        true
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
    type Output = Layer;

    fn index(&self, index: LayerId) -> &Self::Output {
        &self.layers[index]
    }
}

impl IndexMut<LayerId> for Document {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        &mut self.layers[index]
    }
}