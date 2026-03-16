use crate::{Id, LayoutNode, Secondary, Tree};
use crate::{prelude::*, Bridge, InternalLayoutId};

pub trait LayoutTree<K: Id, V, Context = ()> {
    fn as_context<MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>>(
        &mut self,
        measure: MeasureFunction,
    ) -> LayoutContext<'_, K, V, Context, MeasureFunction>;

    fn use_rounding(&self) -> bool;

    fn compute_layout(&mut self, id: K, available_space: LayoutSize<AvailableSpace>) {
        let mut context = self.as_context(|_, _, _, _, _| LayoutSize::ZERO);
        taffy::compute_root_layout(&mut context, id.into_layout(), available_space);
    }

    fn compute_layout_with_measure<MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>>(
        &mut self,
        id: K,
        available_space: LayoutSize<AvailableSpace>,
        measure: MeasureFunction,
    ) {
        let key = id.into_layout();
        let use_rounding = self.use_rounding();
        let mut context = self.as_context(measure);
        taffy::compute_root_layout(&mut context, key, available_space);
        if use_rounding {
            taffy::round_layout(&mut context, key);
        }
    }

    fn print_tree(&mut self, root: K) {
        taffy::util::print_tree(&self.as_context(|_, _, _, _, _| LayoutSize::ZERO), root.into_layout());
    }
}


/// Layout context that holds the tree itself along with a reference to the context.
/// It implements taffy's layout traits and allows for layout computation.
#[derive(Debug)]
pub struct LayoutContext<'t, K: Id, V, Context, MeasureFunction: FnMut(
    LayoutSize<Option<f32>>,
    LayoutSize<AvailableSpace>,
    K,
    Option<&mut Context>,
    &Layout,
) -> LayoutSize<f32>>
{
    pub tree: &'t mut Tree<K, V>,
    pub layouts: &'t mut Secondary<K, LayoutNode>,
    pub contexts: &'t mut Secondary<K, Context>,
    pub measure_function: MeasureFunction,
}

impl<'t, K: Id, V, Context, MeasureFunction> LayoutContext<'t, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>,
{
    #[inline(always)]
    fn key(layout_id: InternalLayoutId) -> K {
        K::from_layout(layout_id)
    }

    #[inline(always)]
    fn id(key: K) -> InternalLayoutId {
        K::into_layout(key)
    }

    #[inline(always)]
    fn get(&self, layout_id: InternalLayoutId) -> &LayoutNode {
        &self.layouts[Self::key(layout_id)]
    }

    #[inline(always)]
    fn get_mut(&mut self, layout_id: InternalLayoutId) -> &mut LayoutNode {
        &mut self.layouts[Self::key(layout_id)]
    }
}

pub struct LayoutChildren<'a, K: Id, V>(crate::iter::Children<'a, K, V>);

impl<K: Id, V> Iterator for LayoutChildren<'_, K, V> {
    type Item = InternalLayoutId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bridge::into_layout)
    }
}

impl<K: Id, V, Context, MeasureFunction> TraverseLayoutPartialTree for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    type ChildIter<'a>
    = LayoutChildren<'a, K, V>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: InternalLayoutId) -> Self::ChildIter<'_> {
        LayoutChildren(self.tree.children(K::from_layout(parent_node_id)))
    }

    fn child_count(&self, parent_node_id: InternalLayoutId) -> usize {
        self.tree.children(K::from_layout(parent_node_id)).count()
    }

    fn get_child_id(&self, parent_node_id: InternalLayoutId, child_index: usize) -> InternalLayoutId {
        let child_key = self.tree.children(K::from_layout(parent_node_id)).nth(child_index).unwrap();
        child_key.into_layout()
    }
}

impl<K: Id, V, Context, MeasureFunction> TraverseLayoutTree for LayoutContext<'_, K, V, Context, MeasureFunction> where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>
{
}

