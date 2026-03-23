use geometry::Edges;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Style {
    // paint
    pub foreground: ansi::Color,
    pub background: ansi::Color,
    pub attributes_on: ansi::Attribute,
    pub attributes_off: ansi::Attribute,

    // box model
    pub padding: Edges,
    pub margin: Edges,

    // optional fill chars
    pub padding_char: Option<char>,
    pub margin_char: Option<char>,

    // sizing
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,

    // layout
    pub align_horizontal: Option<AlignHorizontal>,
    pub align_vertical: Option<AlignVertical>,
    pub inline: bool,
    pub tab_width: Option<usize>,

    // decoration
    pub border: Option<Border>,
}