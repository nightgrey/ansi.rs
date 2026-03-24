/// Line
///
/// Defines symbols for edges, junctions, crosses and diagonal segments.
pub struct Lines<'a> {
    pub center: &'a str,
    pub horizontal: &'a str,
    pub vertical: &'a str,

    pub top_right: &'a str,
    pub top_left: &'a str,
    pub bottom_right: &'a str,
    pub bottom_left: &'a str,

    pub up: &'a str,
    pub right: &'a str,
    pub left: &'a str,
    pub down: &'a str,

    pub top_junction: &'a str,
    pub bottom_junction: &'a str,
    pub right_junction: &'a str,
    pub left_junction: &'a str,

    pub cross: &'a str,

    pub diagonal_left: &'a str,
    pub diagonal_right: &'a str,
}

impl<'a> Lines<'a> {
    pub const fn top(&self) -> &'a str {
        self.horizontal
    }

    pub const fn right(&self) -> &'a str {
        self.vertical
    }

    pub const fn bottom(&self) -> &'a str {
        self.horizontal
    }

    pub const fn left(&self) -> &'a str {
        self.vertical
    }
}

impl Lines<'static> {
    /// Standard light single-line box drawing characters
    ///
    /// Creates clean, lightweight lines suitable for general use in terminal UIs.
    /// These are the most commonly used box-drawing characters with minimal visual weight.
    ///
    /// # Character Set
    /// ```text
    /// ┌─┬─┐
    /// ├─┼─┤
    /// └─┴─┘
    /// ```
    pub const LIGHT: Lines<'static> = Lines {
        top_junction: "┬",
        bottom_junction: "┴",
        right_junction: "┼",
        left_junction: "├",

        center: "┼",

        horizontal: "─",
        vertical: "│",
        top_right: "┐",
        top_left: "┌",
        bottom_right: "┘",
        bottom_left: "└",

        up: "╵",
        right: "╶",
        left: "╴",
        down: "╷",

        cross: "╳",

        diagonal_left: "╱",
        diagonal_right: "╱",
    };
    /// Double-line box drawing characters with parallel lines
    ///
    /// Creates a formal, structured appearance with increased visual emphasis.
    /// Particularly effective for highlighting important sections or primary containers.
    ///
    /// # Character Set
    /// ```text
    /// ┏╍┳╍┓
    /// ┣╍╋╍┫
    /// ┗╍┻╍┛
    /// ```
    pub const BOLD: Lines<'static> = Lines {
        top_junction: "┳",
        bottom_junction: "┻",
        right_junction: "┫",
        left_junction: "┣",

        center: "╋",

        horizontal: "━",
        vertical: "┃",
        top_right: "┓",
        top_left: "┏",
        bottom_right: "┛",
        bottom_left: "┗",

        up: "╹",
        right: "╺",
        left: "╸",
        down: "╻",

        ..Lines::LIGHT
    };
    /// Rounded corner box drawing characters with smooth curves
    ///
    /// Combines normal line edges with softened corners for a friendlier appearance.
    /// Creates a more approachable and modern aesthetic compared to sharp corners.
    ///
    /// # Character Set
    /// ```text
    /// ╭─┬─╮
    /// ├─┼─┤
    /// ╰─┴─╯
    /// ```
    pub const ROUNDED: Lines<'static> = Lines {
        top_right: "╮",
        top_left: "╭",
        bottom_right: "╯",
        bottom_left: "╰",
        ..Self::LIGHT
    };
    /// Double-line box drawing characters with parallel lines
    ///
    /// Creates a formal, structured appearance with increased visual emphasis.
    /// Particularly effective for highlighting important sections or primary containers.
    ///
    /// # Character Set
    /// ```text
    /// ╔═╦═╗
    /// ╠═╬═╣
    /// ╚═╩═╝
    /// ```
    pub const DOUBLE: Lines<'static> = Lines {
        top_junction: "╦",
        bottom_junction: "╩",
        right_junction: "╣",
        left_junction: "╠",
        center: "╬",
        horizontal: "═",
        vertical: "║",
        top_right: "╗",
        top_left: "╔",
        bottom_right: "╝",
        bottom_left: "╚",

        ..Lines::LIGHT
    };

    /// Double-dashed box drawing characters
    ///
    /// Creates subtle, non-intrusive lines suitable for secondary content or dividers.
    /// The dashed pattern reduces visual weight while maintaining clear boundaries.
    ///
    /// # Character Set
    /// ```text
    /// ┌╌┬╌┐
    /// ├╌┼╌┤
    /// └╌┴╌┘
    /// ```
    pub const DASHED_DOUBLE: Lines<'static> = Lines {
        vertical: "╎",
        horizontal: "╌",
        ..Lines::LIGHT
    };
    /// Bold single-dashed box drawing characters
    ///
    /// Combines the emphasis of bold weight with the lighter visual presence of dashes.
    /// Useful for highlighted sections that shouldn't dominate the visual hierarchy.
    ///
    /// # Character Set
    /// ```text
    /// ┏╍┳╍┓
    /// ┣╍╋╍┫
    /// ┗╍┻╍┛
    /// ```
    pub const DASHED_DOUBLE_BOLD: Lines<'static> = Lines {
        vertical: "╏",
        horizontal: "╍",
        ..Lines::BOLD
    };

    /// Triple-dashed box drawing characters with three-dash segments
    ///
    /// Creates a delicate, decorative pattern with lighter visual weight than double-dash.
    /// Ideal for subtle divisions or background elements that shouldn't draw attention.
    ///
    /// # Character Set
    /// ```text
    /// ┌┄┬┄┐
    /// ├┄┼┄┤
    /// └┄┴┄┘
    /// ```
    pub const DASHED_TRIPLE: Lines<'static> = Lines {
        vertical: "┆",
        horizontal: "┄",
        ..Lines::LIGHT
    };

    /// Bold triple-dashed box drawing characters
    ///
    /// Provides emphasis through bold weight while maintaining an airy, segmented appearance.
    /// Balances visibility with subtlety for intermediate visual hierarchy.
    ///
    /// # Character Set
    /// ```text
    /// ┏┅┳┅┓
    /// ┣┅╋┅┫
    /// ┗┅┻┅┛
    /// ```
    pub const DASHED_TRIPLE_BOLD: Lines<'static> = Lines {
        vertical: "┇",
        horizontal: "┅",
        ..Lines::BOLD
    };

    /// Quadruple-dashed box drawing characters with four-dash segments
    ///
    /// Creates the most subtle dashed pattern, ideal for minimal visual interference.
    /// Perfect for backgrounds, spatials, or guidelines that should remain unobtrusive.
    ///
    /// # Character Set
    /// ```text
    /// ┌┈┬┈┐
    /// ├┈┼┈┤
    /// └┈┴┈┘
    /// ```
    pub const DASHED_QUADRUPLE: Lines<'static> = Lines {
        vertical: "┊",
        horizontal: "┈",
        ..Lines::LIGHT
    };
    /// Bold quadruple-dashed box drawing characters
    ///
    /// Balances prominence with segmentation for distinctive framing.
    /// Provides visual interest through pattern while maintaining structural emphasis.
    ///
    /// # Character Set
    /// ```text
    /// ┏┉┳┉┓
    /// ┣┉╋┉┫
    /// ┗┉┻┉┛
    /// ```
    ///
    /// # Available Characters
    /// - Bold Quadruple Dashed Edges: `┉` (horizontal) `┋` (vertical)
    /// - Corners and Junctions: inherited from bold lines
    pub const DASHED_QUADRUPLE_BOLD: Lines<'static> = Lines {
        vertical: "┋",
        horizontal: "┉",
        ..Lines::BOLD
    };

    // Mixed

    /// Light horizontal / bold vertical
    ///
    /// Creates strong vertical divisions while maintaining subtle horizontal separations.
    /// Perfect for tables or layouts where columns are the primary organizational unit.
    ///
    /// # Character Set
    /// ```text
    /// ┎─┰─┒
    /// ┠─╂─┨
    /// ┖─┸─┚
    /// ```
    pub const LIGHT_HORIZONTAL_BOLD_VERTICAL: Lines<'static> = Lines {
        horizontal: Lines::LIGHT.horizontal,
        vertical: Lines::BOLD.vertical,

        top_left: "┎",
        top_right: "┒",
        bottom_left: "┖",
        bottom_right: "┚",

        top_junction: "┰",
        bottom_junction: "┸",
        left_junction: "┠",
        right_junction: "┨",

        up: Lines::BOLD.up,
        right: Lines::LIGHT.right,
        left: Lines::BOLD.left,
        down: Lines::LIGHT.down,

        center: "╂",

        cross: "╳",
        diagonal_left: "╱",
        diagonal_right: "╲",
    };

    /// Bold horizontal / light vertical
    ///
    /// Creates strong horizontal divisions while maintaining subtle vertical separations.
    /// Perfect for tables or layouts where rows are the primary organizational unit.
    ///
    /// # Character Set
    /// ```text
    /// ┍━┯━┑
    /// ┝━┿━┥
    /// ┕━┷━┙
    /// ```
    pub const BOLD_HORIZONTAL_LIGHT_VERTICAL: Lines<'static> = Lines {
        horizontal: Lines::BOLD.horizontal,
        vertical: Lines::LIGHT.vertical,

        top_left: "┍",
        top_right: "┑",
        bottom_left: "┕",
        bottom_right: "┙",

        top_junction: "┯",
        bottom_junction: "┷",
        left_junction: "┝",
        right_junction: "┥",

        center: "┿",

        up: Lines::LIGHT.up,
        right: Lines::BOLD.right,
        left: Lines::BOLD.left,
        down: Lines::LIGHT.down,

        ..Lines::LIGHT
    };

    /// Single horizontal / double vertical
    ///
    /// # Character Set
    /// ```text
    /// ╓─╥─╖
    /// ╟─╫─╢
    /// ╙─╨─╜
    /// ```
    pub const SINGLE_HORIZONTAL_DOUBLE_VERTICAL: Lines<'static> = Lines {
        horizontal: Lines::LIGHT.horizontal,
        vertical: Lines::DOUBLE.vertical,

        top_left: "╓",
        top_right: "╖",
        bottom_left: "╙",
        bottom_right: "╜",

        top_junction: "╥",
        bottom_junction: "╨",
        left_junction: "╟",
        right_junction: "╢",

        center: "╫",

        ..Lines::LIGHT
    };

    /// Double horizontal / single vertical
    ///
    /// # Character Set
    /// ```text
    /// ╒═╤═╕
    /// ╞═╪═╡
    /// ╘═╧═╛
    /// ```
    pub const DOUBLE_HORIZONTAL_SINGLE_VERTICAL: Lines<'static> = Lines {
        horizontal: Lines::DOUBLE.horizontal,
        vertical: Lines::LIGHT.vertical,

        top_left: "╒",
        top_right: "╕",
        bottom_left: "╘",
        bottom_right: "╛",

        top_junction: "╤",
        bottom_junction: "╧",
        left_junction: "╞",
        right_junction: "╡",

        center: "╪",

        ..Lines::LIGHT
    };
}
impl Default for Lines<'static> {
    fn default() -> Self {
        Lines::LIGHT
    }
}
/// Visual line style
///
/// Each variant maps to a concrete set of Unicode (or ASCII) characters that
/// are used for the top, bottom, left, right, corners and edge centers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineStyle {
    /// Standard single-line box drawing characters
    ///
    /// Creates clean, lightweight lines suitable for general use in terminal UIs.
    /// These are the most commonly used box-drawing characters with minimal visual weight.
    ///
    /// # Character Set
    /// ```text
    /// ┌─┬─┐
    /// ├─┼─┤
    /// └─┴─┘
    /// ```
    #[default]
    Light,
    /// Double-line box drawing characters with parallel lines
    ///
    /// Creates a formal, structured appearance with increased visual emphasis.
    /// Particularly effective for highlighting important sections or primary containers.
    ///
    /// # Character Set
    /// ```text
    /// ╔═╦═╗
    /// ╠═╬═╣
    /// ╚═╩═╝
    /// ```
    Bold,
    /// Rounded corner box drawing characters with smooth curves
    ///
    /// Combines normal line edges with softened corners for a friendlier appearance.
    /// Creates a more approachable and modern aesthetic compared to sharp corners.
    ///
    /// # Character Set
    /// ```text
    /// ╭─┬─╮
    /// ├─┼─┤
    /// ╰─┴─╯
    /// ```
    Rounded,
    /// Double-line box drawing characters with parallel lines
    ///
    /// Creates a formal, structured appearance with increased visual emphasis.
    /// Particularly effective for highlighting important sections or primary containers.
    ///
    /// # Character Set
    /// ```text
    /// ╔═╦═╗
    /// ╠═╬═╣
    /// ╚═╩═╝
    /// ```
    Double,
    /// Single-dashed box drawing characters with double-dash segments
    ///
    /// Creates subtle, non-intrusive lines suitable for secondary content or dividers.
    /// The dashed pattern reduces visual weight while maintaining clear boundaries.
    ///
    /// # Character Set
    /// ```text
    /// ┌╌┬╌┐
    /// ├╌┼╌┤
    /// └╌┴╌┘
    /// ```
    Dashed,
    /// Bold single-dashed box drawing characters
    ///
    /// Combines the emphasis of bold weight with the lighter visual presence of dashes.
    /// Useful for highlighted sections that shouldn't dominate the visual hierarchy.
    ///
    /// # Character Set
    /// ```text
    /// ┏╍┳╍┓
    /// ┣╍╋╍┫
    /// ┗╍┻╍┛
    /// ```
    DashedBold,
    /// Triple-dashed box drawing characters with three-dash segments
    ///
    /// Creates a delicate, decorative pattern with lighter visual weight than double-dash.
    /// Ideal for subtle divisions or background elements that shouldn't draw attention.
    ///
    /// # Character Set
    /// ```text
    /// ┌┄┬┄┐
    /// ├┄┼┄┤
    /// └┄┴┄┘
    /// ```
    DashedTriple,

    /// Bold triple-dashed box drawing characters
    ///
    /// Provides emphasis through bold weight while maintaining an airy, segmented appearance.
    /// Balances visibility with subtlety for intermediate visual hierarchy.
    ///
    /// # Character Set
    /// ```text
    /// ┏┅┳┅┓
    /// ┣┅╋┅┫
    /// ┗┅┻┅┛
    /// ```
    DashedTripleBold,

    /// Quadruple-dashed box drawing characters with four-dash segments
    ///
    /// Creates the most subtle dashed pattern, ideal for minimal visual interference.
    /// Perfect for backgrounds, spatials, or guidelines that should remain unobtrusive.
    ///
    /// # Character Set
    /// ```text
    /// ┌┈┬┈┐
    /// ├┈┼┈┤
    /// └┈┴┈┘
    /// ```
    DashedQuadruple,
    /// Bold quadruple-dashed box drawing characters
    ///
    /// Balances prominence with segmentation for distinctive framing.
    /// Provides visual interest through pattern while maintaining structural emphasis.
    ///
    /// # Character Set
    /// ```text
    /// ┏┉┳┉┓
    /// ┣┉╋┉┫
    /// ┗┉┻┉┛
    /// ```
    DashedQuadrupleBold,
    /// Light horizontal / bold vertical
    ///
    /// Creates strong vertical divisions while maintaining subtle horizontal separations.
    /// Perfect for tables or layouts where columns are the primary organizational unit.
    ///
    /// # Character Set
    /// ```text
    /// ┎─┰─┒
    /// ┠─╂─┨
    /// ┖─┸─┚
    /// ```
    LightHorizontalBoldVertical,
    /// Bold horizontal / light vertical
    ///
    /// Creates strong horizontal divisions while maintaining subtle vertical separations.
    /// Perfect for tables or layouts where rows are the primary organizational unit.
    ///
    /// # Character Set
    /// ```text
    /// ┍━┯━┑
    /// ┝━┿━┥
    /// ┕━┷━┙
    /// ```
    BoldHorizontalLightVertical,
    /// Single horizontal / double vertical
    ///
    /// # Character Set
    /// ```text
    /// ╓─╥─╖
    /// ╟─╫─╢
    /// ╙─╨─╜
    /// ```
    SingleHorizontalDoubleVertical,
    /// Double horizontal / single vertical
    ///
    /// # Character Set
    /// ```text
    /// ╒═╤═╕
    /// ╞═╪═╡
    /// ╘═╧═╛
    /// ```
    DoubleHorizontalSingleVertical,
}

