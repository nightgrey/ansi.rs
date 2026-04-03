use crate::{Dirty, LayoutContext, LayoutNode, Node, NodeId};
use crate::measure_node;
use crate::{Available, Dimension, Space, Style};
use geometry::{Point, Rect, Size};
use tree::{At, Secondary, Tree, id};

#[derive(Debug)]
pub struct Document<'a> {
    pub root: NodeId,
    pub nodes: Tree<NodeId, Node<'a>>,
    pub layouts: Secondary<NodeId, LayoutNode>,
}

impl<'a> Document<'a> {
    pub fn new() -> Self {
        let mut nodes = Tree::default();
        let mut layouts = Secondary::default();

        let root_id = nodes.insert(Node::Div());
        layouts.insert(root_id, LayoutNode::default());

        Self {
            root: root_id,
            nodes,
            layouts,
        }
    }
    
    /// Inserts a node as the last child of the root.
    pub fn insert(&mut self, node: Node<'a>) -> NodeId {
        self.insert_at(node, At::Child(self.root))
    }

    /// Inserts a node as the last child of the root.
    pub fn insert_with(&mut self, node: Node<'a>, f: impl FnOnce(&mut Node<'a>)) -> NodeId {
        let id = self.insert(node);
        f(&mut self.nodes[id]);
        id
    }
    
    /// Inserts a node at the given position.
    pub fn insert_at(&mut self, node: Node<'a>, at: At<NodeId>) -> NodeId{
        let id = self.nodes.insert_at(node, at);
        self.layouts.insert(id, LayoutNode::default());
        id
    }
    
    pub fn insert_at_with(&mut self, node: Node<'a>, at: At<NodeId>, f: impl FnOnce(&mut Node<'a>)) -> NodeId {
        let id = self.insert_at(node, at);
        f(&mut self.nodes[id]);
        id
    }
    
    pub fn move_to(&mut self, id: NodeId, at: At<NodeId>) {
        self.nodes.move_to(id, at);
        self.mark_dirty(id, Dirty::Style | Dirty::Measure | Dirty::Layout);
    }

    pub fn node(&self, id: NodeId) -> &Node<'a> {
        &self.nodes[id]
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node<'a> {
        self.mark_dirty(id, Dirty::Style | Dirty::Measure | Dirty::Layout);
        &mut self.nodes[id]
    }

    pub fn layout_node(&self, id: NodeId) -> &LayoutNode {
        &self.layouts[id]
    }

    pub fn layout_node_mut(&mut self, id: NodeId) -> &mut LayoutNode {
        self.layouts.get_mut(id).expect("missing layout node")
    }

    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.children(id)
    }

    pub fn mark_dirty(&mut self, id: NodeId, flags: Dirty) {
        if let Some(layout) = self.layouts.get_mut(id) {
            layout.dirty |= flags;
        }
    }

    pub fn set_style(&mut self, id: NodeId, style: Style) {
        self.nodes[id].style = style;
        self.mark_dirty(id, Dirty::Style | Dirty::Measure | Dirty::Layout);
    }

    pub fn compute_layout(&mut self, space: Space) {
        self.nodes[self.root].set_width(match space.width {
            Available::Definite(val) => Dimension::Length(val),
            Available::Min => Dimension::Auto,
            Available::Max => Dimension::MAX,
        });
        self.nodes[self.root].set_height(match space.height {
            Available::Definite(val) => Dimension::Length(val),
            Available::Min => Dimension::Auto,
            Available::Max => Dimension::MAX,
        });
        let mut context = LayoutContext::new(
            &mut self.nodes,
            &mut self.layouts,
            |known, available, id, style| measure_node(known, available, style),
        );

        context.compute_layout(
            self.root,
            space
        );

        self.clear_layout(self.root);
    }

    pub fn print_layout(&mut self) {
        LayoutContext::new(
            &mut self.nodes,
            &mut self.layouts,
            |known, available, id, style| measure_node(known, available, style),
        ).print_tree(self.root)
    }

    fn clear_layout(&mut self, id: NodeId) {
        let ids: Vec<NodeId> = std::iter::once(id)
            .chain(self.nodes.descendants(id))
            .collect();
        for id in ids {
            if let Some(layout) = self.layouts.get_mut(id) {
                layout.dirty.remove(Dirty::Layout | Dirty::Measure);
            }
        }
    }

    pub fn bounds(&self, id: NodeId) -> Rect {
        let layout = &self.layouts[id].final_layout;
        let x = layout.location.x.max(0.0) as usize;
        let y = layout.location.y.max(0.0) as usize;
        let w = layout.size.width.max(0.0) as usize;
        let h = layout.size.height.max(0.0) as usize;

        Rect {
            min: Point { x, y },
            max: Point { x: x + w, y: y + h },
        }
    }

    pub fn size(&self, id: NodeId) -> Size {
        let layout = &self.layouts[id].final_layout;
        Size {
            width: layout.size.width.max(0.0) as usize,
            height: layout.size.height.max(0.0) as usize,
        }
    }

    pub fn content_bounds(&self, id: NodeId) -> Rect {
        let layout = &self.layouts[id].final_layout;

        Rect::bounds(layout.content_box_x() as usize, layout.content_box_y() as usize, layout.content_box_width() as usize, layout.content_box_height() as usize)
    }
}

impl<'a> Default for Document<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle returned by [`Document::insert`] / [`Document::insert_at`] for
/// chaining `.with()` and `.with_children()` calls.
pub struct Insertion<'d, 'a> {
    doc: &'d mut Document<'a>,
    id: NodeId,
}

impl<'d, 'a> Insertion<'d, 'a> {
    pub fn new(doc: &'d mut Document<'a>, id: NodeId) -> Self {
        Self { doc, id }
    }
    /// Returns the id of the inserted node.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Mutates the inserted node via a callback.
    pub fn with(self, f: impl FnOnce(&mut Node<'a>)) -> Self {
        f(&mut self.doc.nodes[self.id]);
        self
    }

    /// Inserts children under the inserted node.
    pub fn with_children(self, children: impl IntoIterator<Item = Node<'a>>) -> Self {
        for child in children {
            self.doc.insert_at(child, At::Child(self.id));
        }
        self
    }
}

impl From<Insertion<'_, '_>> for NodeId {
    fn from(insertion: Insertion<'_, '_>) -> Self {
        insertion.id
    }
}