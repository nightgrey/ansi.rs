use ansi::{Style};

pub type Edges = geometry::Edges;

enum Align {
    Start,
    Center,
    End,
}

struct Layout {
    pub padding: Edges,
    pub margin: Edges,
    pub style: Style,
}
