#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At<T> {
    /// Insert the node as a detached node.
    Detached,

    /// Insert the node as the first child of the given ID.
    FirstChild(T),

    /// Insert the node as the child of the given ID.
    Child(T),

    /// Insert the node as a sibling before the node with the given ID.
    Before(T),
    /// Insert the node as a sibling after the node with the given ID.
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