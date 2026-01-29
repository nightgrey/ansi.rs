/// Macros for ergonomic UI tree construction.
///
/// # Example
///
/// ```rust
/// use ansi::{Style, Color};
/// use kasten::*;
///
/// let ui = stack![
///     text!("Title"),
///     row![
///         text!("Left"),
///         fill!(' '),
///         text!("Right"),
///     ],
/// ];
///
/// // With modifiers:
/// let ui = style!(Style::new().bold() =>
///     pad!(Edges::all(1) =>
///         stack![
///             text!("Hello"),
///             text!("World"),
///         ]
///     )
/// );
/// ```

/// Create a text leaf node.
#[macro_export]
macro_rules! text {
    ($s:expr) => {
        $crate::Node::Base($crate::Content::Text($s.into()))
    };
}

/// Create a fill leaf node.
#[macro_export]
macro_rules! fill {
    ($ch:expr) => {
        $crate::Node::Base($crate::Content::Fill($ch))
    };
}

/// Create an empty leaf node.
#[macro_export]
macro_rules! empty {
    () => {
        $crate::Node::Base($crate::Content::Empty)
    };
}

/// Create a vertical stack of children.
#[macro_export]
macro_rules! stack {
    ($($child:expr),* $(,)?) => {
        $crate::Node::Stack(vec![$($child),*])
    };
}

/// Create a horizontal row of children.
#[macro_export]
macro_rules! row {
    ($($child:expr),* $(,)?) => {
        $crate::Node::Row(vec![$($child),*])
    };
}

/// Create overlapping layers of children.
#[macro_export]
macro_rules! layer {
    ($($child:expr),* $(,)?) => {
        $crate::Node::Layer(vec![$($child),*])
    };
}

/// Apply a style to a child node.
#[macro_export]
macro_rules! style {
    ($style:expr => $child:expr) => {
        $crate::Node::Style($style, Box::new($child))
    };
}

/// Apply padding to a child node.
#[macro_export]
macro_rules! pad {
    ($edges:expr => $child:expr) => {
        $crate::Node::Pad($edges, Box::new($child))
    };
}

/// Apply size constraints to a child node.
#[macro_export]
macro_rules! size {
    ($constraints:expr => $child:expr) => {
        $crate::Node::Size($constraints, Box::new($child))
    };
}

/// Align a child node within available space.
#[macro_export]
macro_rules! align {
    ($alignment:expr => $child:expr) => {
        $crate::Node::Align($alignment, Box::new($child))
    };
}

/// Center a child node both horizontally and vertically.
#[macro_export]
macro_rules! center {
    ($child:expr) => {
        $crate::Node::Align(
            $crate::Alignment {
                x: $crate::Align::Center,
                y: $crate::Align::Center,
            },
            Box::new($child),
        )
    };
}
