use super::tree::Tree;
use crate::{Element, ElementId, Layer, LayerId, NodeRef, NodeRefMut};

pub type ElementRef<'a> = NodeRef<'a, ElementId, Element>;
pub type ElementRefMut<'a> = NodeRefMut<'a, ElementId, Element>;
pub type LayerRef<'a> = NodeRef<'a, LayerId, Layer>;
pub type LayerRefMut<'a> = NodeRefMut<'a, LayerId, Layer>;

pub struct Engine {
    elements: Tree<ElementId, Element>,
    layers: Tree<LayerId, Layer>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            elements: Tree::new(),
            layers: Tree::new(),
        }
    }

    pub fn get_element(&self, id: ElementId) -> Option<ElementRef> {
        self.elements.get(id)
    }

    pub fn get_element_mut(&mut self, id: ElementId) -> Option<ElementRefMut> {
        self.elements.get_mut(id)
    }

    pub fn add_element(&mut self, element: Element) -> ElementId {
        self.elements.insert(element)
    }

    pub fn get_layer(&self, id: LayerId) -> Option<LayerRef> {
        self.layers.get(id)
    }

    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<LayerRefMut> {
        self.layers.get_mut(id)
    }

    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        self.layers.insert(layer)
    }
}

#[test]
fn test_add_layer() {
    let mut engine = Engine::new();
    let layer_id = engine.add_layer(Layer::new(10, 5));
    let root_id = engine.add_element(Element::container());
    let root = engine.get_element(root_id).unwrap();
    assert_eq!(root.layer_id, Some(layer_id));
    assert_eq!(root.layer_id.unwrap(), layer_id);
}
