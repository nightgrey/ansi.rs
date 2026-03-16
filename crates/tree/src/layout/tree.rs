use crate::{At, DefaultId, Error, Id, Secondary, Tree};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use derive_more::{Deref, DerefMut, Index, IndexMut};
use super::{prelude::*, Bridge, InternalLayoutId, LayoutNode};

/// A tree with integrated CSS layout computation powered by [`taffy`].
///
/// `LayoutTree` composes a [`Tree<K, LayoutNode>`](crate::Tree) with a
/// [`Secondary`] map for optional per-node `Context` values (used by custom
/// measure functions for leaf sizing).
///
/// It dereferences to [`Tree`](crate::Tree) so all standard tree operations
/// (navigation, iteration, etc.) are available directly.
///
/// # Layout modes
///
/// After building the tree, call [`compute_layout`](Self::compute_layout) (or
/// [`compute_layout_with_measure`](Self::compute_layout_with_measure) for leaf
/// measurement) and then read the results with [`layout`](Self::layout).
/// Supported display modes: [`Display::Flex`], [`Display::Grid`],
/// [`Display::Block`], and [`Display::None`].
///
/// # Rounding
///
/// Layout values are rounded to whole pixels by default. Disable with
/// [`disable_rounding`](Self::disable_rounding).
#[derive(Debug, Deref, DerefMut)]
pub struct LayoutTree<K: Id, Context = ()> {
    #[deref]
    #[deref_mut]
    inner: Tree<K, LayoutNode>,
    pub context: Secondary<K, Context>,
    config: LayoutConfig,
}

