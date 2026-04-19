use crate::{Computation, Display, Element, ElementId, ElementKind, FlexDirection, Layout, Space};
use compact_str::CompactString;
use slotmap::Key;
use taffy::{BlockContext, LayoutInput, LayoutOutput, TraversePartialTree};
use tree::{Id, Secondary, Tree};

/// Layout context that holds the tree itself along with a reference to the context.
/// It implements taffy's layout traits and allows for layout computation.
#[derive(Debug)]
pub struct LayoutContext<'d, 'n, M: MeasureFunction<'n>> {
    pub tree: &'d mut Tree<ElementId, Element<'n>>,
    pub layouts: &'d mut Secondary<ElementId, Computation>,
    pub measure_function: M,
}

impl<'d, 'n, M: MeasureFunction<'n>> LayoutContext<'d, 'n, M> {
    pub fn new(
        tree: &'d mut Tree<ElementId, Element<'n>>,
        layouts: &'d mut Secondary<ElementId, Computation>,
        measure_function: M,
    ) -> Self {
        Self {
            tree,
            layouts,
            measure_function,
        }
    }

    pub fn compute_layout(&mut self, id: ElementId, available_space: Space) {
        let taffy_id = Self::taffy_id(id);
        taffy::compute_root_layout(self, taffy_id, available_space.into());
        taffy::round_layout(self, taffy_id);
    }

    pub(crate) fn print_tree(&mut self, root: ElementId) {
        taffy::util::print_tree(self, Self::taffy_id(root));
    }

    fn style(&self, id: taffy::NodeId) -> &Layout {
        &self.node(id).layout
    }

    #[inline]
    fn node(&self, node_id: taffy::NodeId) -> &tree::Node<ElementId, Element<'n>> {
        &self.tree[Self::tree_id(node_id)]
    }

    #[inline]
    fn node_mut(&mut self, node_id: taffy::NodeId) -> &mut tree::Node<ElementId, Element<'n>> {
        &mut self.tree[Self::tree_id(node_id)]
    }

    #[inline]
    fn layout_node(&self, node_id: taffy::NodeId) -> &Computation {
        &self.layouts[Self::tree_id(node_id)]
    }

    #[inline]
    fn layout_node_mut(&mut self, node_id: taffy::NodeId) -> &mut Computation {
        &mut self.layouts[Self::tree_id(node_id)]
    }

    #[inline]
    fn taffy_id<K: Id>(id: K) -> taffy::NodeId {
        taffy::NodeId::new(id.data().as_ffi())
    }

    #[inline]
    fn tree_id<K: Id>(id: taffy::NodeId) -> K {
        slotmap::KeyData::from_ffi(u64::from(id)).into()
    }
}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::TraversePartialTree for LayoutContext<'d, 'n, M> {
    type ChildIter<'a>
        = LayoutChildren<'a>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        LayoutChildren(self.tree.children(Self::tree_id(parent_node_id)))
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.tree.children(Self::tree_id(parent_node_id)).count()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        Self::taffy_id(
            self.tree
                .children(Self::tree_id(parent_node_id))
                .nth(child_index)
                .unwrap(),
        )
    }
}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::TraverseTree for LayoutContext<'d, 'n, M> {}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::LayoutPartialTree for LayoutContext<'d, 'n, M> {
    type CoreContainerStyle<'a>
        = &'a Layout
    where
        Self: 'a;
    type CustomIdent = CompactString;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        self.style(node_id)
    }

    #[inline(always)]
    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.layout_node_mut(node_id).unrounded_layout = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        if inputs.run_mode == taffy::RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        taffy::compute_cached_layout(self, node_id, inputs, |ctx, node_id, inputs| {
            let node_key = Self::tree_id(node_id);
            let has_children = ctx.child_count(node_id) > 0;

            let node = &ctx.tree[node_key];
            let layout = &node.layout;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (layout.display, has_children) {
                (Display::None, _) => taffy::compute_hidden_layout(ctx, node_id),
                (Display::Block, true) => taffy::compute_block_layout(ctx, node_id, inputs, None),
                (Display::Flex, true) => taffy::compute_flexbox_layout(ctx, node_id, inputs),
                (_, false) | (Display::Inline, _) => taffy::compute_leaf_layout(
                    inputs,
                    layout,
                    |_, _| 0.0,
                    |known_dimensions, available_space| {
                        (ctx.measure_function)(known_dimensions, available_space, node_key, node)
                    },
                ),
            }
        })
    }
}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::CacheTree for LayoutContext<'d, 'n, M> {
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        input: &taffy::LayoutInput,
    ) -> Option<taffy::LayoutOutput> {
        self.layout_node(node_id).cache.get(input)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        input: &taffy::LayoutInput,
        layout_output: taffy::LayoutOutput,
    ) {
        self.layout_node_mut(node_id)
            .cache
            .store(input, layout_output);
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.layout_node_mut(node_id).cache.clear();
    }
}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::LayoutFlexboxContainer for LayoutContext<'d, 'n, M> {
    type FlexboxContainerStyle<'a>
        = &'a Layout
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
        = &'a Layout
    where
        Self: 'a;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        self.style(node_id)
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.style(child_node_id)
    }
}
// impl<'d, 'n, MeasureFunction> taffy::LayoutGridContainer for LayoutContext<'d, 'n, MeasureFunction>
// {
//     type GridContainerStyle<'a>
//     = &'a Style
//     where
//         Self: 'a;
//     type GridItemStyle<'a>
//     = &'a Style
//     where
//         Self: 'a;
//
//     fn get_grid_container_style(&self, node_id: taffy::NodeId) -> Self::GridContainerStyle<'_> {
//         taffy::LayoutPartialTree::get_core_container_style(self, node_id)
//     }
//
//     fn get_grid_child_style(&self, child_node_id: taffy::NodeId) -> Self::GridItemStyle<'_> {
//         taffy::LayoutPartialTree::get_core_container_style(self, child_node_id)
//     }
// }

