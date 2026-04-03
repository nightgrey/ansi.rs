use super::{Block, Line, Symbol};
use maybe::Maybe;
use geometry::Edges;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Border {
    pub top_left: Symbol,
    pub top: Symbol,
    pub top_right: Symbol,

    pub right: Symbol,

    pub bottom_left: Symbol,
    pub bottom: Symbol,
    pub bottom_right: Symbol,

    pub left: Symbol,
}

impl Border {
    pub fn new(
        top_left: char,
        top: char,
        top_right: char,
        right: char,
        bottom_left: char,
        bottom: char,
        bottom_right: char,
        left: char,
    ) -> Self {
        Self {
            top_left: Symbol::measured(top_left),
            top: Symbol::measured(top),
            top_right: Symbol::measured(top_right),
            right: Symbol::measured(right),
            bottom_left: Symbol::measured(bottom_left),
            bottom: Symbol::measured(bottom),
            bottom_right: Symbol::measured(bottom_right),
            left: Symbol::measured(left),
        }
    }

    pub fn top_width(&self) -> usize {
        let top_left = self.top_left.width();
        let top = self.top.width();
        let top_right = self.top_right.width();

        top_left.max(top).max(top_right)
    }

    pub fn right_width(&self) -> usize {
        let top_right = self.top_right.width();
        let top = self.top.width();
        let bottom_right = self.bottom_right.width();

        top_right.max(top).max(bottom_right)
    }

    pub fn left_width(&self) -> usize {
        let top_left = self.top_left.width();
        let left = self.left.width();
        let bottom_left = self.bottom_left.width();

        top_left.max(left).max(bottom_left)
    }

    pub fn bottom_width(&self) -> usize {
        let bottom_left = self.bottom_left.width();
        let bottom = self.bottom.width();
        let bottom_right = self.bottom_right.width();

        bottom_left.max(bottom).max(bottom_right)
    }

    pub fn horizontal_width(&self) -> usize {
        let left = self.left.width();
        let right = self.right.width();

        left.max(right)
    }

    pub fn vertical_width(&self) -> usize {
        let top = self.top.width();
        let bottom = self.bottom.width();

        top.max(bottom)
    }

    pub fn to_edges(self) -> Edges<usize> {
        Edges::new(
            self.top_width(),
            self.right_width(),
            self.bottom_width(),
            self.left_width(),
        )
    }

    pub const fn from_line(line: Line) -> Self {
        Self {
            top_left: line.top_left,
            top_right: line.top_right,
            bottom_left: line.bottom_left,
            bottom_right: line.bottom_right,
            left: line.vertical,
            right: line.vertical,
            top: line.horizontal,
            bottom: line.horizontal,
        }
    }
}

impl Border {
    /// A standard border using normal line-drawing characters
    ///
    /// Creates a clean, lightweight border suitable for general use.
    ///
    /// # Example
    /// ```text
    ///  ┌───────┐
    ///  │       │
    ///  │  xxx  │
    ///  │  xxx  │
    ///  │       │
    ///  └───────┘
    /// ```
    pub const SINGLE: Self = Border::from_line(Line::LIGHT);
    /// A bold border using bold line-drawing characters
    ///
    /// Creates a prominent border with increased visual weight.
    ///
    /// # Example
    /// ```text
    ///  ┏━━━━━━━┓
    ///  ┃       ┃
    ///  ┃  xxx  ┃
    ///  ┃  xxx  ┃
    ///  ┃       ┃
    ///  ┗━━━━━━━┛
    /// ```
    pub const BOLD: Self = Border::from_line(Line::BOLD);

    /// A rounded border with smooth corners
    ///
    /// Creates a softer, more approachable visual style.
    ///
    /// # Example
    /// ```text
    ///  ╭───────╮
    ///  │       │
    ///  │  xxx  │
    ///  │  xxx  │
    ///  │       │
    ///  ╰───────╯
    /// ```
    pub const ROUNDED: Self = Border::from_line(Line::ROUNDED);

    /// A border using double-line characters
    ///
    /// Creates a formal, structured appearance with parallel lines.
    ///
    /// # Example
    /// ```text
    ///  ╔═══════╗
    ///  ║       ║
    ///  ║  xxx  ║
    ///  ║  xxx  ║
    ///  ║       ║
    ///  ╚═══════╝
    /// ```
    pub const DOUBLE: Self = Border::from_line(Line::DOUBLE);

