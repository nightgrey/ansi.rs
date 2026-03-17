use super::prelude::*;

/// A layout-aware tree node.
///
/// Stores the computed layout: unrounded, rounded, and cached. Typically you interact
/// with this through [`LayoutTree`](super::AsLayoutContext) rather than directly.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub cache: LayoutCache,
    pub unrounded_computation: LayoutComputation,
    pub final_computation: LayoutComputation,
}

impl LayoutNode {
    /// Creates a new layout node with the given style and default (zero) layout.
    pub fn new() -> Self {
        Self {
            cache: LayoutCache::new(),
            unrounded_computation: LayoutComputation::with_order(0),
            final_computation: LayoutComputation::with_order(0),
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self::new()
    }
}
