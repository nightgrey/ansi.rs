use taffy::{GridContainerStyle, Style, TaffyTree};
use crate::{ElementNode, LayerNode, RootTree};

pub struct Engine {
    elements: RootTree<ElementNode>,
    layers: RootTree<LayerNode>,
    layout: TaffyTree
}

impl Engine {
    pub fn new() -> Self {
        Self {
            elements: RootTree::new(ElementNode::container()),
            layers: RootTree::new(LayerNode::ZERO),
            layout: TaffyTree::new()
        }
    }

    pub fn add(&mut self, node: ElementNode) {
        self.elements.add(node);
    }
}