    /// A border using single-dashed lines
    ///
    /// Creates a subtle, non-intrusive border for secondary content.
    ///
    /// # Example
    /// ```text
    ///  ┌╌╌╌╌╌╌╌┐
    ///  ╎       ╎
    ///  ╎  xxx  ╎
    ///  ╎  xxx  ╎
    ///  ╎       ╎
    ///  └╌╌╌╌╌╌╌┘
    /// ```
    pub const DASHED: Self = Border::from_line(Line::DASHED_DOUBLE);

    /// A bold border using single-dashed lines
    ///
    /// Combines emphasis with visual lightness through dashed styling.
    ///
    /// # Example
    /// ```text
    ///  ┏╍╍╍╍╍╍╍┓
    ///  ╏       ╏
    ///  ╏  xxx  ╏
    ///  ╏  xxx  ╏
    ///  ╏       ╏
    ///  ┗╍╍╍╍╍╍╍┛
    /// ```
    pub const DASHED_BOLD: Self = Border::from_line(Line::DASHED_DOUBLE_BOLD);

    /// A border using triple-dashed lines
    ///
    /// Creates a delicate, decorative border with lighter visual weight.
    ///
    /// # Example
    /// ```text
    ///  ┌┄┄┄┄┄┄┄┐
    ///  ┆       ┆
    ///  ┆  xxx  ┆
    ///  ┆  xxx  ┆
    ///  ┆       ┆
    ///  └┄┄┄┄┄┄┄┘
    /// ```
    pub const DASHED_TRIPLE: Self = Border::from_line(Line::DASHED_TRIPLE);

    /// A bold border using triple-dashed lines
    ///
    /// Provides emphasis while maintaining an airy, segmented appearance.
    ///
    /// # Example
    /// ```text
    ///  ┏┅┅┅┅┅┅┅┓
    ///  ┇       ┇
    ///  ┇  xxx  ┇
    ///  ┇  xxx  ┇
    ///  ┇       ┇
    ///  ┗┅┅┅┅┅┅┅┛
    /// ```
    pub const DASHED_TRIPLE_BOLD: Self = Border::from_line(Line::DASHED_TRIPLE_BOLD);

    /// A border using quadruple-dashed lines
    ///
    /// Creates the most subtle dashed border, ideal for minimal interference.
    ///
    /// # Example
    /// ```text
    ///  ┌┈┈┈┈┈┈┈┐
    ///  ┊       ┊
    ///  ┊  xxx  ┊
    ///  ┊  xxx  ┊
    ///  ┊       ┊
    ///  └┈┈┈┈┈┈┈┘
    /// ```
    pub const DASHED_QUADRUPLE: Self = Border::from_line(Line::DASHED_QUADRUPLE);

    /// A bold border using quadruple-dashed lines
    ///
    /// Balances prominence with segmentation for distinctive framing.
    ///
    /// # Example
    /// ```text
    ///  ┏┉┉┉┉┉┉┉┓
    ///  ┋       ┋
    ///  ┋  xxx  ┋
    ///  ┋  xxx  ┋
    ///  ┋       ┋
    ///  ┗┉┉┉┉┉┉┉┛
    /// ```
    pub const DASHED_QUADRUPLE_BOLD: Self = Border::from_line(Line::DASHED_QUADRUPLE_BOLD);

    /// A thick border using bold block characters pointing outward
    ///
    /// Creates a solid, impactful frame with maximum visual presence.
    ///
    /// # Example
    /// ```text
    ///  ▛▀▀▀▀▀▀▀▜
    ///  ▌       ▐
    ///  ▌  xxx  ▐
    ///  ▌  xxx  ▐
    ///  ▌       ▐
    ///  ▙▄▄▄▄▄▄▄▟
    /// ```
    pub const BLOCK_THICK_OUTER: Self = Border {
        top_left: Block::CORNER.top_left,
        top_right: Block::CORNER.top_right,
        bottom_left: Block::CORNER.bottom_left,
        bottom_right: Block::CORNER.bottom_right,

        left: Block::LEFT.four_eighth,
        right: Block::RIGHT.four_eighth,
        top: Block::TOP.four_eighth,
        bottom: Block::BOTTOM.four_eighth,
    };

