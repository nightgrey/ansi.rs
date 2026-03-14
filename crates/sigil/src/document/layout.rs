use derive_more::{Deref, DerefMut};
use taffy::{TaffyError, TaffyResult, TraversePartialTree};
use tree::{RootTree, At, NodeRef, NodeRefMut, Node};
use geometry::Rect;
use grid::{Spatial, Within};

pub type Computed = taffy::TaffyTree<Rect>;

pub type Layout = taffy::Style;
pub type LayoutId = taffy::NodeId;
pub type ComputedLayout = taffy::Layout;

#[derive(Debug)]
pub struct LayoutTree<T = Rect> {
    inner: taffy::TaffyTree<T>,
}

impl<T> LayoutTree<T> {
    pub fn new() -> Self {
        Self { inner: taffy::TaffyTree::new() }
    }

    pub fn children(&self, id: LayoutId) -> TaffyResult<Vec<LayoutId>> {
        self.inner.children(id)
    }

    pub fn parent(&self, id: LayoutId) -> Option<LayoutId> {
        self.inner.parent(id)
    }

    pub fn get_layout(&self, id: LayoutId) -> TaffyResult<&Layout> {
        self.inner.style(id)
    }

    pub fn get_computed(&self, id: LayoutId) -> TaffyResult<&ComputedLayout> {
        self.inner.layout(id)
    }

    pub fn insert(&mut self, value: Layout) -> TaffyResult<LayoutId> {
        self.inner.new_leaf(value)
    }

    pub fn insert_at(&mut self, value: Layout, at: At<LayoutId>) -> LayoutId {
        self.try_insert_at(value, at).unwrap()
    }

    pub fn try_insert_at(&mut self, value: Layout, at: At<LayoutId>) -> TaffyResult<LayoutId> {
        let id = self.insert(value)?;
        if let Err(e) = match at {
            At::Detached => Ok(()),
            At::FirstChild(parent) => {
                self.insert_child_at_index(parent, 0, id)
            }

            At::Child(parent) | At::Child(parent) => {
                self.add_child(parent, id)
            }

            At::Before(target) => {
                if id == target {
                    Ok(())
                }  else {
                    let parent = self.parent(target);

                    match parent {
                        Some(parent) => {
                            match self.children(parent)?.iter().position(|&c| c == target) {
                                Some(index) => self.insert_child_at_index(parent, index, id),
                                None => return Err(TaffyError::InvalidChildNode(target)),
                            }
                        }
                        None => Err(TaffyError::InvalidParentNode(target)),
                    }
                }
            }

            At::After(target) => {
                if id == target {
                    Ok(())
                }  else {
                    let parent = self.parent(target);

                    match parent {
                        Some(parent) => {
                            let children = self.children(parent)?;
                            let len = children.len();
                            match children.iter().position(|&c| c == target) {
                                Some(index) => self.insert_child_at_index(parent, (index + 1).min(len), id),
                                None => return Err(TaffyError::InvalidChildNode(target)),
                            }
                        }
                        None => Err(TaffyError::InvalidParentNode(target)),
                    }
                }
            }
        } {
            let _ = self.remove(id);
            return Err(e);
        }

        Ok(id)
    }

    pub fn insert_at_with_children(
        &mut self,
        value: Layout,
        children: &[LayoutId],
        at: At<LayoutId>,
    ) -> LayoutId {
        self.try_insert_at_with_children(value, children, at).unwrap()
    }

    pub fn try_insert_at_with_children(
        &mut self,
        value: Layout,
        children: &[LayoutId],
        at: At<LayoutId>,
    ) -> TaffyResult<LayoutId> {
        let id = self.try_insert_at(value, at)?;
        for &child in children {
            self.try_move_to(child, At::Child(id))?;
        }
        Ok(id)
    }

    // --- Mutation ----------------------------------------------------------

    pub fn move_to(&mut self, id: LayoutId, to: At<LayoutId>) {
        self.try_move_to(id, to).unwrap()
    }