impl<'d, 'n, M: MeasureFunction<'n>> taffy::LayoutBlockContainer for LayoutContext<'d, 'n, M> {
    type BlockContainerStyle<'a>
        = &'a Layout
    where
        Self: 'a;
    type BlockItemStyle<'a>
        = &'a Layout
    where
        Self: 'a;

    fn get_block_container_style(&self, node_id: taffy::NodeId) -> Self::BlockContainerStyle<'_> {
        self.style(node_id)
    }

    fn get_block_child_style(&self, child_node_id: taffy::NodeId) -> Self::BlockItemStyle<'_> {
        self.style(child_node_id)
    }

    fn compute_block_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: LayoutInput,
        block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        taffy::compute_block_layout(self, node_id, inputs, block_ctx)
    }
}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::RoundTree for LayoutContext<'d, 'n, M> {
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.layout_node(node_id).unrounded_layout
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.layout_node_mut(node_id).final_layout = *layout;
    }
}

impl<'d, 'n, M: MeasureFunction<'n>> taffy::PrintTree for LayoutContext<'d, 'n, M> {
    fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
        let style = self.style(node_id);

        match style.display {
            Display::Inline => match &self.node(node_id).kind {
                ElementKind::Span(_) => "Span (Inline)",
                ElementKind::Div => "Div (Inline)",
            },
            Display::Block => match &self.node(node_id).kind {
                ElementKind::Span(_) => "Span (Block)",
                ElementKind::Div => "Div (Block)",
            },
            Display::Flex => match &self.node(node_id).kind {
                ElementKind::Span(_) => "Span (Flex)",
                ElementKind::Div => "Div (Flex)",
            },
            Display::None => match &self.node(node_id).kind {
                ElementKind::Span(_) => "Span (None)",
                ElementKind::Div => "Div (None)",
            },
        }
    }

    fn get_final_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.layout_node(node_id).final_layout
    }
}

/// Iterates over the children of a node, returning the [`taffy::NodeId`] of each.
pub struct LayoutChildren<'a>(tree::iter::Children<'a, ElementId, Element<'a>>);

impl<'a> Iterator for LayoutChildren<'a> {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|id| taffy::NodeId::new(id.data().as_ffi()))
    }
}

/// Measures the intrinsic size of a leaf (inline / no-children) node.
///
/// Blanket-implemented for any `FnMut` with the matching signature — this trait
/// exists solely to collapse an otherwise 7-fold-repeated where-clause on
/// [`LayoutContext`] and its taffy trait impls.
pub trait MeasureFunction<'n>:
    FnMut(
    taffy::Size<Option<f32>>,
    taffy::Size<taffy::AvailableSpace>,
    ElementId,
    &Element<'n>,
) -> taffy::Size<f32>
{
}

impl<'n, F> MeasureFunction<'n> for F where
    F: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        ElementId,
        &Element<'n>,
    ) -> taffy::Size<f32>
{
}