    /// A thick border using block characters pointing inward
    ///
    /// Creates an inverted effect where borders appear to contain the space.
    ///
    /// # Example
    /// ```text
    /// ▗▄▄▄▄▄▄▄▖
    /// ▐       ▌
    /// ▐  xxx  ▌
    /// ▐  xxx  ▌
    /// ▐       ▌
    /// ▝▀▀▀▀▀▀▀▘
    /// ```
    pub const BLOCK_THICK_INNER: Self = Border {
        top_right: Block::CORNER.bottom_left,
        top_left: Block::CORNER.bottom_right,
        bottom_right: Block::CORNER.top_left,
        bottom_left: Block::CORNER.top_right,
        left: Block::RIGHT.four_eighth,
        right: Block::LEFT.four_eighth,
        top: Block::BOTTOM.four_eighth,
        bottom: Block::TOP.four_eighth,
    };

    /// A thin border using thin one-eighth block characters
    ///
    /// Creates an extremely subtle frame with minimal visual weight.
    ///
    /// # Example
    /// ```text
    ///  ▁▁▁▁▁▁▁▁▁
    ///  ▏       ▕
    ///  ▏  xxx  ▕
    ///  ▏  xxx  ▕
    ///  ▏       ▕
    ///  ▔▔▔▔▔▔▔▔▔
    /// ```
    pub const BLOCK_THIN: Self = Border {
        top_right: Block::TOP.one_eighth,
        top_left: Block::TOP.one_eighth,
        bottom_right: Block::BOTTOM.one_eighth,
        bottom_left: Block::BOTTOM.one_eighth,
        left: Block::LEFT.one_eighth,
        right: Block::RIGHT.one_eighth,
        top: Block::TOP.one_eighth,
        bottom: Block::BOTTOM.one_eighth,
    };
    /// A tall thin border using the McGugan rendering technique
    ///
    /// Optimizes vertical alignment for better proportions in terminal display.
    ///
    /// # Example
    /// ```text
    ///  ▕▔▔▔▔▔▔▔▏
    ///  ▕       ▏
    ///  ▕  xxx  ▏
    ///  ▕  xxx  ▏
    ///  ▕       ▏
    ///  ▕▁▁▁▁▁▁▁▏
    /// ```
    pub const BLOCK_THIN_TALL: Self = Border {
        top_right: Line::LIGHT.top_right,
        top_left: Block::RIGHT.one_eighth,
        bottom_right: Line::LIGHT.bottom_right,
        bottom_left: Line::LIGHT.bottom_left,
        left: Block::LEFT.one_eighth,
        right: Block::RIGHT.one_eighth,
        top: Block::TOP.one_eighth,
        bottom: Block::BOTTOM.one_eighth,
    };

    /// A proportional border with balanced visual weight
    ///
    /// Uses four-eighth blocks for top and bottom, eight-eighth for sides,
    /// creating horizontal and vertical lines that appear equal in thickness.
    ///
    /// # Example
    /// ```text
    ///  ▄▄▄▄▄▄▄▄▄
    ///  █       █
    ///  █  xxx  █
    ///  █  xxx  █
    ///  █       █
    ///  ▀▀▀▀▀▀▀▀▀
    /// ```
    pub const BLOCK_MEDIUM: Self = Border {
        top_left: Block::BOTTOM.four_eighth,
        top: Block::BOTTOM.four_eighth,
        top_right: Block::BOTTOM.four_eighth,

        right: Block::BOTTOM.eight_eighth,

        bottom_left: Block::TOP.four_eighth,
        bottom: Block::TOP.four_eighth,
        bottom_right: Block::TOP.four_eighth,

        left: Block::BOTTOM.eight_eighth,
    };

