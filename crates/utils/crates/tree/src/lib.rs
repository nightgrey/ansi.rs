//! A generic, slotmap-backed tree data structure.
//!
//! This crate provides [`Tree`], an arena-allocated tree where every node is
//! addressed by a typed [`Id`] key. Nodes are stored in a flat [`slotmap`] and
//! linked together through embedded parent/child/sibling pointers, giving O(1)
//! insertion, removal, and navigation.
//!
//! # Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`Tree`] | The core tree container — insert, remove, move, and query nodes. |
//! | [`Secondary`] | An auxiliary map for associating side data with tree nodes. |
//! | [`Node`] | A node with embedded structural links; dereferences to its inner value. |
//! | [`At`] | Describes *where* to insert or move a node (child, sibling, detached). |
//! | [`Id`] | Trait that extends [`slotmap::Key`] with `Option`-like combinators. |
//!
//! # Quick start
//!
//! ```rust
//! use tree::*;
//!
//! // Create a custom id type (or use `DefaultId`).
//! id!(pub struct MyId);
//!
//! let mut tree = Tree::<MyId, &str>::new();
//! let root  = tree.insert("root");
//! let hello = tree.insert_at("hello", At::Child(root));
//! let world = tree.insert_at("world", At::Child(root));
//!
//! assert_eq!(tree.first_child(root), Some(hello));
//! assert_eq!(tree.next_sibling(hello), Some(world));
//!
//! for child in tree.children(root) {
//!     println!("{}", *tree[child]);
//! }
//! ```

#[macro_export]
mod id;
mod at;
mod error;
pub mod iter;
mod node;
mod tree;
mod secondary;
mod traits;

pub use at::*;
pub use error::*;
pub use id::*;
pub use iter::*;
pub use node::*;
pub use secondary::*;
pub use tree::*;
pub use traits::*;