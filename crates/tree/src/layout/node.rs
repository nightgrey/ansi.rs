use super::prelude::*;

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub layout: Layout,
    pub(super) cache: LayoutCache,
    pub(super) unrounded_computation: Computation,
    pub(super) final_computation: Computation,
}

impl LayoutNode {
    pub fn new(style: Layout) -> Self {
        Self {
            layout: style,
            cache: LayoutCache::new(),
            unrounded_computation: Computation::with_order(0),
            final_computation: Computation::with_order(0),
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self::new(Layout::default())
    }
}