    /// A tall proportional border with enhanced vertical balance
    ///
    /// Uses eight-eighth blocks for all sides except top and bottom edges,
    /// which use four-eighth blocks to maintain proportional appearance.
    ///
    /// # Example
    /// ```text
    ///  ▕█▀▀▀▀▀▀▀█
    ///  ▕█       █
    ///  ▕█  xxx  █
    ///  ▕█  xxx  █
    ///  ▕█       █
    ///  ▕█▄▄▄▄▄▄▄█
    /// ```
    pub const BLOCK_MEDIUM_TALL: Self = Border {
        top_left: Block::BOTTOM.eight_eighth,
        top: Block::TOP.four_eighth,
        top_right: Block::BOTTOM.eight_eighth,

        right: Block::BOTTOM.eight_eighth,

        bottom_left: Block::TOP.eight_eighth,
        bottom: Block::BOTTOM.four_eighth,
        bottom_right: Block::TOP.eight_eighth,

        left: Block::BOTTOM.eight_eighth,
    };

    /// A solid block border using full-width characters
    ///
    /// Creates the most substantial border with complete visual enclosure.
    ///
    /// # Example
    /// ```text
    ///  ██████████
    ///  █        █
    ///  █  xxx   █
    ///  █  xxx   █
    ///  █        █
    ///  ██████████
    /// ```
    pub const BLOCK_SOLID: Self = Border {
        top_left: Block::BOTTOM.eight_eighth,
        top: Block::BOTTOM.eight_eighth,
        top_right: Block::BOTTOM.eight_eighth,

        right: Block::BOTTOM.eight_eighth,

        bottom_left: Block::BOTTOM.eight_eighth,
        bottom: Block::BOTTOM.eight_eighth,
        bottom_right: Block::BOTTOM.eight_eighth,

        left: Block::BOTTOM.eight_eighth,
    };

    /// An invisible border using whitespace
    ///
    /// Preserves spacing and layout structure without visible border characters.
    /// Useful for consistent padding, layering effects, or placeholder borders.
    ///
    /// # Example
    /// ```text
    ///
    ///
    ///     xxx
    ///     xxx
    ///
    ///
    /// ```
    pub const INVISIBLE: Self = Border {
        top_left: Symbol::SPACE,
        top_right: Symbol::SPACE,
        bottom_left: Symbol::SPACE,
        bottom_right: Symbol::SPACE,
        left: Symbol::SPACE,
        right: Symbol::SPACE,
        top: Symbol::SPACE,
        bottom: Symbol::SPACE,
    };
    pub const NONE: Self = Border {
        top_left: Symbol::MIN,
        top_right: Symbol::MIN,
        bottom_left: Symbol::MIN,
        bottom_right: Symbol::MIN,
        left: Symbol::MIN,
        right: Symbol::MIN,
        top: Symbol::MIN,
        bottom: Symbol::MIN,
    };
}

impl Default for Border {
    fn default() -> Self {
        Self::SINGLE
    }
}