impl<K: Id, V, Context, MeasureFunction> LayoutPartialTree for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    type CoreContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type CustomIdent = String;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: InternalLayoutId) -> Self::CoreContainerStyle<'_> {
        &self.get(node_id).layout
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: InternalLayoutId, layout: &Computation) {
        self.get_mut(node_id).unrounded_computation = *layout;
    }

    #[inline(always)]
    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
    }

    fn compute_child_layout(&mut self, node_id: InternalLayoutId, inputs: LayoutInput) -> LayoutOutput {
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        taffy::compute_cached_layout(self, node_id, inputs, |ctx, node_id, inputs| {
            let node_key = K::from_layout(node_id);
            let display_mode = ctx.layouts[node_key].layout.display;
            let has_children = ctx.child_count(node_id) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => taffy::compute_hidden_layout(ctx, node_id),
                (Display::Block, true) => taffy::compute_block_layout(ctx, node_id, inputs),
                (Display::Flex, true) => taffy::compute_flexbox_layout(ctx, node_id, inputs),
                (Display::Grid, true) => taffy::compute_grid_layout(ctx, node_id, inputs),
                (_, false) => {
                    let layout = &ctx.layouts[node_key].layout;
                    let node_context = ctx.contexts.get_mut(node_key);
                    let measure_function = |known_dimensions, available_space| {
                        (ctx.measure_function)(known_dimensions, available_space, node_key, node_context, layout)
                    };
                    // TODO: implement calc() in high-level API
                    taffy::compute_leaf_layout(inputs, layout, |_, _| 0.0, measure_function)
                }
            }
        })
    }
}

impl<K: Id, V, Context, MeasureFunction> CacheLayoutTree for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    fn cache_get(
        &self,
        node_id: InternalLayoutId,
        known_dimensions: LayoutSize<Option<f32>>,
        available_space: LayoutSize<AvailableSpace>,
        run_mode: RunMode,
    ) -> Option<LayoutOutput> {
        self.get(node_id)
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: InternalLayoutId,
        known_dimensions: LayoutSize<Option<f32>>,
        available_space: LayoutSize<AvailableSpace>,
        run_mode: RunMode,
        layout_output: LayoutOutput,
    ) {
        self.get_mut(node_id)
            .cache
            .store(known_dimensions, available_space, run_mode, layout_output);
    }

    fn cache_clear(&mut self, node_id: InternalLayoutId) {
        self.get_mut(node_id).cache.clear();
    }
}

impl<K: Id, V, Context, MeasureFunction> LayoutFlexboxContainer for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    type FlexboxContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
    = &'a Layout
    where
        Self: 'a;

    fn get_flexbox_container_style(&self, node_id: InternalLayoutId) -> Self::FlexboxContainerStyle<'_> {
        &self.get(node_id).layout
    }

    fn get_flexbox_child_style(&self, child_node_id: InternalLayoutId) -> Self::FlexboxItemStyle<'_> {
        &self.get(child_node_id).layout
    }
}

impl<K: Id, V, Context, MeasureFunction> LayoutGridContainer for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    type GridContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type GridItemStyle<'a>
    = &'a Layout
    where
        Self: 'a;

    fn get_grid_container_style(&self, node_id: InternalLayoutId) -> Self::GridContainerStyle<'_> {
        &self.get(node_id).layout
    }

    fn get_grid_child_style(&self, child_node_id: InternalLayoutId) -> Self::GridItemStyle<'_> {
        &self.get(child_node_id).layout
    }
}

impl<K: Id, V, Context, MeasureFunction> LayoutBlockContainer for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    type BlockContainerStyle<'a>
    = &'a Layout
    where
        Self: 'a;
    type BlockItemStyle<'a>
    = &'a Layout
    where
        Self: 'a;

    fn get_block_container_style(&self, node_id: InternalLayoutId) -> Self::BlockContainerStyle<'_> {
        &self.get(node_id).layout
    }

    fn get_block_child_style(&self, child_node_id: InternalLayoutId) -> Self::BlockItemStyle<'_> {
        &self.get(child_node_id).layout
    }
}

impl<K: Id, V, Context, MeasureFunction> RoundLayoutTree for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    fn get_unrounded_layout(&self, node_id: InternalLayoutId) -> Computation {
        self.get(node_id).unrounded_computation
    }

    fn set_final_layout(&mut self, node_id: InternalLayoutId, layout: &Computation) {
        self.get_mut(node_id).final_computation = *layout;
    }
}

impl<K: Id, V, Context, MeasureFunction> PrintLayoutTree for LayoutContext<'_, K, V, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    fn get_debug_label(&self, node_id: InternalLayoutId) -> &'static str {
        let node = self.get(node_id);
        let num_children = self.child_count(node_id);

        match (num_children, node.layout.display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",
            (_, Display::Block) => "BLOCK",
            (_, Display::Flex) => match node.layout.flex_direction {
                FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
            },
            (_, Display::Grid) => "GRID",
            (_, _) => "UNKNOWN",
        }
    }

    fn get_final_layout(&self, node_id: InternalLayoutId) -> Computation {
        self.get(node_id).final_computation
    }
}