impl<K: Id, Context> LayoutTree<K, Context> {
    /// Creates a new layout tree with a default capacity of 16 nodes.
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Creates a new layout tree pre-allocated for `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Tree::with_capacity(capacity),
            context: Secondary::with_capacity(capacity),
            config: LayoutConfig::default(),
        }
    }

    // --- Insertion ---------------------------------------------------------

    /// Inserts a detached layout node with the given style.
    pub fn insert(&mut self, layout: Layout) -> K {
        self.inner.insert(LayoutNode::new(layout))
    }

    /// Inserts a layout node at the specified position.
    pub fn insert_at(&mut self, layout: Layout, at: At<K>) -> K {
        self.inner.insert_at(LayoutNode::new(layout), at)
    }

    /// Fallible version of [`insert_at`](Self::insert_at).
    pub fn try_insert_at(&mut self, layout: Layout, at: At<K>) -> Result<K, Error<K>> {
        self.inner.try_insert_at(LayoutNode::new(layout), at)
    }

    /// Inserts a detached node with a style and an associated context value.
    ///
    /// The context is available during layout via the measure function.
    pub fn insert_with_context(&mut self, layout: Layout, context: Context) -> K {
        let key = self.inner.insert(LayoutNode::new(layout));
        self.context.insert(key, context);
        key
    }

    /// Inserts a node with context at the specified position.
    pub fn insert_with_context_at(
        &mut self,
        layout: Layout,
        context: Context,
        at: At<K>,
    ) -> K {
        self.try_insert_with_context_at(layout, context, at).unwrap()
    }

    pub fn try_insert_with_context_at(
        &mut self,
        layout: Layout,
        context: Context,
        at: At<K>,
    ) -> Result<K, Error<K>> {
        let key = self.inner.try_insert_at(LayoutNode::new(layout), at)?;
        self.context.insert(key, context);
        Ok(key)
    }
    // --- Context access ----------------------------------------------------

    /// Returns a reference to the context associated with the given node.
    pub fn get_context(&self, id: K) -> Option<&Context> {
        self.context.get(id)
    }

    /// Returns a mutable reference to the context associated with the given node.
    pub fn get_context_mut(&mut self, id: K) -> Option<&mut Context> {
        self.context.get_mut(id)
    }

    /// Sets or removes the context for the given node.
    ///
    /// Pass `None` to remove the context entirely.
    pub fn set_context(&mut self, id: K, context: Option<Context>) {
        match context {
            Some(ctx) => { self.context.insert(id, ctx); }
            None => { let _ = self.context.remove(id); }
        }
    }

    pub fn contains_context(&self, id: K) -> bool {
        self.context.contains(id)
    }

    // --- Style access ------------------------------------------------------

    /// Returns a reference to the layout style of the given node.
    pub fn get_layout(&self, id: K) -> &Layout {
        &self.inner[id].layout
    }

    /// Returns a mutable reference to the layout style of the given node.
    pub fn get_layout_mut(&mut self, id: K) -> &mut Layout {
        &mut self.inner[id].layout
    }

    // --- Layout results ----------------------------------------------------

    /// Returns the computed layout for the given node.
    ///
    /// If rounding is enabled (the default), returns the rounded layout;
    /// otherwise returns the raw floating-point result.
    pub fn get_computation(&self, id: K) -> &Computation {
        if self.config.use_rounding {
            &self.inner[id].final_computation
        } else {
            &self.inner[id].unrounded_computation
        }
    }

    // --- Layout computation ------------------------------------------------

    /// Computes the layout for the subtree rooted at `root`.
    ///
    /// Leaf nodes without children are sized to zero. Use
    /// [`compute_layout_with_measure`](Self::compute_layout_with_measure) to
    /// provide a custom measure function for leaf sizing.
    pub fn compute_layout(&mut self, root: K, available_space: LayoutSize<AvailableSpace>) {
        self.compute_layout_with_measure(root, available_space, |_, _, _, _, _| LayoutSize::ZERO);
    }

    /// Computes layout with a custom measure function for leaf nodes.
    ///
    /// The measure function receives the known dimensions, available space,
    /// the node's key, its optional context, and its style, and should return
    /// the intrinsic size of the leaf.
    pub fn compute_layout_with_measure<MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>>(
        &mut self,
        root: K,
        available_space: LayoutSize<AvailableSpace>,
        measure: MeasureFunction,
    ) {
        let use_rounding = self.config.use_rounding;
        let root_id = root.into_layout();
        let mut ctx = LayoutContext {
            tree: self,
            measure_function: measure,
        };
        taffy::compute_root_layout(&mut ctx, root_id, available_space);
        if use_rounding {
            taffy::round_layout(&mut ctx, root_id);
        }
    }

    // --- Debug -------------------------------------------------------------

    /// Prints a debug representation of the layout tree to stdout via [`taffy`].
    pub fn print_tree(&mut self, root: K) {
        taffy::util::print_tree(&LayoutContext {
            tree: self,
            measure_function: |_, _, _, _, _| LayoutSize::ZERO,
        }, root.into_layout());
    }

    // --- Config ------------------------------------------------------------

    /// Enable rounding of layout values. Rounding is enabled by default.
    pub fn enable_rounding(&mut self) {
        self.config.use_rounding = true;
    }

    /// Disable rounding of layout values. Rounding is enabled by default.
    pub fn disable_rounding(&mut self) {
        self.config.use_rounding = false;
    }
}

impl<K: Id, Context> Default for LayoutTree<K, Context> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Id, Context> Index<K> for LayoutTree<K, Context> {
    type Output = LayoutNode;

    fn index(&self, index: K) -> &Self::Output {
        &self.inner[index]
    }
}

impl<K: Id, Context> IndexMut<K> for LayoutTree<K, Context> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
impl<K: Id, Context> Index<InternalLayoutId> for LayoutTree<K, Context> {
    type Output = LayoutNode;

    fn index(&self, index: InternalLayoutId) -> &Self::Output {
        &self.inner[K::from_layout(index)]
    }
}

impl<K: Id, Context> IndexMut<InternalLayoutId> for LayoutTree<K, Context> {
    fn index_mut(&mut self, index: InternalLayoutId) -> &mut Self::Output {
        &mut self.inner[K::from_layout(index)]
    }
}

#[derive(Debug, Clone, Copy)]
struct LayoutConfig {
    use_rounding: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { use_rounding: true }
    }
}

/// View over the [`LayoutTree`] that holds the tree itself along with a reference to the context
/// and implements LayoutTree. This allows the context to be stored outside of the [`LayoutTree`] struct
/// which makes the lifetimes of the context much more flexible.
#[derive(Debug, Index, IndexMut)]
struct LayoutContext<'t, K: Id, Context, MeasureFunction>
where
    MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>,
{
    #[index]
    #[index_mut]
    tree: &'t mut LayoutTree<K, Context>,
    measure_function: MeasureFunction,
}

impl<'t, K: Id, Context, MeasureFunction> LayoutContext<'t, K, Context, MeasureFunction>
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
    fn node(&self, layout_id: InternalLayoutId) -> &LayoutNode {
        &self.tree[layout_id]
    }

    #[inline(always)]
    fn node_mut(&mut self, layout_id: InternalLayoutId) -> &mut LayoutNode {
        &mut self.tree[layout_id]
    }
}

impl<K: Id, Context, MeasureFunction> TraverseLayoutPartialTree for LayoutContext<'_, K, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    type ChildIter<'a>
    = LayoutChildren<'a, K>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: InternalLayoutId) -> Self::ChildIter<'_> {
        LayoutChildren(self.tree.children(K::from_layout(parent_node_id)))
    }

    fn child_count(&self, parent_node_id: InternalLayoutId) -> usize {
        self.tree.children(K::from_layout(parent_node_id)).count()
    }

    fn get_child_id(&self, parent_node_id: InternalLayoutId, child_index: usize) -> InternalLayoutId {
        let a = &self.tree[parent_node_id];
        let child_key = self.tree.children(K::from_layout(parent_node_id)).nth(child_index).unwrap();
        child_key.into_layout()
    }
}

// --- TraverseTree (marker) -------------------------------------------------

impl<K: Id, Context, MeasureFunction> TraverseLayoutTree for LayoutContext<'_, K, Context, MeasureFunction> where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>
{
}

// --- LayoutPartialTree -----------------------------------------------------

impl<K: Id, Context, MeasureFunction> LayoutPartialTree for LayoutContext<'_, K, Context, MeasureFunction>
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
        &self.node(node_id).layout
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: InternalLayoutId, layout: &Computation) {
        self.node_mut(node_id).unrounded_computation = *layout;
    }

    #[inline(always)]
    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
    }

    fn compute_child_layout(&mut self, node_id: InternalLayoutId, inputs: LayoutInput) -> LayoutOutput {
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        taffy::compute_cached_layout(self, node_id, inputs, |tree, node_id, inputs| {
            let display_mode = tree.node(node_id).layout.display;
            let has_children = tree.child_count(node_id) > 0;

            match (display_mode, has_children) {
                (Display::None, _) => taffy::compute_hidden_layout(tree, node_id),
                (Display::Block, true) => taffy::compute_block_layout(tree, node_id, inputs),
                (Display::Flex, true) => taffy::compute_flexbox_layout(tree, node_id, inputs),
                (Display::Grid, true) => taffy::compute_grid_layout(tree, node_id, inputs),
                (_, false) => {
                    let key: K = K::from_layout(node_id);
                    let has_context = tree.tree.context.contains(key);
                    let style = &tree.tree.inner[key].layout;
                    let node_context = if has_context {
                        tree.tree.context.get_mut(key)
                    } else {
                        None
                    };
                    let measure_function = |known_dimensions, available_space| {
                        (tree.measure_function)(
                            known_dimensions,
                            available_space,
                            key,
                            node_context,
                            style,
                        )
                    };
                    taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, measure_function)
                }
                // Container with children but unknown display mode — treat as hidden
                (_, true) => taffy::compute_hidden_layout(tree, node_id),
            }
        })
    }
}

// --- CacheTree -------------------------------------------------------------

impl<K: Id, Context, MeasureFunction> CacheLayoutTree for LayoutContext<'_, K, Context, MeasureFunction>
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
        self.node(node_id)
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
        self.node_mut(node_id)
            .cache
            .store(known_dimensions, available_space, run_mode, layout_output);
    }

    fn cache_clear(&mut self, node_id: InternalLayoutId) {
        self.node_mut(node_id).cache.clear();
    }
}

// --- LayoutFlexboxContainer ------------------------------------------------

impl<K: Id, Context, MeasureFunction> LayoutFlexboxContainer for LayoutContext<'_, K, Context, MeasureFunction>
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
        &self.node(node_id).layout
    }

    fn get_flexbox_child_style(&self, child_node_id: InternalLayoutId) -> Self::FlexboxItemStyle<'_> {
        &self.node(child_node_id).layout
    }
}

// --- LayoutGridContainer ---------------------------------------------------

impl<K: Id, Context, MeasureFunction> LayoutGridContainer for LayoutContext<'_, K, Context, MeasureFunction>
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
        &self.node(node_id).layout
    }

    fn get_grid_child_style(&self, child_node_id: InternalLayoutId) -> Self::GridItemStyle<'_> {
        &self.node(child_node_id).layout
    }
}

// --- LayoutBlockContainer --------------------------------------------------

impl<K: Id, Context, MeasureFunction> LayoutBlockContainer for LayoutContext<'_, K, Context, MeasureFunction>
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
        &self.node(node_id).layout
    }

    fn get_block_child_style(&self, child_node_id: InternalLayoutId) -> Self::BlockItemStyle<'_> {
        &self.node(child_node_id).layout
    }
}

// --- RoundTree -------------------------------------------------------------

impl<K: Id, Context, MeasureFunction> RoundLayoutTree for LayoutContext<'_, K, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    fn get_unrounded_layout(&self, node_id: InternalLayoutId) -> Computation {
        self.node(node_id).unrounded_computation
    }

    fn set_final_layout(&mut self, node_id: InternalLayoutId, layout: &Computation) {
        self.node_mut(node_id).final_computation = *layout;
    }
}

// --- PrintTree -------------------------------------------------------------

impl<K: Id, Context, MeasureFunction> PrintLayoutTree for LayoutContext<'_, K, Context, MeasureFunction>
where
    MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, K, Option<&mut Context>, &Layout) -> LayoutSize<f32>,
{
    fn get_debug_label(&self, node_id: InternalLayoutId) -> &'static str {
        let node = self.node(node_id);
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
        self.node(node_id).final_computation
    }
}

pub struct LayoutChildren<'a, K: Id>(crate::iter::Children<'a, K, LayoutNode>);

impl<K: Id> Iterator for LayoutChildren<'_, K> {
    type Item = InternalLayoutId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bridge::into_layout)
    }
}


// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::DefaultId;
    use super::*;

    #[test]
    fn flex_layout_basic() {
        let mut tree: LayoutTree<DefaultId> = LayoutTree::new();

        // Container: flex column, 200x200
        let root = tree.insert(Layout {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: LayoutSize {
                width: Dimension::from_length(200.0),
                height: Dimension::from_length(200.0),
            },
            ..Default::default()
        });

        // Child A: 100x50
        let a = tree.insert_at(
            Layout {
                size: LayoutSize {
                    width: Dimension::from_length(100.0),
                    height: Dimension::from_length(50.0),
                },
                ..Default::default()
            },
            At::Child(root),
        );

        // Child B: 100x50
        let b = tree.insert_at(
            Layout {
                size: LayoutSize {
                    width: Dimension::from_length(100.0),
                    height: Dimension::from_length(50.0),
                },
                ..Default::default()
            },
            At::Child(root),
        );

        tree.compute_layout(root, LayoutSize::MAX_CONTENT);

        tree.children(root).for_each(|child| {
            let layout = tree.get_computation(child);
            assert!(layout.size.width > 0.0);
            assert!(layout.size.height > 0.0);
        });

        tree.compute_layout_with_measure(root, LayoutSize::MAX_CONTENT, |size, a, b, c, d| LayoutSize::ZERO);

        let root_layout = tree.get_computation(root);
        assert_eq!(root_layout.size.width, 200.0);
        assert_eq!(root_layout.size.height, 200.0);

        let a_layout = tree.get_computation(a);
        assert_eq!(a_layout.size.width, 100.0);
        assert_eq!(a_layout.size.height, 50.0);
        assert_eq!(a_layout.location.x, 0.0);
        assert_eq!(a_layout.location.y, 0.0);

        let b_layout = tree.get_computation(b);
        assert_eq!(b_layout.size.width, 100.0);
        assert_eq!(b_layout.size.height, 50.0);
        assert_eq!(b_layout.location.x, 0.0);
        assert_eq!(b_layout.location.y, 50.0);
    }

    #[test]
    fn flex_row_layout() {
        let mut tree: LayoutTree<DefaultId> = LayoutTree::new();

        let root = tree.insert(Layout {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: LayoutSize {
                width: Dimension::from_length(300.0),
                height: Dimension::from_length(100.0),
            },
            ..Default::default()
        });

        let a = tree.insert_at(
            Layout {
                size: LayoutSize {
                    width: Dimension::from_length(100.0),
                    height: Dimension::from_length(100.0),
                },
                ..Default::default()
            },
            At::Child(root),
        );

        let b = tree.insert_at(
            Layout {
                size: LayoutSize {
                    width: Dimension::from_length(100.0),
                    height: Dimension::from_length(100.0),
                },
                ..Default::default()
            },
            At::Child(root),
        );

        tree.compute_layout(root, LayoutSize::MAX_CONTENT);

        let a_layout = tree.get_computation(a);
        assert_eq!(a_layout.location.x, 0.0);
        assert_eq!(a_layout.location.y, 0.0);

        let b_layout = tree.get_computation(b);
        assert_eq!(b_layout.location.x, 100.0);
        assert_eq!(b_layout.location.y, 0.0);
    }

    #[test]
    fn layout_with_measure_function() {
        let mut tree: LayoutTree<DefaultId, LayoutSize<f32>> = LayoutTree::new();

        let root = tree.insert(Layout {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        });

        let leaf = tree.insert_with_context_at(
            Layout::default(),
            LayoutSize { width: 50.0, height: 25.0 },
            At::Child(root),
        );

        tree.compute_layout_with_measure(
            root,
            LayoutSize::MAX_CONTENT,
            |known_dimensions, _available_space, _key, context, _style| {
                let size = context.unwrap();
                LayoutSize {
                    width: known_dimensions.width.unwrap_or(size.width),
                    height: known_dimensions.height.unwrap_or(size.height),
                }
            },
        );

        let leaf_layout = tree.get_computation(leaf);
        assert_eq!(leaf_layout.size.width, 50.0);
        assert_eq!(leaf_layout.size.height, 25.0);
    }

    #[test]
    fn deref_to_tree() {
        let mut tree: LayoutTree<DefaultId> = LayoutTree::new();
        let root = tree.insert(Layout::default());
        let child = tree.insert_at(Layout::default(), At::Child(root));

        // These use Tree methods via Deref
        assert!(tree.contains(root));
        assert_eq!(tree.parent(child), Some(root));
        assert_eq!(tree.first_child(root), Some(child));
        assert!(tree.is_leaf(child));
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn context_operations() {
        let mut tree: LayoutTree<DefaultId, String> = LayoutTree::new();
        let node = tree.insert_with_context(Layout::default(), "hello".into());

        assert_eq!(tree.get_context(node), Some(&"hello".to_string()));

        *tree.get_context_mut(node).unwrap() = "world".into();
        assert_eq!(tree.get_context(node), Some(&"world".to_string()));

        tree.set_context(node, None);
        assert_eq!(tree.get_context(node), None);

        tree.set_context(node, Some("back".into()));
        assert_eq!(tree.get_context(node), Some(&"back".to_string()));
    }
}