/// Visual style of the border that is drawn around a box or table.
///
/// Each variant maps to a concrete set of Unicode (or ASCII) characters that
/// are used for the top, bottom, left, right, corners and edge centers.
#[derive(Copy, Maybe, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum BorderStyle {
    #[none]
    None,
    /// A standard border using normal line-drawing characters
    ///
    /// Creates a clean, lightweight border suitable for general use.
    ///
    /// # Example
    /// ```text
    ///  ┌───────┐
    ///  │       │
    ///  │  xxx  │
    ///  │  xxx  │
    ///  │       │
    ///  └───────┘
    /// ```
    #[default]
    Solid,

    /// A bold border using bold line-drawing characters
    ///
    /// Creates a prominent border with increased visual weight.
    ///
    /// # Example
    /// ```text
    ///  ┏━━━━━━━┓
    ///  ┃       ┃
    ///  ┃  xxx  ┃
    ///  ┃  xxx  ┃
    ///  ┃       ┃
    ///  ┗━━━━━━━┛
    /// ```
    Bold,

    /// A rounded border with smooth corners
    ///
    /// Creates a softer, more approachable visual style.
    ///
    /// # Example
    /// ```text
    ///  ╭───────╮
    ///  │       │
    ///  │  xxx  │
    ///  │  xxx  │
    ///  │       │
    ///  ╰───────╯
    /// ```
    Rounded,

    /// A double-line border using pipe-style characters
    ///
    /// Creates a formal, structured appearance with parallel lines.
    ///
    /// # Example
    /// ```text
    ///  ╔═══════╗
    ///  ║       ║
    ///  ║  xxx  ║
    ///  ║  xxx  ║
    ///  ║       ║
    ///  ╚═══════╝
    /// ```
    Double,

    /// A border using single-dashed lines
    ///
    /// Creates a subtle, non-intrusive border for secondary content.
    ///
    /// # Example
    /// ```text
    ///  ┌╌╌╌╌╌╌╌┐
    ///  ╎       ╎
    ///  ╎  xxx  ╎
    ///  ╎  xxx  ╎
    ///  ╎       ╎
    ///  └╌╌╌╌╌╌╌┘
    /// ```
    Dashed,

    /// A bold border using single-dashed lines
    ///
    /// Combines emphasis with visual lightness through dashed styling.
    ///
    /// # Example
    /// ```text
    ///  ┏╍╍╍╍╍╍╍┓
    ///  ╏       ╏
    ///  ╏  xxx  ╏
    ///  ╏  xxx  ╏
    ///  ╏       ╏
    ///  ┗╍╍╍╍╍╍╍┛
    /// ```
    DashedBold,

    /// A border using triple-dashed lines
    ///
    /// Creates a delicate, decorative border with lighter visual weight.
    ///
    /// # Example
    /// ```text
    ///  ┌┄┄┄┄┄┄┄┐
    ///  ┆       ┆
    ///  ┆  xxx  ┆
    ///  ┆  xxx  ┆
    ///  ┆       ┆
    ///  └┄┄┄┄┄┄┄┘
    /// ```
    DashedTriple,

    /// A bold border using triple-dashed lines
    ///
    /// Provides emphasis while maintaining an airy, segmented appearance.
    ///
    /// # Example
    /// ```text
    ///  ┏┅┅┅┅┅┅┅┓
    ///  ┇       ┇
    ///  ┇  xxx  ┇
    ///  ┇  xxx  ┇
    ///  ┇       ┇
    ///  ┗┅┅┅┅┅┅┅┛
    /// ```
    DashedTripleBold,

    /// A border using quadruple-dashed lines
    ///
    /// Creates the most subtle dashed border, ideal for minimal interference.
    ///
    /// # Example
    /// ```text
    ///  ┌┈┈┈┈┈┈┈┐
    ///  ┊       ┊
    ///  ┊  xxx  ┊
    ///  ┊  xxx  ┊
    ///  ┊       ┊
    ///  └┈┈┈┈┈┈┈┘
    /// ```
    DashedQuadruple,

    /// A bold border using quadruple-dashed lines
    ///
    /// Balances prominence with segmentation for distinctive framing.
    ///
    /// # Example
    /// ```text
    ///  ┏┉┉┉┉┉┉┉┓
    ///  ┋       ┋
    ///  ┋  xxx  ┋
    ///  ┋  xxx  ┋
    ///  ┋       ┋
    ///  ┗┉┉┉┉┉┉┉┛
    /// ```
    DashedQuadrupleBold,

    /// A thick border using bold block characters pointing outward
    ///
    /// Creates a solid, impactful frame with maximum visual presence.
    ///
    /// # Example
    /// ```text
    ///  ▛▀▀▀▀▀▀▀▜
    ///  ▌       ▐
    ///  ▌  xxx  ▐
    ///  ▌  xxx  ▐
    ///  ▌       ▐
    ///  ▙▄▄▄▄▄▄▄▟
    /// ```
    BlockThickOuter,

    /// A thick border using block characters pointing inward
    ///
    /// Creates an inverted effect where borders appear to contain the space.
    ///
    /// # Example
    /// ```text
    /// ▗▄▄▄▄▄▄▄▖
    ///  ▌       ▐
    ///  ▌  xxx  ▐
    ///  ▌  xxx  ▐
    ///  ▌       ▐
    ///  ▝▀▀▀▀▀▀▀▘
    /// ```
    BlockThickInner,

    /// A thin border using thin one-eighth block characters
    ///
    /// Creates an extremely subtle frame with minimal visual weight.
    ///
    /// # Example
    /// ```text
    ///  ▁▁▁▁▁▁▁▁▁
    ///  ▏       ▕
    ///  ▏  xxx  ▕
    ///  ▏  xxx  ▕
    ///  ▏       ▕
    ///  ▔▔▔▔▔▔▔▔▔
    /// ```
    BlockThin,

    /// A tall thin border using the McGugan rendering technique
    ///
    /// Optimizes vertical alignment for better proportions in terminal display.
    ///
    /// # Example
    /// ```text
    ///  ▕▔▔▔▔▔▔▔▏
    ///  ▕       ▏
    ///  ▕  xxx  ▏
    ///  ▕  xxx  ▏
    ///  ▕       ▏
    ///  ▕▁▁▁▁▁▁▁▏
    /// ```
    BlockThinTall,

    /// A proportional border with balanced visual weight
    ///
    /// Uses four-eighth blocks for top and bottom, eight-eighth for sides,
    /// creating horizontal and vertical lines that appear equal in thickness.
    ///
    /// # Example
    /// ```text
    ///  ▄▄▄▄▄▄▄▄▄
    ///  █       █
    ///  █  xxx  █
    ///  █  xxx  █
    ///  █       █
    ///  ▀▀▀▀▀▀▀▀▀
    /// ```
    BlockMedium,

    /// A tall proportional border with enhanced vertical balance
    ///
    /// Uses eight-eighth blocks for all sides except top and bottom edges,
    /// which use four-eighth blocks to maintain proportional appearance.
    ///
    /// # Example
    /// ```text
    ///  ▕█▀▀▀▀▀▀▀█
    ///  ▕█       █
    ///  ▕█  xxx  █
    ///  ▕█  xxx  █
    ///  ▕█       █
    ///  ▕█▄▄▄▄▄▄▄█
    /// ```
    BlockTallMediumTall,

    /// A solid block border using full-width characters
    ///
    /// Creates the most substantial border with complete visual enclosure.
    ///
    /// # Example
    /// ```text
    ///  ██████████
    ///  █        █
    ///  █  xxx   █
    ///  █  xxx   █
    ///  █        █
    ///  ██████████
    /// ```
    BlockSolid,

    /// An invisible border using whitespace
    ///
    /// Preserves spacing and layout structure without visible border characters.
    /// Useful for consistent padding, layering effects, or placeholder borders.
    ///
    /// # Example
    /// ```text
    ///
    ///
    ///     xxx
    ///     xxx
    ///
    ///
    /// ```
    Invisible,
}

