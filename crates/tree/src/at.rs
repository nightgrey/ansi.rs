/// Specifies where a node should be placed within the tree.
///
/// Used by [`Tree::insert_at`](crate::Tree::insert_at),
/// [`Tree::move_to`](crate::Tree::move_to), and related methods to describe
/// the target position relative to an existing node.
///
/// # Variants
///
/// ```text
///       parent
///      /  |  \
///   prev node next      ← Before / After target a sibling
///           \
///          child         ← Child / FirstChild target the parent
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At<T> {
    /// Insert the node without attaching it to any parent.
    Detached,

    /// Insert the node as the **first** child of the given parent.
    FirstChild(T),

    /// Insert the node as the **last** child of the given parent.
    Child(T),

    /// Insert the node as the immediately **preceding** sibling of the given node.
    Before(T),

    /// Insert the node as the immediately **following** sibling of the given node.
    After(T),
}


impl<T> At<T> {
    pub fn map<F, U>(self, f: F) -> At<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            At::Detached => At::Detached,
            At::FirstChild(k) => At::FirstChild(f(k)),
            At::Child(k) => At::Child(f(k)),
            At::Before(k) => At::Before(f(k)),
            At::After(k) => At::After(f(k)),
        }
    }
}