    pub fn try_move_to(&mut self, id: LayoutId, to: At<LayoutId>) -> TaffyResult<()> {
        match to {
            At::Detached => self.try_detach(id),

            At::FirstChild(parent) => {
                self.try_detach(id)?;
                self.insert_child_at_index(parent, 0, id)
            }

            At::Child(parent) | At::Child(parent) => {
                self.try_detach(id)?;
                self.add_child(parent, id)
            }

            At::Before(target) => {
                if id == target {
                    Ok(())
                }  else {
                    let parent = self.parent(target);

                    match parent {
                        Some(parent) => {
                            match self.children(parent)?.iter().position(|&c| c == target) {
                                Some(index) => {
                                    self.try_detach(id)?;
                                    self.insert_child_at_index(parent, index, id)
                                },
                                None => return Err(TaffyError::InvalidChildNode(target)),
                            }
                        }
                        None => Err(TaffyError::InvalidParentNode(target)),
                    }
                }
            }

            At::After(target) => {
                if id == target {
                    Ok(())
                }  else {
                    let parent = self.parent(target);

                    match parent {
                        Some(parent) => {
                            let children = self.children(parent)?;
                            let len = children.len();
                            match children.iter().position(|&c| c == target) {
                                Some(index) => {
                                    self.try_detach(id)?;
                                    self.insert_child_at_index(parent, (index + 1).min(len), id)
                                },
                                None => return Err(TaffyError::InvalidChildNode(target)),
                            }
                        }
                        None => Err(TaffyError::InvalidParentNode(target)),
                    }
                }
            }
        }
    }

    pub fn detach(&mut self, id: LayoutId) {
        self.try_detach(id).unwrap()
    }

    pub fn try_detach(&mut self, id: LayoutId) -> TaffyResult<()> {
        let layout = self.get_layout(id)?.clone();

        let children = self.children(id)?;

        let removed = self.inner.remove(id);

        match removed {
            Ok(id) => {

                self.

            }
            Err(e) => return Err(e),
        }
        if layout.parent.is_null() {
            return Ok(());
        }

        let prev = self.inner[id].previous_sibling;
        let next = self.inner[id].next_sibling;

        if prev.is_null() {
            self.inner[parent].first_child = next;
        } else {
            self.inner[prev].next_sibling = next;
        }

        if next.is_null() {
            self.inner[parent].last_child = prev;
        } else {
            self.inner[next].previous_sibling = prev;
        }

        let n = &mut self.inner[id];
        n.parent = LayoutId::null();
        n.previous_sibling = LayoutId::null();
        n.next_sibling = LayoutId::null();

        Ok(())
    }

    /// Remove node and its descendants.
    pub fn remove(&mut self, id: LayoutId) -> Option<V> {
        if !self.contains(id) {
            return None;
        }

        // Detach root of subtree from parent first.
        let _ = self.detach(id);

        // Remove descendants excluding `id`.
        let to_remove: Vec<_> = self
            .descendants(id)
            .filter(|&k| k != id)
            .collect();

        for k in to_remove {
            let _ = self.inner.remove(k);
        }

        self.inner.remove(id).map(|n| n.inner)
    }


    fn link_as_last_child(&mut self, parent: LayoutId, child: LayoutId) {
        let old_tail = self.inner[parent].last_child;

        if old_tail.is_null() {
            self.inner[parent].first_child = child;
        } else {
            self.inner[old_tail].next_sibling = child;
        }

        self.inner[parent].last_child = child;

        let n = &mut self.inner[child];
        n.parent = parent;
        n.previous_sibling = old_tail;
        n.next_sibling = LayoutId::null();
    }

    fn link_as_first_child(&mut self, parent: LayoutId, child: LayoutId) {
        let old_head = self.inner[parent].first_child;

        if old_head.is_null() {
            self.inner[parent].last_child = child;
        } else {
            self.inner[old_head].previous_sibling = child;
        }

        self.inner[parent].first_child = child;

        let n = &mut self.inner[child];
        n.parent = parent;
        n.previous_sibling = LayoutId::null();
        n.next_sibling = old_head;
    }

    fn link_before(&mut self, node: LayoutId, before: LayoutId, parent: LayoutId) {
        let prev = self.inner[before].previous_sibling;

        if prev.is_null() {
            self.inner[parent].first_child = node;
        } else {
            self.inner[prev].next_sibling = node;
        }

        self.inner[before].previous_sibling = node;

        let n = &mut self.inner[node];
        n.parent = parent;
        n.previous_sibling = prev;
        n.next_sibling = before;
    }

    fn link_after(&mut self, node: LayoutId, after: LayoutId, parent: LayoutId) {
        let next = self.inner[after].next_sibling;

        if next.is_null() {
            self.inner[parent].last_child = node;
        } else {
            self.inner[next].previous_sibling = node;
        }

        self.inner[after].next_sibling = node;

        let n = &mut self.inner[node];
        n.parent = parent;
        n.previous_sibling = after;
        n.next_sibling = next;
    }
}
