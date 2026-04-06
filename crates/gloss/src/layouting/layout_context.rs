use crate::{Display, FlexDirection, LayoutNode, Node, NodeId, NodeKind, Space, Style};
use tree::{Id, Secondary, Tree};
use compact_str::CompactString;
use slotmap::Key;
use taffy::{BlockContext, LayoutInput, LayoutOutput, TraversePartialTree};


trait GetTaffyStyle<'a, S: taffy::CoreStyle> {
    fn get_taffy_style(&self, node_id: taffy::NodeId) -> &'a S;
}

/// Layout context that holds the tree itself along with a reference to the context.
/// It implements taffy's layout traits and allows for layout computation.
#[derive(Debug)]
pub struct LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
    ) -> taffy::Size<f32>,
{
    pub tree: &'d mut Tree<NodeId, Node<'n>>,
    pub layouts: &'d mut Secondary<NodeId, LayoutNode>,
    pub measure_function: MeasureFunction,
}

impl<'d, 'n, MeasureFunction> LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
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
        let taffy_id = Self::into_taffy_id(id);
        taffy::compute_root_layout(self, taffy_id, available_space.into());
        taffy::round_layout(self, taffy_id);
    }

    pub(crate) fn print_tree(&mut self, root: NodeId) {
        taffy::util::print_tree(self, Self::into_taffy_id(root));
    }

    fn style(&self, id: taffy::NodeId) -> &Style {
        &self.tree[Self::from_taffy_id(id)].style
    }


    #[inline]
    fn node(&self, node_id: taffy::NodeId) -> &tree::Node<NodeId, Node<'n>> {
        &self.tree[Self::from_taffy_id(node_id)]
    }

    #[inline]
    fn node_mut(&mut self, node_id: taffy::NodeId) -> &mut tree::Node<NodeId, Node<'n>> {
        &mut self.tree[Self::from_taffy_id(node_id)]
    }

    #[inline]
    fn layout(&self, node_id: taffy::NodeId) -> &LayoutNode {
        &self.layouts[Self::from_taffy_id(node_id)]
    }

    #[inline]
    fn layout_mut(&mut self, node_id: taffy::NodeId) -> &mut LayoutNode {
        &mut self.layouts[Self::from_taffy_id(node_id)]
    }

    #[inline]
    fn into_taffy_id<K: Id>(id: K) -> taffy::NodeId {
        taffy::NodeId::new(id.data().as_ffi())
    }

    #[inline]
    fn from_taffy_id<K: Id>(id: taffy::NodeId) -> K {
        slotmap::KeyData::from_ffi(u64::from(id)).into()
    }
}

pub struct LayoutChildren<'a>(tree::iter::Children<'a, NodeId, Node<'a>>);

impl<'a> Iterator for LayoutChildren<'a> {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|id| taffy::NodeId::new(id.data().as_ffi()))
    }
}

impl<'d, 'n, MeasureFunction> taffy::TraversePartialTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
    ) -> taffy::Size<f32>,
{
    type ChildIter<'a>
        = LayoutChildren<'a>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        LayoutChildren(self.tree.children(Self::from_taffy_id(parent_node_id)))
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.tree
            .children(Self::from_taffy_id(parent_node_id))
            .count()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        Self::into_taffy_id(
            self.tree
                .children(Self::from_taffy_id(parent_node_id))
                .nth(child_index)
                .unwrap(),
        )
    }
}

impl<'d, 'n, MeasureFunction> taffy::TraverseTree for LayoutContext<'d, 'n, MeasureFunction> where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
    ) -> taffy::Size<f32>
{
}

impl<'d, 'n, MeasureFunction> taffy::LayoutPartialTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
    ) -> taffy::Size<f32>,
{
    type CoreContainerStyle<'a>
        = &'a Style
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
        self.layout_mut(node_id).unrounded_layout = *layout;
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
            let node_key = Self::from_taffy_id(node_id);
            let has_children = ctx.child_count(node_id) > 0;

            let node = &ctx.tree[node_key];
            let style = &node.style;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (style.display, has_children) {
                (Display::None, _) => taffy::compute_hidden_layout(ctx, node_id),
                (Display::Block, true) => taffy::compute_block_layout(ctx, node_id, inputs, None),
                (Display::Flex, true) => taffy::compute_flexbox_layout(ctx, node_id, inputs),
                (_, false) | (Display::Inline, _) => taffy::compute_leaf_layout(
                    inputs,
                    style,
                    |_, _| 0.0,
                    |known_dimensions, available_space| {
                        (ctx.measure_function)(
                            known_dimensions,
                            available_space,
                            node_key,
                            node,
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
        &Node<'n>,
    ) -> taffy::Size<f32>,
{
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        input: &taffy::LayoutInput,
    ) -> Option<taffy::LayoutOutput> {
        self.layout(node_id).cache.get(input)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        input: &taffy::LayoutInput,
        layout_output: taffy::LayoutOutput,
    ) {
        self.layout_mut(node_id).cache.store(input, layout_output);
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.layout_mut(node_id).cache.clear();
    }
}

impl<'d, 'n, MeasureFunction> taffy::LayoutFlexboxContainer
    for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
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

impl<'d, 'n, MeasureFunction> taffy::LayoutBlockContainer for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
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

impl<'d, 'n, MeasureFunction> taffy::RoundTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
    ) -> taffy::Size<f32>,
{
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.layout(node_id).unrounded_layout
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.layout_mut(node_id).final_layout = *layout;
    }
}

impl<'d, 'n, MeasureFunction> taffy::PrintTree for LayoutContext<'d, 'n, MeasureFunction>
where
    MeasureFunction: FnMut(
        taffy::Size<Option<f32>>,
        taffy::Size<taffy::AvailableSpace>,
        NodeId,
        &Node<'n>,
    ) -> taffy::Size<f32>,
{
    fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
        let style = self.style(node_id);

        match (style.display, &self.node(node_id).kind) {
            (Display::Inline, NodeKind::Span(_)) => "Node::Span",
            (Display::Block, NodeKind::Span(_)) => "Node::Span [Block]",
            (Display::Flex, NodeKind::Span(_)) => "Node::Span [Flex]",
            (Display::None, NodeKind::Span(_)) => "Node::Span [None]",

            (Display::Inline, NodeKind::Div) => "Node::Div [Inline]",
            (Display::Block, NodeKind::Div) => "Node::Div [Block]",
            (Display::Flex, NodeKind::Div) => match style.flex_direction {
                FlexDirection::Column => "Node::Div [Flex::Column]",
                FlexDirection::Row => "Node::Div [Flex::Row]",
                FlexDirection::RowReverse => "Node::Div [Flex::RowReverse]",
                FlexDirection::ColumnReverse => "Node::Div [Flex::ColumnReverse]",
            },
            (Display::None, NodeKind::Div) => "Node::Div [None]",
        }
    }

    fn get_final_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.layout(node_id).final_layout
    }
}
