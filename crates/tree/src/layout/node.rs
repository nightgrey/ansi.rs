use derive_more::{Deref, DerefMut};
use super::prelude::*;


/// A layout-aware tree node.
///
/// Stores the computed layout: unrounded, rounded, and cached. Typically you interact
/// with this through [`LayoutTree`](super::AsLayoutContext) rather than directly.
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct LayoutNode {
    #[deref]
    #[deref_mut]
    pub layout: Layout,
    pub cache: LayoutCache,
    pub unrounded_computation: LayoutComputation,
    pub final_computation: LayoutComputation,
}

impl LayoutNode {
    /// Creates a new layout node with the given style and default (zero) layout.
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            cache: LayoutCache::new(),
            unrounded_computation: LayoutComputation::default(),
            final_computation: LayoutComputation::default(),
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self::new(Layout::default())
    }
}