impl LineStyle {
    /// Convert line style into a [`Lines`].
    pub fn as_line(self) -> Lines<'static> {
        match self {
            LineStyle::Light => Lines::LIGHT,
            LineStyle::Bold => Lines::BOLD,
            LineStyle::Rounded => Lines::ROUNDED,
            LineStyle::Double => Lines::DOUBLE,
            LineStyle::Dashed => Lines::DASHED_DOUBLE,
            LineStyle::DashedBold => Lines::DASHED_DOUBLE_BOLD,
            LineStyle::DashedTriple => Lines::DASHED_TRIPLE,
            LineStyle::DashedTripleBold => Lines::DASHED_TRIPLE_BOLD,
            LineStyle::DashedQuadruple => Lines::DASHED_QUADRUPLE,
            LineStyle::DashedQuadrupleBold => Lines::DASHED_QUADRUPLE_BOLD,
            LineStyle::LightHorizontalBoldVertical => Lines::LIGHT_HORIZONTAL_BOLD_VERTICAL,
            LineStyle::BoldHorizontalLightVertical => Lines::BOLD_HORIZONTAL_LIGHT_VERTICAL,
            LineStyle::SingleHorizontalDoubleVertical => Lines::SINGLE_HORIZONTAL_DOUBLE_VERTICAL,
            LineStyle::DoubleHorizontalSingleVertical => Lines::DOUBLE_HORIZONTAL_SINGLE_VERTICAL,
        }
    }
}
impl From<LineStyle> for Lines<'static> {
    fn from(style: LineStyle) -> Self {
        style.as_line()
    }
}

