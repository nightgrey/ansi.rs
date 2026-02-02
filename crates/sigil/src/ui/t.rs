use derive_more::{Index, IndexMut};
use slotmap::{Key, SecondaryMap, SlotMap};

#[derive(Index, IndexMut)]
pub struct Tree<K: Key, V> {
    #[index]
    #[index_mut]
    nodes: SlotMap<K, V>,

    /// The list of child node IDs for each parent (in order).
    children: SecondaryMap<K, Vec<K>>,

    /// The parent ID for each node, or `None` if it is a root or detached.
    parents: SecondaryMap<K, Option<K>>,

    /// The previous sibling for each node, or None if it is the first child.
    previous_sibling: SecondaryMap<K, Option<K>>,

    /// The next sibling for each node, or None if it is the last child.
    next_sibling: SecondaryMap<K, Option<K>>,
}

impl<K: Key, V> Tree<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Tree {
            nodes: SlotMap::with_capacity_and_key(capacity),
            parents: SecondaryMap::with_capacity(capacity),
            children: SecondaryMap::with_capacity(capacity),
            previous_sibling: SecondaryMap::with_capacity(capacity),
            next_sibling: SecondaryMap::with_capacity(capacity),
        }
    }

    pub fn get(&self, id: K) -> Option<&V> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
        self.nodes.get_mut(id)
    }

    /// Insert a new detached node (no parent, no siblings).
    pub fn insert(&mut self, value: V) -> K {
        let id = self.nodes.insert(value);

        self.parents.insert(id, None);
        self.previous_sibling.insert(id, None);
        self.next_sibling.insert(id, None);
        self.children.insert(id, Vec::new());

        id
    }

    /// Insert a node and immediately attach the given children to it.
    pub fn insert_with_children(&mut self, value: V, children: &[K]) -> K {
        let id = self.nodes.insert(value);

        self.parents.insert(id, None);
        self.previous_sibling.insert(id, None);
        self.next_sibling.insert(id, None);

        // Build the children list and update sibling links
        let mut children_vec = Vec::with_capacity(children.len());

        for (index, &child_id) in children.iter().enumerate() {
            // Update parent
            self.parents.insert(child_id, Some(id));

            // Update sibling links
            let prev = if index > 0 {
                Some(children[index - 1])
            } else {
                None
            };

            let next = children.get(index + 1).copied();

            self.previous_sibling.insert(child_id, prev);
            self.next_sibling.insert(child_id, next);

            children_vec.push(child_id);
        }

        self.children.insert(id, children_vec);

        id
    }

    /// Recursively removes a node and all its descendants from the tree.
    pub fn remove(&mut self, id: K) {
        // Detach from parent first
        if let Some(Some(parent_id)) = self.parents.get(id).copied() {
            self.detach_child(parent_id, id);
        }

        // Get children (remove from map to avoid borrowing issues)
        let children = self.children.remove(id).unwrap_or_default();

        // Recursively remove all descendants
        for child_id in children {
            self.remove(child_id);
        }

        // Remove the node itself from all maps
        self.nodes.remove(id);
        self.parents.remove(id);
        self.previous_sibling.remove(id);
        self.next_sibling.remove(id);
    }

    /// Append a child to the end of the parent's children list.
    pub fn append_child(&mut self, parent_id: K, child_id: K) {
        // Detach from current parent if any
        if let Some(Some(old_parent)) = self.parents.get(child_id).copied() {
            self.detach_child(old_parent, child_id);
        }

        let children = self
            .children
            .entry(parent_id)
            .unwrap()
            .or_insert_with(Vec::new);

        // Get the current last child
        let previous = children.last().copied();

        // Update sibling links
        self.previous_sibling.insert(child_id, previous);
        self.next_sibling.insert(child_id, None);

        // Update the old last child's next pointer
        if let Some(prev_id) = previous {
            self.next_sibling.insert(prev_id, Some(child_id));
        }

        // Add to children list
        children.push(child_id);

        // Update parent
        self.parents.insert(child_id, Some(parent_id));
    }

    /// Prepend a child to the beginning of the parent's children list.
    pub fn prepend_child(&mut self, parent_id: K, child_id: K) {
        // Detach from current parent if any
        if let Some(Some(old_parent)) = self.parents.get(child_id).copied() {
            self.detach_child(old_parent, child_id);
        }

        let children = self
            .children
            .entry(parent_id)
            .unwrap()
            .or_insert_with(Vec::new);

        // Get the current first child
        let next = children.first().copied();

        // Update sibling links
        self.previous_sibling.insert(child_id, None);
        self.next_sibling.insert(child_id, next);

        // Update the old first child's previous pointer
        if let Some(next_id) = next {
            self.previous_sibling.insert(next_id, Some(child_id));
        }

        // Add to children list
        children.insert(0, child_id);

        // Update parent
        self.parents.insert(child_id, Some(parent_id));
    }

    /// Insert a child before a specific sibling.
    pub fn insert_child_before(&mut self, parent_id: K, child_id: K, before_id: K) {
        // Detach from current parent if any
        if let Some(Some(old_parent)) = self.parents.get(child_id).copied() {
            self.detach_child(old_parent, child_id);
        }

        if let Some(children) = self.children.get_mut(parent_id) {
            if let Some(index) = children.iter().position(|&id| id == before_id) {
                let previous = if index > 0 {
                    Some(children[index - 1])
                } else {
                    None
                };

                // Update the new child's pointers
                self.previous_sibling.insert(child_id, previous);
                self.next_sibling.insert(child_id, Some(before_id));

                // Update the before node's previous pointer
                self.previous_sibling.insert(before_id, Some(child_id));

                // Update the previous node's next pointer
                if let Some(prev_id) = previous {
                    self.next_sibling.insert(prev_id, Some(child_id));
                }

                // Insert into children list
                children.insert(index, child_id);

                // Update parent
                self.parents.insert(child_id, Some(parent_id));
            }
        }
    }

    /// Insert a child after a specific sibling.
    pub fn insert_child_after(&mut self, parent_id: K, child_id: K, after_id: K) {
        // Detach from current parent if any
        if let Some(Some(old_parent)) = self.parents.get(child_id).copied() {
            self.detach_child(old_parent, child_id);
        }

        if let Some(children) = self.children.get_mut(parent_id) {
            if let Some(index) = children.iter().position(|&id| id == after_id) {
                let next = children.get(index + 1).copied();

                // Update the new child's pointers
                self.previous_sibling.insert(child_id, Some(after_id));
                self.next_sibling.insert(child_id, next);

                // Update the after node's next pointer
                self.next_sibling.insert(after_id, Some(child_id));

                // Update the next node's previous pointer
                if let Some(next_id) = next {
                    self.previous_sibling.insert(next_id, Some(child_id));
                }

                // Insert into children list
                children.insert(index + 1, child_id);

                // Update parent
                self.parents.insert(child_id, Some(parent_id));
            }
        }
    }

    /// Detach a child from its parent (but don't remove it from the tree).
    pub fn detach_child(&mut self, parent_id: K, child_id: K) {
        if let Some(children) = self.children.get_mut(parent_id) {
            if let Some(position) = children.iter().position(|&x| x == child_id) {
                let previous = if position > 0 {
                    Some(children[position - 1])
                } else {
                    None
                };

                let next = children.get(position + 1).copied();

                // Update surrounding siblings' links
                if let Some(prev_id) = previous {
                    self.next_sibling.insert(prev_id, next);
                }

                if let Some(next_id) = next {
                    self.previous_sibling.insert(next_id, previous);
                }

                // Clear the child's links
                self.previous_sibling.insert(child_id, None);
                self.next_sibling.insert(child_id, None);

                // Clear parent
                self.parents.insert(child_id, None);

                // Remove from Vec
                children.remove(position);
            }
        }
    }

    // Navigation methods
    pub fn next_sibling(&self, id: K) -> Option<K> {
        match self.next_sibling.get(id) {
            Some(option) => *option,
            _ => None,
        }
    }

    pub fn previous_sibling(&self, id: K) -> Option<K> {
        match self.previous_sibling.get(id) {
            Some(option) => *option,
            _ => None,
        }
    }

    pub fn parent(&self, id: K) -> Option<K> {
        match self.parents.get(id) {
            Some(option) => *option,
            _ => None,
        }
    }

    pub fn children(&self, parent: K) -> Option<&[K]> {
        self.children.get(parent).map(|v| v.as_slice())
    }

    pub fn first_child(&self, parent: K) -> Option<K> {
        self.children.get(parent).and_then(|v| v.first().copied())
    }

    pub fn last_child(&self, parent: K) -> Option<K> {
        self.children.get(parent).and_then(|v| v.last().copied())
    }

    /// Iterator over siblings (including the starting node).
    pub fn siblings(&self, id: K) -> SiblingIterator<K, V> {
        let parent = self.parent(id);
        let first = parent.and_then(|p| self.first_child(p));

        SiblingIterator {
            tree: self,
            current: first,
            end: None,
        }
    }

    /// Iterator over children of a node.
    pub fn children_iter(&self, parent: K) -> ChildrenIterator<K, V> {
        let first = self.first_child(parent);

        ChildrenIterator {
            tree: self,
            current: first,
        }
    }
}

