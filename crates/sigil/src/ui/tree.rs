use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use indextree::{Arena, };
use crate::{Node, NodeKind};

type ArenaNode<T> = indextree::Node<T>;
type ArenaId = indextree::NodeId;

trait Id: PartialEq + Eq + PartialOrd + Ord + Copy + Clone + Debug + Hash + From<ArenaId> + Into<ArenaId> {}

pub struct Tree<T>(Arena<T>);

#[derive(Debug)]
pub struct TreeNode<'a, T> {
    pub id: ArenaId,
    node: &'a ArenaNode<T>
}

impl<'a, T> TreeNode<'a, T> {
    pub fn parent(&self) -> Option<ArenaId> {
        self.node.parent()
    }

}

impl<T> Deref for TreeNode<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.node.get()
    }
}


#[derive(Debug)]
pub struct TreeNodeMut<'a, T> {
    pub id: ArenaId,
    node: &'a mut ArenaNode<T>
}

impl<T> Deref for TreeNodeMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.node.get()
    }
}

impl<T> DerefMut for TreeNodeMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node.get_mut()
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Tree(Arena::new(), PhantomData)
    }

    pub fn add(&mut self, node: T) -> ArenaId {
        self.0.new_node(node)
    }

    pub fn get(&self, id: ArenaId) -> Option<TreeNode<T>> {
        self.0.get(id).map(|node| TreeNode { id, node })
    }

    pub fn get_mut(&mut self, id: ArenaId) -> Option<TreeNodeMut<T>> {
        self.0.get_mut(id).map(|node| TreeNodeMut { id, node })
    }

    pub fn iter(&self) -> impl Iterator<Item = TreeNode<T>> {
    self.0.iter().map(|(id, node)| TreeNode { id, node })
}

#[test]
fn qwe() {
    let mut tree = Tree::new();

    let id = tree.add(Node {
        kind: NodeKind::Container,
    });

    dbg!(id);

    let retrieved = tree.get(id);
    if let Some(node) = retrieved {
    }
}