// pub const BOX_DRAWINGS_DOWN_HEAVY_AND_LEFT_UP_LIGHT: &str = "┧";
// pub const BOX_DRAWINGS_DOWN_HEAVY_AND_RIGHT_UP_LIGHT: &str = "┟";
// pub const BOX_DRAWINGS_DOWN_HEAVY_AND_UP_HORIZONTAL_LIGHT: &str = "╁";
// pub const BOX_DRAWINGS_DOWN_LIGHT_AND_LEFT_UP_HEAVY: &str = "┩";
// pub const BOX_DRAWINGS_DOWN_LIGHT_AND_RIGHT_UP_HEAVY: &str = "┡";
// pub const BOX_DRAWINGS_DOWN_LIGHT_AND_UP_HORIZONTAL_HEAVY: &str = "╇";
// pub const BOX_DRAWINGS_HEAVY_LEFT_AND_LIGHT_RIGHT: &str = "╾";
// pub const BOX_DRAWINGS_HEAVY_UP_AND_LIGHT_DOWN: &str = "╿";
// pub const BOX_DRAWINGS_LEFT_DOWN_HEAVY_AND_RIGHT_UP_LIGHT: &str = "╅";
// pub const BOX_DRAWINGS_LEFT_HEAVY_AND_RIGHT_DOWN_LIGHT: &str = "┭";
// pub const BOX_DRAWINGS_LEFT_HEAVY_AND_RIGHT_UP_LIGHT: &str = "┵";
// pub const BOX_DRAWINGS_LEFT_HEAVY_AND_RIGHT_VERTICAL_LIGHT: &str = "┽";
// pub const BOX_DRAWINGS_LEFT_LIGHT_AND_RIGHT_DOWN_HEAVY: &str = "┲";
// pub const BOX_DRAWINGS_LEFT_LIGHT_AND_RIGHT_UP_HEAVY: &str = "┺";
// pub const BOX_DRAWINGS_LEFT_LIGHT_AND_RIGHT_VERTICAL_HEAVY: &str = "╊";
// pub const BOX_DRAWINGS_LEFT_UP_HEAVY_AND_RIGHT_DOWN_LIGHT: &str = "╃";
// pub const BOX_DRAWINGS_LIGHT_LEFT_AND_HEAVY_RIGHT: &str = "╼";
// pub const BOX_DRAWINGS_LIGHT_UP_AND_HEAVY_DOWN: &str = "╽";
// pub const BOX_DRAWINGS_RIGHT_DOWN_HEAVY_AND_LEFT_UP_LIGHT: &str = "╆";
// pub const BOX_DRAWINGS_RIGHT_HEAVY_AND_LEFT_DOWN_LIGHT: &str = "┮";
// pub const BOX_DRAWINGS_RIGHT_HEAVY_AND_LEFT_UP_LIGHT: &str = "┶";
// pub const BOX_DRAWINGS_RIGHT_HEAVY_AND_LEFT_VERTICAL_LIGHT: &str = "┾";
// pub const BOX_DRAWINGS_RIGHT_LIGHT_AND_LEFT_DOWN_HEAVY: &str = "┱";
// pub const BOX_DRAWINGS_RIGHT_LIGHT_AND_LEFT_UP_HEAVY: &str = "┹";
// pub const BOX_DRAWINGS_RIGHT_LIGHT_AND_LEFT_VERTICAL_HEAVY: &str = "╉";
// pub const BOX_DRAWINGS_RIGHT_UP_HEAVY_AND_LEFT_DOWN_LIGHT: &str = "╄";
// pub const BOX_DRAWINGS_UP_HEAVY_AND_DOWN_HORIZONTAL_LIGHT: &str = "╀";
// pub const BOX_DRAWINGS_UP_HEAVY_AND_LEFT_DOWN_LIGHT: &str = "┦";
// pub const BOX_DRAWINGS_UP_HEAVY_AND_RIGHT_DOWN_LIGHT: &str = "┞";
// pub const BOX_DRAWINGS_UP_LIGHT_AND_DOWN_HORIZONTAL_HEAVY: &str = "╈";
// pub const BOX_DRAWINGS_UP_LIGHT_AND_LEFT_DOWN_HEAVY: &str = "┪";
// pub const BOX_DRAWINGS_UP_LIGHT_AND_RIGHT_DOWN_HEAVY: &str = "┢";
