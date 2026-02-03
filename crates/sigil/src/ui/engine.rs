use crate::{ElementId, Element, Layer, LayerId};
use super::tree::*;

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

    pub fn add(&mut self, node: Element) {
        self.elements.insert(node);
    }
}
