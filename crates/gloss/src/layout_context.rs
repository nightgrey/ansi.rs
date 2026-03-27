use crate::{Border, Display, FlexDirection, LayoutNode, Node, NodeId, NodeKind, Space, Style};
use taffy::TraversePartialTree;
use tree::{Bridge, Id, Secondary, Tree};
use compact_str::CompactString;

/// Layout context that holds the tree itself along with a reference to the context.
/// It implements taffy's layout traits and allows for layout computation.
#[derive(Debug)]
pub struct LayoutContext<'a, 'b, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    pub tree: &'a mut Tree<NodeId, Node<'b>>,
    pub layouts: &'a mut Secondary<NodeId, LayoutNode>,
    pub measure_function: MeasureFunction,
}

impl<'d, 'n, MeasureFunction> LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    pub fn new(
        tree: &'d mut Tree<NodeId, Node<'n>>,
        layouts: &'d mut Secondary<NodeId, LayoutNode>,
        measure_function: MeasureFunction,
    ) -> Self {
        Self {
            tree,
            layouts,
            measure_function,
        }
    }

    pub fn compute_layout(
        &mut self,
        id: NodeId,
        available_space: Space,
    ) {
        let key = id.into_layout_id();
        taffy::compute_root_layout(self, key, available_space.into());
        taffy::round_layout(self, key);
    }

    pub(crate) fn print_tree(&mut self, root: NodeId) {
        taffy::util::print_tree(self, root.into_layout_id());
    }

    fn element(&self, node_id: taffy::NodeId) -> &tree::Node<NodeId, Node<'n>> {
        &self.tree[NodeId::from_layout_id(node_id)]
    }

    fn layout_node(&self, node_id: taffy::NodeId) -> &LayoutNode {
        &self.layouts[NodeId::from_layout_id(node_id)]
    }

    fn layout_node_mut(&mut self, node_id: taffy::NodeId) -> &mut LayoutNode {
        &mut self.layouts[NodeId::from_layout_id(node_id)]
    }
}

pub struct LayoutChildren<'a, K: Id, V>(tree::iter::Children<'a, K, V>);

impl<K: Id, V> Iterator for LayoutChildren<'_, K, V> {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bridge::into_layout_id)
    }
}

impl<'d, 'n, MeasureFunction> taffy::TraversePartialTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    type ChildIter<'a>
        = LayoutChildren<'a, NodeId, Node<'a>>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        LayoutChildren(self.tree.children(NodeId::from_layout_id(parent_node_id)))
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.tree
            .children(NodeId::from_layout_id(parent_node_id))
            .count()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        let child_key = self
            .tree
            .children(NodeId::from_layout_id(parent_node_id))
            .nth(child_index)
            .unwrap();
        child_key.into_layout_id()
    }
}

impl<'d, 'n, MeasureFunction> taffy::TraverseTree for LayoutContext<'d, 'n, MeasureFunction> where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>
{
}

impl<'d, 'n, MeasureFunction> taffy::LayoutPartialTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type CustomIdent = CompactString;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        &self.tree[NodeId::from_layout_id(node_id)].style
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.layouts[NodeId::from_layout_id(node_id)].unrounded_layout = *layout;
    }

    #[inline(always)]
    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
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
            let node_key = NodeId::from_layout_id(node_id);
            let has_children = ctx.child_count(node_id) > 0;

            let node = &mut ctx.tree[node_key];
            let style = &node.style.clone();

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (style.display(), has_children) {
                (Display::None, _) => taffy::compute_hidden_layout(ctx, node_id),
                (Display::Block, true) => taffy::compute_block_layout(ctx, node_id, inputs),
                (Display::Flex, true) => taffy::compute_flexbox_layout(ctx, node_id, inputs),
                (_, false) | (Display::Inline, _) => taffy::compute_leaf_layout(
                    inputs,
                    &style,
                    |_, _| 0.0,
                    |known_dimensions, available_space| {
                        (ctx.measure_function)(
                            known_dimensions,
                            available_space,
                            node_key,
                            node,
                            &style,
                        )
                    },
                ),
            }
        })
    }
}

impl<'d, 'n, MeasureFunction> taffy::CacheTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.layouts[NodeId::from_layout_id(node_id)].cache.get(
            known_dimensions,
            available_space,
            run_mode,
        )
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.layouts[NodeId::from_layout_id(node_id)].cache.store(
            known_dimensions,
            available_space,
            run_mode,
            layout_output,
        );
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.layout_node_mut(node_id).cache.clear();
    }
}

impl<'d, 'n, MeasureFunction> taffy::LayoutFlexboxContainer
    for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        taffy::LayoutPartialTree::get_core_container_style(self, node_id)
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        taffy::LayoutPartialTree::get_core_container_style(self, child_node_id)
    }
}
//
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

impl<'d, 'n, MeasureFunction> taffy::LayoutBlockContainer for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_block_container_style(&self, node_id: taffy::NodeId) -> Self::BlockContainerStyle<'_> {
        taffy::LayoutPartialTree::get_core_container_style(self, node_id)
    }

    fn get_block_child_style(&self, child_node_id: taffy::NodeId) -> Self::BlockItemStyle<'_> {
        taffy::LayoutPartialTree::get_core_container_style(self, child_node_id)
    }
}

impl<'d, 'n, MeasureFunction> taffy::RoundTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.layouts[NodeId::from_layout_id(node_id)].unrounded_layout
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.layouts[NodeId::from_layout_id(node_id)].final_layout = *layout;
    }
}

impl<'d, 'n, MeasureFunction> taffy::PrintTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &mut Node,
        &Style,
    ) -> taffy::Size<f32>,
{
    fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
        let layout = taffy::LayoutPartialTree::get_core_container_style(self, node_id);
        let num_children = self.child_count(node_id);


        match (num_children, layout.display()) {
            (_, Display::None) => "None",
            (0, _) =>
                match &self.element(node_id).kind {
                    NodeKind::Div => "Node::Div",
                    NodeKind::Span(_) => "Node::Span"
                },
            (_, Display::Block) => "Block",
            (_, Display::Flex) => match layout.flex_direction() {
                FlexDirection::Row | FlexDirection::RowReverse => "Flex Row",
                FlexDirection::Column | FlexDirection::ColumnReverse => "Flex Col",
            },
            // (_, Display::Grid) => "GRID",
            (_, _) => "Unknown",
        }
    }

    fn get_final_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.layouts[NodeId::from_layout_id(node_id)].final_layout
    }
}
