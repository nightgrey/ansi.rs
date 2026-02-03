use super::tree::*;
use crate::{Element, ElementId, Layer, LayerId};

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
