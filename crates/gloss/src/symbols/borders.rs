use super::{Blocks, LineSymbols, Symbol};
use etwa::Maybe;
use geometry::Edges;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BorderSymbols {
    pub top_left: Symbol<char>,
    pub top: Symbol<char>,
    pub top_right: Symbol<char>,

    pub right: Symbol<char>,

    pub bottom_left: Symbol<char>,
    pub bottom: Symbol<char>,
    pub bottom_right: Symbol<char>,

    pub left: Symbol<char>,
}

impl BorderSymbols {
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

    pub fn bottom_width(&self) -> usize {
        let bottom_left = self.bottom_left.width();
        let bottom = self.bottom.width();
        let bottom_right = self.bottom_right.width();

        bottom_left.max(bottom).max(bottom_right)
    }

    pub fn left_width(&self) -> usize {
        let top_left = self.top_left.width();
        let left = self.left.width();
        let bottom_left = self.bottom_left.width();

        top_left.max(left).max(bottom_left)
    }

    pub fn to_edges(self) -> Edges<usize> {
        Edges::new(
            self.top_width(),
            self.right_width(),
            self.bottom_width(),
            self.left_width(),
        )
    }

    pub const fn from_line(line: LineSymbols) -> Self {
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

impl BorderSymbols {
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
    pub const SINGLE: Self = BorderSymbols::from_line(LineSymbols::LIGHT);
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
    pub const BOLD: Self = BorderSymbols::from_line(LineSymbols::BOLD);

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
    pub const ROUNDED: Self = BorderSymbols::from_line(LineSymbols::ROUNDED);

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
    pub const DOUBLE: Self = BorderSymbols::from_line(LineSymbols::DOUBLE);

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
    pub const DASHED: Self = BorderSymbols::from_line(LineSymbols::DASHED_DOUBLE);

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
    pub const DASHED_BOLD: Self = BorderSymbols::from_line(LineSymbols::DASHED_DOUBLE_BOLD);

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
    pub const DASHED_TRIPLE: Self = BorderSymbols::from_line(LineSymbols::DASHED_TRIPLE);

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
    pub const DASHED_TRIPLE_BOLD: Self = BorderSymbols::from_line(LineSymbols::DASHED_TRIPLE_BOLD);

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
    pub const DASHED_QUADRUPLE: Self = BorderSymbols::from_line(LineSymbols::DASHED_QUADRUPLE);

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
    pub const DASHED_QUADRUPLE_BOLD: Self = BorderSymbols::from_line(LineSymbols::DASHED_QUADRUPLE_BOLD);

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
    pub const BLOCK_THICK_OUTER: Self = BorderSymbols {
        top_left: Blocks::CORNERS.top_left,
        top_right: Blocks::CORNERS.top_right,
        bottom_left: Blocks::CORNERS.bottom_left,
        bottom_right: Blocks::CORNERS.bottom_right,

        left: Blocks::LEFT.four_eighth,
        right: Blocks::RIGHT.four_eighth,
        top: Blocks::TOP.four_eighth,
        bottom: Blocks::BOTTOM.four_eighth,
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
    pub const BLOCK_THICK_INNER: Self = BorderSymbols {
        top_right: Blocks::CORNERS.bottom_left,
        top_left: Blocks::CORNERS.bottom_right,
        bottom_right: Blocks::CORNERS.top_left,
        bottom_left: Blocks::CORNERS.top_right,
        left: Blocks::RIGHT.four_eighth,
        right: Blocks::LEFT.four_eighth,
        top: Blocks::BOTTOM.four_eighth,
        bottom: Blocks::TOP.four_eighth,
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
    pub const BLOCK_THIN: Self = BorderSymbols {
        top_right: Blocks::TOP.one_eighth,
        top_left: Blocks::TOP.one_eighth,
        bottom_right: Blocks::BOTTOM.one_eighth,
        bottom_left: Blocks::BOTTOM.one_eighth,
        left: Blocks::LEFT.one_eighth,
        right: Blocks::RIGHT.one_eighth,
        top: Blocks::TOP.one_eighth,
        bottom: Blocks::BOTTOM.one_eighth,
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
    pub const BLOCK_THIN_TALL: Self = BorderSymbols {
        top_right: LineSymbols::LIGHT.top_right,
        top_left: Blocks::RIGHT.one_eighth,
        bottom_right: LineSymbols::LIGHT.bottom_right,
        bottom_left: LineSymbols::LIGHT.bottom_left,
        left: Blocks::LEFT.one_eighth,
        right: Blocks::RIGHT.one_eighth,
        top: Blocks::TOP.one_eighth,
        bottom: Blocks::BOTTOM.one_eighth,
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
    pub const BLOCK_MEDIUM: Self = BorderSymbols {
        top_left: Blocks::BOTTOM.four_eighth,
        top: Blocks::BOTTOM.four_eighth,
        top_right: Blocks::BOTTOM.four_eighth,

        right: Blocks::BOTTOM.eight_eighth,

        bottom_left: Blocks::TOP.four_eighth,
        bottom: Blocks::TOP.four_eighth,
        bottom_right: Blocks::TOP.four_eighth,

        left: Blocks::BOTTOM.eight_eighth,
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
    pub const BLOCK_MEDIUM_TALL: Self = BorderSymbols {
        top_left: Blocks::BOTTOM.eight_eighth,
        top: Blocks::TOP.four_eighth,
        top_right: Blocks::BOTTOM.eight_eighth,

        right: Blocks::BOTTOM.eight_eighth,

        bottom_left: Blocks::TOP.eight_eighth,
        bottom: Blocks::BOTTOM.four_eighth,
        bottom_right: Blocks::TOP.eight_eighth,

        left: Blocks::BOTTOM.eight_eighth,
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
    pub const BLOCK_SOLID: Self = BorderSymbols {
        top_left: Blocks::BOTTOM.eight_eighth,
        top: Blocks::BOTTOM.eight_eighth,
        top_right: Blocks::BOTTOM.eight_eighth,

        right: Blocks::BOTTOM.eight_eighth,

        bottom_left: Blocks::BOTTOM.eight_eighth,
        bottom: Blocks::BOTTOM.eight_eighth,
        bottom_right: Blocks::BOTTOM.eight_eighth,

        left: Blocks::BOTTOM.eight_eighth,
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
    pub const INVISIBLE: Self = BorderSymbols {
        top_left: Symbol { inner: ' ', width: 1 },
        top_right: Symbol { inner: ' ', width: 1 },
        bottom_left: Symbol { inner: ' ', width: 1 },
        bottom_right: Symbol { inner: ' ', width: 1 },
        left: Symbol { inner: ' ', width: 1 },
        right: Symbol { inner: ' ', width: 1 },
        top: Symbol { inner: ' ', width: 1 },
        bottom: Symbol { inner: ' ', width: 1 },
    };
    pub const NONE: Self = BorderSymbols {
        top_left: Symbol { inner: char::MIN, width: 0 },
        top_right: Symbol { inner: char::MIN, width: 0 },
        bottom_left: Symbol { inner: char::MIN, width: 0 },
        bottom_right: Symbol { inner: char::MIN, width: 0 },
        left: Symbol { inner: char::MIN, width: 0 },
        right: Symbol { inner: char::MIN, width: 0 },
        top: Symbol { inner: char::MIN, width: 0 },
        bottom: Symbol { inner: char::MIN, width: 0 },
    };
}

impl Default for BorderSymbols {
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

    pub fn width(&self) -> usize {
        match self {
            BorderStyle::None => 0,
            _ => 1,
        }
    }

    /// Returns the width of the borders as [`Edges`].
    pub fn into_edges(self) -> Edges<usize> {
        if !self.is_some() {
            return Edges::ZERO;
        }

        self.into_symbols().to_edges()
    }

    /// Convert a border style into a [`BorderSymbols`].
    pub fn into_symbols(self) ->   BorderSymbols {
        match self {
            BorderStyle::Solid =>BorderSymbols::SINGLE,
            BorderStyle::Bold => BorderSymbols::BOLD,
            BorderStyle::Rounded => BorderSymbols::ROUNDED,
            BorderStyle::Double => BorderSymbols::DOUBLE,
            BorderStyle::Dashed => BorderSymbols::DASHED,
            BorderStyle::DashedBold => BorderSymbols::DASHED_BOLD,
            BorderStyle::DashedTriple => BorderSymbols::DASHED_TRIPLE,
            BorderStyle::DashedTripleBold => BorderSymbols::DASHED_TRIPLE_BOLD,
            BorderStyle::DashedQuadruple => BorderSymbols::DASHED_QUADRUPLE,
            BorderStyle::DashedQuadrupleBold => BorderSymbols::DASHED_QUADRUPLE_BOLD,
            BorderStyle::BlockThickOuter => BorderSymbols::BLOCK_THICK_OUTER,
            BorderStyle::BlockThickInner => BorderSymbols::BLOCK_THICK_INNER,
            BorderStyle::BlockThin => BorderSymbols::BLOCK_THIN,
            BorderStyle::BlockThinTall => BorderSymbols::BLOCK_THIN_TALL,
            BorderStyle::BlockMedium => BorderSymbols::BLOCK_MEDIUM,
            BorderStyle::BlockTallMediumTall => BorderSymbols::BLOCK_MEDIUM_TALL,
            BorderStyle::BlockSolid => BorderSymbols::BLOCK_SOLID,
            BorderStyle::Invisible => BorderSymbols::INVISIBLE,
            BorderStyle::None => BorderSymbols::NONE,
        }
    }
}
impl From<BorderStyle> for BorderSymbols {
    fn from(style: BorderStyle) -> Self {
        style.into_symbols()
    }
}
