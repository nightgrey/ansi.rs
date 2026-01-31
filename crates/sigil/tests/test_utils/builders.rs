//! Test builders for convenient test construction.

use kasten::{Node, Content, Rect, Position, Point, Size};

/// Quick rectangle builder.
///
/// # Example
/// ```
/// # use kasten::Rect;
/// # fn rect(x: usize, y: usize, w: usize, h: usize) -> Rect { Rect::new((x, y), (x + w, y + h)) }
/// let r = rect(10, 5, 20, 15);
/// assert_eq!(r.width(), 20);
/// assert_eq!(r.height(), 15);
/// ```
pub fn rect(x: usize, y: usize, w: usize, h: usize) -> Rect {
    Rect::new((x, y), (x + w, y + h))
}

/// Quick position builder.
///
/// # Example
/// ```
/// # use kasten::Position;
/// # fn pos(row: usize, col: usize) -> Position { Position::new(row, col) }
/// let p = pos(5, 10);
/// assert_eq!(p.row, 5);
/// assert_eq!(p.col, 10);
/// ```
pub fn pos(row: usize, col: usize) -> Position {
    Position::new(row, col)
}

/// Quick point builder.
pub fn point(x: usize, y: usize) -> Point {
    Point::new(x, y)
}

/// Quick size builder.
pub fn size(width: usize, height: usize) -> Size {
    Size::new(width, height)
}

/// Quick text node builder.
///
/// # Example
/// ```
/// # use kasten::{Node, Content};
/// # fn text_node(s: &str) -> Node { Node::Base(Content::Text(s.into())) }
/// let node = text_node("Hello");
/// ```
pub fn text_node(s: &str) -> Node {
    Node::Base(Content::Text(s.into()))
}

/// Quick empty node builder.
pub fn empty_node() -> Node {
    Node::Base(Content::Empty)
}

/// Quick fill node builder.
pub fn fill_node(ch: char) -> Node {
    Node::Base(Content::Fill(ch))
}

/// Quick stack builder.
pub fn stack(children: Vec<Node>) -> Node {
    Node::Stack(children)
}

/// Quick row builder.
pub fn row(children: Vec<Node>) -> Node {
    Node::Row(children)
}
