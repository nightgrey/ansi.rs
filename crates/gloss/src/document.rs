use crate::{Available, Layout, Length, Space};
use crate::{ComputedLayout, Dirty, Element, ElementId, LayoutContext};
use crate::{document, measure};
use geometry::Rect;
use tree::{At, Secondary, Tree};

#[derive(Debug, Clone)]
pub struct Document<'a> {
    pub root_id: ElementId,
    elements: Tree<ElementId, Element<'a>>,
    layouts: Secondary<ElementId, ComputedLayout>,
}

impl<'a> Document<'a> {
    pub fn new() -> Self {
        let mut elements = Tree::default();
        let mut layouts = Secondary::default();

        let root_id = elements.insert(Element::Div());
        layouts.insert(root_id, ComputedLayout::default());

        Self {
            root_id,
            elements,
            layouts,
        }
    }

    pub fn root(&self) -> &Element<'a> {
        &self.elements[self.root_id]
    }

    pub fn root_mut(&mut self) -> &mut Element<'a> {
        self.mark(self.root_id, Dirty::all());
        &mut self.elements[self.root_id]
    }

    pub fn element(&self, id: ElementId) -> &Element<'a> {
        &self.elements[id]
    }

    pub fn element_mut(&mut self, id: ElementId) -> &mut Element<'a> {
        self.mark(id, Dirty::all());
        &mut self.elements[id]
    }

    /// Inserts a node as the last child of the root.
    pub fn insert(&mut self, node: Element<'a>) -> ElementId {
        self.insert_at(node, At::Child(self.root_id))
    }

    /// Inserts a node as the last child of the root.
    pub fn insert_with(
        &mut self,
        node: Element<'a>,
        f: impl FnOnce(&mut Element<'a>),
    ) -> ElementId {
        let id = self.insert(node);
        f(&mut self.elements[id]);
        id
    }

    /// Inserts a node at the given position.
    pub fn insert_at(&mut self, node: Element<'a>, at: At<ElementId>) -> ElementId {
        let id = self.elements.insert_at(node, at);
        self.layouts.insert(id, ComputedLayout::default());
        id
    }

    pub fn insert_at_with(
        &mut self,
        node: Element<'a>,
        at: At<ElementId>,
        f: impl FnOnce(&mut Element<'a>),
    ) -> ElementId {
        let id = self.insert_at(node, at);
        f(&mut self.elements[id]);
        id
    }

    pub fn move_to(&mut self, id: ElementId, at: At<ElementId>) {
        self.elements.move_to(id, at);
        self.mark(id, Dirty::all());
    }

    pub fn children(&self, id: ElementId) -> impl Iterator<Item = ElementId> + '_ {
        self.elements.children(id)
    }

    pub fn mark(&mut self, id: ElementId, flags: Dirty) {
        self.computed_layout_mut(id).mark(flags);
    }

    pub fn unmark(&mut self, id: ElementId, flags: Dirty) {
        self.computed_layout_mut(id).unmark(flags);
    }

    pub fn computed_layout(&self, id: ElementId) -> &ComputedLayout {
        &self.layouts[id]
    }

    pub fn computed_layout_mut(&mut self, id: ElementId) -> &mut ComputedLayout {
        &mut self.layouts[id]
    }

    pub fn is_dirty(&self, id: ElementId) -> bool {
        self.computed_layout(id).is_dirty()
    }

    pub fn border_bounds(&self, id: ElementId) -> Rect {
        self.computed_layout(id).border_bounds()
    }

    pub fn content_bounds(&self, id: ElementId) -> Rect {
        self.computed_layout(id).content_bounds()
    }

    pub fn set_layout(&mut self, id: ElementId, style: Layout) {
        self.elements[id].layout = style;
        self.mark(id, Dirty::all());
    }

    pub fn compute_layout(&mut self, space: impl Into<Space>) {
        let mut context = LayoutContext::new(
            &mut self.elements,
            &mut self.layouts,
            |known, available, id, style| measure(known, available, style),
        );

        context.compute_layout(self.root_id, space.into());

        self.clear_layout(self.root_id);
    }

    pub fn print_layout(&mut self) {
        LayoutContext::new(
            &mut self.elements,
            &mut self.layouts,
            |known, available, id, style| measure(known, available, style),
        )
        .print_tree(self.root_id)
    }

    fn clear_layout(&mut self, id: ElementId) {
        for id in std::iter::once(id).chain(self.elements.descendants(id)) {
            if let Some(layout) = self.layouts.get_mut(id) {
                layout.unmark(Dirty::Computation | Dirty::Measure);
            }
        }
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layouts.clear();

        self.root_id = self.elements.insert(Element::Div());
        self.layouts.insert(self.root_id, ComputedLayout::default());
    }
}

impl<'a> Default for Document<'a> {
    fn default() -> Self {
        Self::new()
    }
}
