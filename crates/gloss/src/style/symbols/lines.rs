use super::Symbol;

/// Line
///
/// Defines symbols for edges, junctions, crosses and diagonal segments.
pub struct Line {
    pub center: Symbol,
    pub horizontal: Symbol,
    pub vertical: Symbol,

    pub top_right: Symbol,
    pub top_left: Symbol,
    pub bottom_right: Symbol,
    pub bottom_left: Symbol,

    pub up: Symbol,
    pub right: Symbol,
    pub left: Symbol,
    pub down: Symbol,

    pub top_junction: Symbol,
    pub bottom_junction: Symbol,
    pub right_junction: Symbol,
    pub left_junction: Symbol,

    pub cross: Symbol,

    pub diagonal_left: Symbol,
    pub diagonal_right: Symbol,
}
impl Line {
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
    pub const LIGHT: Line = Line {
        top_junction: Symbol::new('┬'),
        bottom_junction: Symbol::new('┴'),
        right_junction: Symbol::new('┼'),
        left_junction: Symbol::new('├'),

        center: Symbol::new('┼'),

        horizontal: Symbol::new('─'),
        vertical: Symbol::new('│'),
        top_right: Symbol::new('┐'),
        top_left: Symbol::new('┌'),
        bottom_right: Symbol::new('┘'),
        bottom_left: Symbol::new('└'),

        up: Symbol::new('╵'),
        right: Symbol::new('╶'),
        left: Symbol::new('╴'),
        down: Symbol::new('╷'),

        cross: Symbol::new('╳'),

        diagonal_left: Symbol::new('╱'),
        diagonal_right: Symbol::new('╱'),
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
    pub const BOLD: Line = Line {
        top_junction: Symbol::new('┳'),
        bottom_junction: Symbol::new('┻'),
        right_junction: Symbol::new('┫'),
        left_junction: Symbol::new('┣'),

        center: Symbol::new('╋'),

        horizontal: Symbol::new('━'),
        vertical: Symbol::new('┃'),
        top_right: Symbol::new('┓'),
        top_left: Symbol::new('┏'),
        bottom_right: Symbol::new('┛'),
        bottom_left: Symbol::new('┗'),

        up: Symbol::new('╹'),
        right: Symbol::new('╺'),
        left: Symbol::new('╸'),
        down: Symbol::new('╻'),

        ..Line::LIGHT
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
    pub const ROUNDED: Line = Line {
        top_right: Symbol::new('╮'),
        top_left: Symbol::new('╭'),
        bottom_right: Symbol::new('╯'),
        bottom_left: Symbol::new('╰'),
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
    pub const DOUBLE: Line = Line {
        top_junction: Symbol::new('╦'),
        bottom_junction: Symbol::new('╩'),
        right_junction: Symbol::new('╣'),
        left_junction: Symbol::new('╠'),
        center: Symbol::new('╬'),
        horizontal: Symbol::new('═'),
        vertical: Symbol::new('║'),
        top_right: Symbol::new('╗'),
        top_left: Symbol::new('╔'),
        bottom_right: Symbol::new('╝'),
        bottom_left: Symbol::new('╚'),

        ..Line::LIGHT
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
    pub const DASHED_DOUBLE: Line = Line {
        vertical: Symbol::new('╎'),
        horizontal: Symbol::new('╌'),
        ..Line::LIGHT
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
    pub const DASHED_DOUBLE_BOLD: Line = Line {
        vertical: Symbol::new('╏'),
        horizontal: Symbol::new('╍'),
        ..Line::BOLD
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
    pub const DASHED_TRIPLE: Line = Line {
        vertical: Symbol::new('┆'),
        horizontal: Symbol::new('┄'),
        ..Line::LIGHT
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
    pub const DASHED_TRIPLE_BOLD: Line = Line {
        vertical: Symbol::new('┇'),
        horizontal: Symbol::new('┅'),
        ..Line::BOLD
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
    pub const DASHED_QUADRUPLE: Line = Line {
        vertical: Symbol::new('┊'),
        horizontal: Symbol::new('┈'),
        ..Line::LIGHT
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
    pub const DASHED_QUADRUPLE_BOLD: Line = Line {
        vertical: Symbol::new('┋'),
        horizontal: Symbol::new('┉'),
        ..Line::BOLD
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
    pub const LIGHT_HORIZONTAL_BOLD_VERTICAL: Line = Line {
        horizontal: Line::LIGHT.horizontal,
        vertical: Line::BOLD.vertical,

        top_left: Symbol::new('┎'),
        top_right: Symbol::new('┒'),
        bottom_left: Symbol::new('┖'),
        bottom_right: Symbol::new('┚'),

        top_junction: Symbol::new('┰'),
        bottom_junction: Symbol::new('┸'),
        left_junction: Symbol::new('┠'),
        right_junction: Symbol::new('┨'),

        up: Line::BOLD.up,
        right: Line::LIGHT.right,
        left: Line::BOLD.left,
        down: Line::LIGHT.down,

        center: Symbol::new('╂'),

        cross: Symbol::new('╳'),
        diagonal_left: Symbol::new('╱'),
        diagonal_right: Symbol::new('╲'),
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
    pub const BOLD_HORIZONTAL_LIGHT_VERTICAL: Line = Line {
        horizontal: Line::BOLD.horizontal,
        vertical: Line::LIGHT.vertical,

        top_left: Symbol::new('┍'),
        top_right: Symbol::new('┑'),
        bottom_left: Symbol::new('┕'),
        bottom_right: Symbol::new('┙'),

        top_junction: Symbol::new('┯'),
        bottom_junction: Symbol::new('┷'),
        left_junction: Symbol::new('┝'),
        right_junction: Symbol::new('┥'),

        center: Symbol::new('┿'),

        up: Line::LIGHT.up,
        right: Line::BOLD.right,
        left: Line::BOLD.left,
        down: Line::LIGHT.down,

        ..Line::LIGHT
    };

    /// Single horizontal / double vertical
    ///
    /// # Character Set
    /// ```text
    /// ╓─╥─╖
    /// ╟─╫─╢
    /// ╙─╨─╜
    /// ```
    pub const SINGLE_HORIZONTAL_DOUBLE_VERTICAL: Line = Line {
        horizontal: Line::LIGHT.horizontal,
        vertical: Line::DOUBLE.vertical,

        top_left: Symbol::new('╓'),
        top_right: Symbol::new('╖'),
        bottom_left: Symbol::new('╙'),
        bottom_right: Symbol::new('╜'),

        top_junction: Symbol::new('╥'),
        bottom_junction: Symbol::new('╨'),
        left_junction: Symbol::new('╟'),
        right_junction: Symbol::new('╢'),

        center: Symbol::new('╫'),

        ..Line::LIGHT
    };

    /// Double horizontal / single vertical
    ///
    /// # Character Set
    /// ```text
    /// ╒═╤═╕
    /// ╞═╪═╡
    /// ╘═╧═╛
    /// ```
    pub const DOUBLE_HORIZONTAL_SINGLE_VERTICAL: Line = Line {
        horizontal: Line::DOUBLE.horizontal,
        vertical: Line::LIGHT.vertical,

        top_left: Symbol::new('╒'),
        top_right: Symbol::new('╕'),
        bottom_left: Symbol::new('╘'),
        bottom_right: Symbol::new('╛'),

        top_junction: Symbol::new('╤'),
        bottom_junction: Symbol::new('╧'),
        left_junction: Symbol::new('╞'),
        right_junction: Symbol::new('╡'),

        center: Symbol::new('╪'),

        ..Line::LIGHT
    };
}

impl Line {
    pub const fn top(&self) -> char {
       self.horizontal.inner
    }

    pub const fn right(&self) -> char {
        self.vertical.inner
    }

    pub const fn bottom(&self) -> char {
        self.horizontal.inner
    }

    pub const fn left(&self) -> char {
        self.vertical.inner
    }
}

impl Default for Line {
    fn default() -> Self {
        Line::LIGHT
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
    /// Convert line style into a [`Line`].
    pub fn as_line(self) -> Line {
        match self {
            LineStyle::Light => Line::LIGHT,
            LineStyle::Bold => Line::BOLD,
            LineStyle::Rounded => Line::ROUNDED,
            LineStyle::Double => Line::DOUBLE,
            LineStyle::Dashed => Line::DASHED_DOUBLE,
            LineStyle::DashedBold => Line::DASHED_DOUBLE_BOLD,
            LineStyle::DashedTriple => Line::DASHED_TRIPLE,
            LineStyle::DashedTripleBold => Line::DASHED_TRIPLE_BOLD,
            LineStyle::DashedQuadruple => Line::DASHED_QUADRUPLE,
            LineStyle::DashedQuadrupleBold => Line::DASHED_QUADRUPLE_BOLD,
            LineStyle::LightHorizontalBoldVertical => Line::LIGHT_HORIZONTAL_BOLD_VERTICAL,
            LineStyle::BoldHorizontalLightVertical => Line::BOLD_HORIZONTAL_LIGHT_VERTICAL,
            LineStyle::SingleHorizontalDoubleVertical => Line::SINGLE_HORIZONTAL_DOUBLE_VERTICAL,
            LineStyle::DoubleHorizontalSingleVertical => Line::DOUBLE_HORIZONTAL_SINGLE_VERTICAL,
        }
    }
}
impl From<LineStyle> for Line {
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
