use crate::{Id, LayoutNode, Secondary, Tree};
use crate::{prelude::*, Bridge, LayoutNodeId};

/// Layout context that holds the tree itself along with a reference to the context.
/// It implements taffy's layout traits and allows for layout computation.
#[derive(Debug)]
pub struct LayoutContext<'t, K: Id, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    pub tree: &'t mut Tree<K, V>,
    pub layouts: &'t mut Secondary<K, LayoutNode>,
    pub measure_function: MeasureFunction,
}

impl<'t, K: Id, V,MeasureFunction> LayoutContext<'t, K, V,  MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    #[inline(always)]
    fn get(&self, layout_id: LayoutNodeId) -> &LayoutNode {
        &self.layouts[K::from_layout_id(layout_id)]
    }

    #[inline(always)]
    fn get_mut(&mut self, layout_id: LayoutNodeId) -> &mut LayoutNode {
        &mut self.layouts[K::from_layout_id(layout_id)]
    }
}

pub struct LayoutChildren<'a, K: Id, V>(crate::iter::Children<'a, K, V>);

impl<K: Id, V> Iterator for LayoutChildren<'_, K, V> {
    type Item = LayoutNodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bridge::into_layout_id)
    }
}

impl<K: Id, V, MeasureFunction> TraverseLayoutPartialTree for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    type ChildIter<'a>
    = LayoutChildren<'a, K, V>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: LayoutNodeId) -> Self::ChildIter<'_> {
        LayoutChildren(self.tree.children(K::from_layout_id(parent_node_id)))
    }

    fn child_count(&self, parent_node_id: LayoutNodeId) -> usize {
        self.tree.children(K::from_layout_id(parent_node_id)).count()
    }

    fn get_child_id(&self, parent_node_id: LayoutNodeId, child_index: usize) -> LayoutNodeId {
        let child_key = self.tree.children(K::from_layout_id(parent_node_id)).nth(child_index).unwrap();
        child_key.into_layout_id()
    }
}

impl<K: Id, V, MeasureFunction> TraverseLayoutTree for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
}

impl<K: Id, V, MeasureFunction> LayoutPartialTree for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    type CoreContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type CustomIdent = String;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: LayoutNodeId) -> Self::CoreContainerStyle<'_> {
        &self.layouts[K::from_layout_id(node_id)].layout
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: LayoutNodeId, layout: &LayoutComputation) {
        self.get_mut(node_id).unrounded_computation = *layout;
    }

    #[inline(always)]
    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
    }

    fn compute_child_layout(&mut self, node_id: LayoutNodeId, inputs: LayoutInput) -> LayoutOutput {
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        taffy::compute_cached_layout(self, node_id, inputs, |ctx, node_id, inputs| {
            let node_key = K::from_layout_id(node_id);
            let layout = ctx.get_core_container_style(node_id);
            let display_mode = layout.display;
            let has_children = ctx.child_count(node_id) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => taffy::compute_hidden_layout(ctx, node_id),
                (Display::Block, true) => taffy::compute_block_layout(ctx, node_id, inputs),
                (Display::Flex, true) => taffy::compute_flexbox_layout(ctx, node_id, inputs),
                (Display::Grid, true) => taffy::compute_grid_layout(ctx, node_id, inputs),
                (_, false) => {
                    let node = &mut ctx.tree[node_key];
                    let layout = &ctx.layouts[node_key].layout;
                    let measure_function = |known_dimensions, available_space| {
                        (ctx.measure_function)(known_dimensions, available_space, node_key, node, layout)
                    };
                    // TODO: implement calc() in high-level API
                    taffy::compute_leaf_layout(inputs, layout, |_, _| 0.0, measure_function)
                }
            }
        })
    }
}

impl<K: Id, V, MeasureFunction> CacheLayoutTree for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    fn cache_get(
        &self,
        node_id: LayoutNodeId,
        known_dimensions: layout::Size<Option<f32>>,
        available_space: layout::Size<AvailableSpace>,
        run_mode: RunMode,
    ) -> Option<LayoutOutput> {
        self.get(node_id)
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: LayoutNodeId,
        known_dimensions: layout::Size<Option<f32>>,
        available_space: layout::Size<AvailableSpace>,
        run_mode: RunMode,
        layout_output: LayoutOutput,
    ) {
        self.get_mut(node_id)
            .cache
            .store(known_dimensions, available_space, run_mode, layout_output);
    }

    fn cache_clear(&mut self, node_id: LayoutNodeId) {
        self.get_mut(node_id).cache.clear();
    }
}

impl<K: Id, V, MeasureFunction> LayoutFlexboxContainer for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    type FlexboxContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
    = &'a Layout
    where
        Self: 'a;

    fn get_flexbox_container_style(&self, node_id: LayoutNodeId) -> Self::FlexboxContainerStyle<'_> {
       self.get_core_container_style(node_id)
    }

    fn get_flexbox_child_style(&self, child_node_id: LayoutNodeId) -> Self::FlexboxItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl<K: Id, V, MeasureFunction> LayoutGridContainer for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    type GridContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type GridItemStyle<'a>
    = &'a Layout
    where
        Self: 'a;

    fn get_grid_container_style(&self, node_id: LayoutNodeId) -> Self::GridContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    fn get_grid_child_style(&self, child_node_id: LayoutNodeId) -> Self::GridItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl<K: Id, V, MeasureFunction> LayoutBlockContainer for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    type BlockContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type BlockItemStyle<'a>
    = &'a Layout
    where
        Self: 'a;

    fn get_block_container_style(&self, node_id: LayoutNodeId) -> Self::BlockContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    fn get_block_child_style(&self, child_node_id: LayoutNodeId) -> Self::BlockItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl<K: Id, V, MeasureFunction> RoundLayoutTree for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    fn get_unrounded_layout(&self, node_id: LayoutNodeId) -> LayoutComputation {
        self.get(node_id).unrounded_computation
    }

    fn set_final_layout(&mut self, node_id: LayoutNodeId, layout: &LayoutComputation) {
        self.get_mut(node_id).final_computation = *layout;
    }
}

impl<K: Id, V, MeasureFunction> PrintLayoutTree for LayoutContext<'_, K, V, MeasureFunction>
where
    MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>,
{
    fn get_debug_label(&self, node_id: LayoutNodeId) -> &'static str {
        let layout = self.get_core_container_style(node_id);
        let num_children = self.child_count(node_id);

        match (num_children, layout.display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",
            (_, Display::Block) => "BLOCK",
            (_, Display::Flex) => match layout.flex_direction {
                FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
            },
            (_, Display::Grid) => "GRID",
            (_, _) => "UNKNOWN",
        }
    }

    fn get_final_layout(&self, node_id: LayoutNodeId) -> LayoutComputation {
        self.get(node_id).final_computation
    }
}
