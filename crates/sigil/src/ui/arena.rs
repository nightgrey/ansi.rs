use std::num::NonZeroUsize;
use derive_more::{Deref, DerefMut};
use indextree::{Arena, Traverse, Ancestors, Predecessors, Children, Descendants, ReverseTraverse, PrecedingSiblings, NodeError, FollowingSiblings, ReverseChildren, };
use std::ops::{Deref, DerefMut};

pub type Node<T> = indextree::Node<T>;
pub type NodeId = indextree::NodeId;

#[repr(transparent)]
pub struct Tree<T>  {
    inner: Arena<T>
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Tree {
            inner: Arena::new()
        }
    }

    pub fn add(&mut self, node: T) -> NodeId {
        self.inner.new_node(node)
    }

    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.inner.get_mut(id)
    }

    pub fn get_id(&self, node: &Node<T>) -> Option<NodeId> {
        self.inner.get_node_id(node)
    }

    pub fn get_id_at(&self, index: NonZeroUsize) -> Option<NodeId> {
        self.inner.get_node_id_at(index)
    }

    pub fn get_ref(&self, id: NodeId) -> NodeRef<T> {
        NodeRef {
            id,
            arena: &self.inner
        }
    }

    pub fn get_ref_mut(&mut self, id: NodeId) -> NodeRefMut<T> {
        NodeRefMut {
            id,
            arena: &mut self.inner
        }
    }

    pub fn traverse(&self, id: NodeId) -> Traverse<'_, T> {
        id.traverse(&self.inner)
    }

    pub fn is_removed(&self, id: NodeId) -> bool {
        id.is_removed(&self.inner)
    }

    pub fn ancestors(&self, id: NodeId) -> Ancestors<'_, T> {
        id.ancestors(&self.inner)
    }

    pub fn predecessors(&self, id: NodeId) -> Predecessors<'_, T> {
        id.predecessors(&self.inner)
    }

    pub fn preceding_siblings(&self, id: NodeId) -> PrecedingSiblings<'_, T> {
        id.preceding_siblings(&self.inner)
    }

    pub fn following_siblings(&self, id: NodeId) -> FollowingSiblings<'_, T> {
        id.following_siblings(&self.inner)
    }

    pub fn children(&self, id: NodeId) -> Children<'_, T> {
        id.children(&self.inner)
    }

    pub fn descendants(&self, id: NodeId) -> Descendants<'_, T> {
        id.descendants(&self.inner)
    }

    pub fn reverse_traverse(&self, id: NodeId) -> ReverseTraverse<'_, T> {
        id.reverse_traverse(&self.inner)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn count(&mut self) {
        self.inner.count();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node<T>> {
        self.inner.iter()
    }

    pub fn iter_data_mut(&mut self) -> impl Iterator<Item = &mut Node<T>> {
        self.inner.iter_mut()
    }
}



#[derive(Deref, DerefMut)]
pub struct RootTree<T>  {
    root: NodeId,
    #[deref]
    #[deref_mut]
    inner: Tree<T>
}

impl<T> RootTree<T> {
    pub fn new(root: T) -> Self {
        let mut inner = Tree::new();
        let root = inner.add(root);

        Self {
            root,
            inner
        }
    }

    pub fn root(&self) -> NodeRef<T> {
        self.get_ref(self.root)
    }

    pub fn root_mut(&mut self) -> NodeRefMut<T> {
        let id = self.root;
        self.get_ref_mut(id)
    }
}


#[derive(Copy, Clone)]
pub struct NodeRef<'a, T> {
    pub id: NodeId,
    arena: &'a Arena<T>
}

impl<T> NodeRef<'_, T> {
    pub fn traverse(&self) -> Traverse<'_, T> {
        self.id.traverse(&self.arena)
    }

    pub fn is_removed(&self) -> bool {
        self.id.is_removed(&self.arena)
    }

    pub fn ancestors(&self) -> Ancestors<'_, T> {
        self.id.ancestors(&self.arena)
    }

    pub fn predecessors(&self) -> Predecessors<'_, T> {
        self.id.predecessors(&self.arena)
    }

    pub fn preceding_siblings(&self) -> PrecedingSiblings<'_, T> {
        self.id.preceding_siblings(&self.arena)
    }

    pub fn following_siblings(&self) -> FollowingSiblings<'_, T> {
        self.id.following_siblings(&self.arena)
    }

    pub fn children(&self) -> Children<'_, T> {
        self.id.children(&self.arena)
    }

    pub fn descendants(&self) -> Descendants<'_, T> {
        self.id.descendants(&self.arena)
    }

    pub fn reverse_traverse(&self) -> ReverseTraverse<'_, T> {
        self.id.reverse_traverse(&self.arena)
    }
}

impl<T> Deref for NodeRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.arena.get(self.id).unwrap().get()
    }
}

pub struct NodeRefMut<'a, T> {
    pub id: NodeId,
    arena: &'a mut Arena<T>
}

impl<T> NodeRefMut<'_, T> {
    pub fn traverse(&self) -> Traverse<'_, T> {
        self.id.traverse(&self.arena)
    }

    pub fn is_removed(&self) -> bool {
        self.id.is_removed(&self.arena)
    }

    pub fn ancestors(&self) -> Ancestors<'_, T> {
        self.id.ancestors(&self.arena)
    }

    pub fn predecessors(&self) -> Predecessors<'_, T> {
        self.id.predecessors(&self.arena)
    }

    pub fn preceding_siblings(&self) -> PrecedingSiblings<'_, T> {
        self.id.preceding_siblings(&self.arena)
    }

    pub fn following_siblings(&self) -> FollowingSiblings<'_, T> {
        self.id.following_siblings(&self.arena)
    }

    pub fn children(&self) -> Children<'_, T> {
        self.id.children(&self.arena)
    }

    pub fn descendants(&self) -> Descendants<'_, T> {
        self.id.descendants(&self.arena)
    }

    pub fn reverse_traverse(&self) -> ReverseTraverse<'_, T> {
        self.id.reverse_traverse(&self.arena)
    }
}

impl<T> Deref for NodeRefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.arena.get(self.id).unwrap().get()
    }
}

impl<T> DerefMut for NodeRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.arena.get_mut(self.id).unwrap().get_mut()
    }
}
