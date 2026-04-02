use crate::{Dirty,  LayoutContext, LayoutNode, Node, NodeId};
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

    pub fn insert(&mut self, node: Node<'a>) -> NodeId {
        self.insert_at(node, At::Child(self.root))
    }

    pub fn insert_with_children(&mut self, node: Node<'a>, children: impl IntoIterator<Item = Node<'a>>) -> NodeId {
        let id = self.insert(node);
        for child in children {
            self.insert_at(child, At::Child(id));
        }
        id
    }

    pub fn insert_at(&mut self, node: Node<'a>, at: At<NodeId>) -> NodeId {
        let id = self.nodes.insert_at(node, at);
        self.layouts.insert(id, LayoutNode::default());
        id
    }
    
    pub fn insert_with_children_at(&mut self, node: Node<'a>, children: impl IntoIterator<Item = Node<'a>>, at: At<NodeId>) -> NodeId {
        let id = self.insert_at(node, at);
        for child in children {
            self.insert_at(child, At::Child(id));
        }
        id
    }

    pub fn insert_with(&mut self, node: Node<'a>, with: impl FnOnce(&mut Node<'a>)) -> NodeId {
        let id = self.insert(node);
        with(&mut self.nodes[id]);
        id
    }
    
    pub fn insert_at_with(
        &mut self,
        node: Node<'a>,
        at: At<NodeId>,
        with: impl FnOnce(&mut Node<'a>),
    ) -> NodeId {
        let id = self.insert_at(node, at);
        with(&mut self.nodes[id]);
        id
    }

    pub fn insert_with_children_at_with(&mut self, node: Node<'a>, children: impl IntoIterator<Item = Node<'a>>, at: At<NodeId>, with: impl FnOnce(&mut Node<'a>)) -> NodeId {
        let id = self.insert_at(node, at);
        for child in children {
            self.insert_at(child, At::Child(id));
        }
        with(&mut self.nodes[id]);
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
        // Adapt this to your tree API.
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
        self.nodes[self.root].style.width = match space.width {
            Available::Definite(val) => Dimension::Length(val),
            Available::Min => Dimension::Auto,
            Available::Max => Dimension::MAX,
        };
        self.nodes[self.root].style.height = match space.height {
            Available::Definite(val) => Dimension::Length(val),
            Available::Min => Dimension::Auto,
            Available::Max => Dimension::MAX,
        };
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
        if let Some(layout) = self.layouts.get_mut(id) {
            layout.dirty.remove(Dirty::Layout | Dirty::Measure);
        }

        let children: Vec<_> = self.children(id).collect();
        for child in children {
            self.clear_layout(child);
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
