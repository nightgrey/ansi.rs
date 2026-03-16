use super::prelude::*;

/// A layout-aware tree node.
///
/// Stores the CSS style, a layout cache for memoization, and both the
/// unrounded and final (rounded) computed layouts. Typically you interact
/// with this through [`LayoutTree`](super::LayoutTree) rather than directly.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// The CSS style properties used as input to layout computation.
    pub layout: Layout,
    pub(crate) cache: LayoutCache,
    pub(crate) unrounded_computation: Computation,
    pub final_computation: Computation,
}

impl LayoutNode {
    /// Creates a new layout node with the given style and default (zero) layout.
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
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