impl BorderStyle {
    pub fn is_visible(&self) -> bool {
        match self {
            BorderStyle::None | BorderStyle::Invisible => false,
            _ => true,
        }
    }

    /// Returns the width of the borders as [`Edges`].
    pub fn into_edges(self) -> Edges<usize> {
        if !self.is_some() {
            return Edges::ZERO;
        }

        self.into_border().to_edges()
    }

    /// Convert a border style into a [`Border`].
    pub fn into_border(self) -> Border {
        match self {
            BorderStyle::Solid => Border::SINGLE,
            BorderStyle::Bold => Border::BOLD,
            BorderStyle::Rounded => Border::ROUNDED,
            BorderStyle::Double => Border::DOUBLE,
            BorderStyle::Dashed => Border::DASHED,
            BorderStyle::DashedBold => Border::DASHED_BOLD,
            BorderStyle::DashedTriple => Border::DASHED_TRIPLE,
            BorderStyle::DashedTripleBold => Border::DASHED_TRIPLE_BOLD,
            BorderStyle::DashedQuadruple => Border::DASHED_QUADRUPLE,
            BorderStyle::DashedQuadrupleBold => Border::DASHED_QUADRUPLE_BOLD,
            BorderStyle::BlockThickOuter => Border::BLOCK_THICK_OUTER,
            BorderStyle::BlockThickInner => Border::BLOCK_THICK_INNER,
            BorderStyle::BlockThin => Border::BLOCK_THIN,
            BorderStyle::BlockThinTall => Border::BLOCK_THIN_TALL,
            BorderStyle::BlockMedium => Border::BLOCK_MEDIUM,
            BorderStyle::BlockTallMediumTall => Border::BLOCK_MEDIUM_TALL,
            BorderStyle::BlockSolid => Border::BLOCK_SOLID,
            BorderStyle::Invisible => Border::INVISIBLE,
            BorderStyle::None => Border::NONE,
        }
    }
}
impl From<BorderStyle> for Border {
    fn from(style: BorderStyle) -> Self {
        style.into_border()
    }
}