// Iterators
pub struct SiblingIterator<'a, K: Key, V> {
    tree: &'a Tree<K, V>,
    current: Option<K>,
    end: Option<K>,
}

impl<'a, K: Key, V> Iterator for SiblingIterator<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        if Some(current) == self.end {
            self.current = None;
            return None;
        }

        self.current = self.tree.next_sibling(current);
        Some(current)
    }
}

pub struct ChildrenIterator<'a, K: Key, V> {
    tree: &'a Tree<K, V>,
    current: Option<K>,
}

impl<'a, K: Key, V> Iterator for ChildrenIterator<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.tree.next_sibling(current);
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::new_key_type;

    new_key_type! {
        pub struct NodeId;
    }

    #[derive(Debug, PartialEq)]
    struct Node(&'static str);

    #[test]
    fn test_basic_operations() {
        let mut tree = Tree::<NodeId, Node>::new();

        let root = tree.insert(Node("root"));
        let child1 = tree.insert(Node("child1"));
        let child2 = tree.insert(Node("child2"));

        tree.append_child(root, child1);
        tree.append_child(root, child2);

        // Check parent-child relationships
        assert_eq!(tree.parent(child1), Some(root));
        assert_eq!(tree.parent(child2), Some(root));

        // Check sibling relationships
        assert_eq!(tree.next_sibling(child1), Some(child2));
        assert_eq!(tree.previous_sibling(child2), Some(child1));

        // Check children
        let children = tree.children(root).unwrap();
        assert_eq!(children, &[child1, child2]);
    }

    #[test]
    fn test_insert_before() {
        let mut tree = Tree::<NodeId, Node>::new();

        let root = tree.insert(Node("root"));
        let child1 = tree.insert(Node("child1"));
        let child2 = tree.insert(Node("child2"));
        let child3 = tree.insert(Node("child3"));

        tree.append_child(root, child1);
        tree.append_child(root, child3);
        tree.insert_child_before(root, child2, child3);

        let children = tree.children(root).unwrap();
        assert_eq!(children, &[child1, child2, child3]);

        assert_eq!(tree.next_sibling(child1), Some(child2));
        assert_eq!(tree.next_sibling(child2), Some(child3));
    }

    #[test]
    fn test_remove() {
        let mut tree = Tree::<NodeId, Node>::new();

        let root = tree.insert(Node("root"));
        let child = tree.insert(Node("child"));
        let grandchild = tree.insert(Node("grandchild"));

        tree.append_child(root, child);
        tree.append_child(child, grandchild);

        tree.remove(child);

        // Child and grandchild should be gone
        assert!(tree.get(child).is_none());
        assert!(tree.get(grandchild).is_none());

        // Root should have no children
        assert!(tree.children(root).unwrap().is_empty());
    }
}