use super::Symbol;

/// Line
///
/// Defines symbols for edges, junctions, crosses and diagonal segments.
pub struct LineSymbols {
    pub center: Symbol<char>,
    pub horizontal: Symbol<char>,
    pub vertical: Symbol<char>,

    pub top_right: Symbol<char>,
    pub top_left: Symbol<char>,
    pub bottom_right: Symbol<char>,
    pub bottom_left: Symbol<char>,

    pub up: Symbol<char>,
    pub right: Symbol<char>,
    pub left: Symbol<char>,
    pub down: Symbol<char>,

    pub top_junction: Symbol<char>,
    pub bottom_junction: Symbol<char>,
    pub right_junction: Symbol<char>,
    pub left_junction: Symbol<char>,

    pub cross: Symbol<char>,

    pub diagonal_left: Symbol<char>,
    pub diagonal_right: Symbol<char>,
}
impl LineSymbols {
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
    pub const LIGHT: LineSymbols = LineSymbols {
        top_junction: Symbol { inner: '┬', width: 1 },
        bottom_junction: Symbol { inner: '┴', width: 1 },
        right_junction: Symbol { inner: '┼', width: 1 },
        left_junction: Symbol { inner: '├', width: 1 },

        center: Symbol { inner: '┼', width: 1 },

        horizontal: Symbol { inner: '─', width: 1 },
        vertical: Symbol { inner: '│', width: 1 },
        top_right: Symbol { inner: '┐', width: 1 },
        top_left: Symbol { inner: '┌', width: 1 },
        bottom_right: Symbol { inner: '┘', width: 1 },
        bottom_left: Symbol { inner: '└', width: 1 },

        up: Symbol { inner: '╵', width: 1 },
        right: Symbol { inner: '╶', width: 1 },
        left: Symbol { inner: '╴', width: 1 },
        down: Symbol { inner: '╷', width: 1 },

        cross: Symbol { inner: '╳', width: 1 },

        diagonal_left: Symbol { inner: '╱', width: 1 },
        diagonal_right: Symbol { inner: '╱', width: 1 },
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
    pub const BOLD: LineSymbols = LineSymbols {
        top_junction: Symbol { inner: '┳', width: 1 },
        bottom_junction: Symbol { inner: '┻', width: 1 },
        right_junction: Symbol { inner: '┫', width: 1 },
        left_junction: Symbol { inner: '┣', width: 1 },

        center: Symbol { inner: '╋', width: 1 },

        horizontal: Symbol { inner: '━', width: 1 },
        vertical: Symbol { inner: '┃', width: 1 },
        top_right: Symbol { inner: '┓', width: 1 },
        top_left: Symbol { inner: '┏', width: 1 },
        bottom_right: Symbol { inner: '┛', width: 1 },
        bottom_left: Symbol { inner: '┗', width: 1 },

        up: Symbol { inner: '╹', width: 1 },
        right: Symbol { inner: '╺', width: 1 },
        left: Symbol { inner: '╸', width: 1 },
        down: Symbol { inner: '╻', width: 1 },

        ..LineSymbols::LIGHT
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
    pub const ROUNDED: LineSymbols = LineSymbols {
        top_right: Symbol { inner: '╮', width: 1 },
        top_left: Symbol { inner: '╭', width: 1 },
        bottom_right: Symbol { inner: '╯', width: 1 },
        bottom_left: Symbol { inner: '╰', width: 1 },
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
    pub const DOUBLE: LineSymbols = LineSymbols {
        top_junction: Symbol { inner: '╦', width: 1 },
        bottom_junction: Symbol { inner: '╩', width: 1 },
        right_junction: Symbol { inner: '╣', width: 1 },
        left_junction: Symbol { inner: '╠', width: 1 },
        center: Symbol { inner: '╬', width: 1 },
        horizontal: Symbol { inner: '═', width: 1 },
        vertical: Symbol { inner: '║', width: 1 },
        top_right: Symbol { inner: '╗', width: 1 },
        top_left: Symbol { inner: '╔', width: 1 },
        bottom_right: Symbol { inner: '╝', width: 1 },
        bottom_left: Symbol { inner: '╚', width: 1 },

        ..LineSymbols::LIGHT
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
    pub const DASHED_DOUBLE: LineSymbols = LineSymbols {
        vertical: Symbol { inner: '╎', width: 1 },
        horizontal: Symbol { inner: '╌', width: 1 },
        ..LineSymbols::LIGHT
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
    pub const DASHED_DOUBLE_BOLD: LineSymbols = LineSymbols {
        vertical: Symbol { inner: '╏', width: 1 },
        horizontal: Symbol { inner: '╍', width: 1 },
        ..LineSymbols::BOLD
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
    pub const DASHED_TRIPLE: LineSymbols = LineSymbols {
        vertical: Symbol { inner: '┆', width: 1 },
        horizontal: Symbol { inner: '┄', width: 1 },
        ..LineSymbols::LIGHT
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
    pub const DASHED_TRIPLE_BOLD: LineSymbols = LineSymbols {
        vertical: Symbol { inner: '┇', width: 1 },
        horizontal: Symbol { inner: '┅', width: 1 },
        ..LineSymbols::BOLD
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
    pub const DASHED_QUADRUPLE: LineSymbols = LineSymbols {
        vertical: Symbol { inner: '┊', width: 1 },
        horizontal: Symbol { inner: '┈', width: 1 },
        ..LineSymbols::LIGHT
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
    pub const DASHED_QUADRUPLE_BOLD: LineSymbols = LineSymbols {
        vertical: Symbol { inner: '┋', width: 1 },
        horizontal: Symbol { inner: '┉', width: 1 },
        ..LineSymbols::BOLD
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
    pub const LIGHT_HORIZONTAL_BOLD_VERTICAL: LineSymbols = LineSymbols {
        horizontal: LineSymbols::LIGHT.horizontal,
        vertical: LineSymbols::BOLD.vertical,

        top_left: Symbol { inner: '┎', width: 1 },
        top_right: Symbol { inner: '┒', width: 1 },
        bottom_left: Symbol { inner: '┖', width: 1 },
        bottom_right: Symbol { inner: '┚', width: 1 },

        top_junction: Symbol { inner: '┰', width: 1 },
        bottom_junction: Symbol { inner: '┸', width: 1 },
        left_junction: Symbol { inner: '┠', width: 1 },
        right_junction: Symbol { inner: '┨', width: 1 },

        up: LineSymbols::BOLD.up,
        right: LineSymbols::LIGHT.right,
        left: LineSymbols::BOLD.left,
        down: LineSymbols::LIGHT.down,

        center: Symbol { inner: '╂', width: 1 },

        cross: Symbol { inner: '╳', width: 1 },
        diagonal_left: Symbol { inner: '╱', width: 1 },
        diagonal_right: Symbol { inner: '╲', width: 1 },
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
    pub const BOLD_HORIZONTAL_LIGHT_VERTICAL: LineSymbols = LineSymbols {
        horizontal: LineSymbols::BOLD.horizontal,
        vertical: LineSymbols::LIGHT.vertical,

        top_left: Symbol { inner: '┍', width: 1 },
        top_right: Symbol { inner: '┑', width: 1 },
        bottom_left: Symbol { inner: '┕', width: 1 },
        bottom_right: Symbol { inner: '┙', width: 1 },

        top_junction: Symbol { inner: '┯', width: 1 },
        bottom_junction: Symbol { inner: '┷', width: 1 },
        left_junction: Symbol { inner: '┝', width: 1 },
        right_junction: Symbol { inner: '┥', width: 1 },

        center: Symbol { inner: '┿', width: 1 },

        up: LineSymbols::LIGHT.up,
        right: LineSymbols::BOLD.right,
        left: LineSymbols::BOLD.left,
        down: LineSymbols::LIGHT.down,

        ..LineSymbols::LIGHT
    };

    /// Single horizontal / double vertical
    ///
    /// # Character Set
    /// ```text
    /// ╓─╥─╖
    /// ╟─╫─╢
    /// ╙─╨─╜
    /// ```
    pub const SINGLE_HORIZONTAL_DOUBLE_VERTICAL: LineSymbols = LineSymbols {
        horizontal: LineSymbols::LIGHT.horizontal,
        vertical: LineSymbols::DOUBLE.vertical,

        top_left: Symbol { inner: '╓', width: 1 },
        top_right: Symbol { inner: '╖', width: 1 },
        bottom_left: Symbol { inner: '╙', width: 1 },
        bottom_right: Symbol { inner: '╜', width: 1 },

        top_junction: Symbol { inner: '╥', width: 1 },
        bottom_junction: Symbol { inner: '╨', width: 1 },
        left_junction: Symbol { inner: '╟', width: 1 },
        right_junction: Symbol { inner: '╢', width: 1 },

        center: Symbol { inner: '╫', width: 1 },

        ..LineSymbols::LIGHT
    };

    /// Double horizontal / single vertical
    ///
    /// # Character Set
    /// ```text
    /// ╒═╤═╕
    /// ╞═╪═╡
    /// ╘═╧═╛
    /// ```
    pub const DOUBLE_HORIZONTAL_SINGLE_VERTICAL: LineSymbols = LineSymbols {
        horizontal: LineSymbols::DOUBLE.horizontal,
        vertical: LineSymbols::LIGHT.vertical,

        top_left: Symbol { inner: '╒', width: 1 },
        top_right: Symbol { inner: '╕', width: 1 },
        bottom_left: Symbol { inner: '╘', width: 1 },
        bottom_right: Symbol { inner: '╛', width: 1 },

        top_junction: Symbol { inner: '╤', width: 1 },
        bottom_junction: Symbol { inner: '╧', width: 1 },
        left_junction: Symbol { inner: '╞', width: 1 },
        right_junction: Symbol { inner: '╡', width: 1 },

        center: Symbol { inner: '╪', width: 1 },

        ..LineSymbols::LIGHT
    };
}

impl LineSymbols {
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

impl Default for LineSymbols {
    fn default() -> Self {
        LineSymbols::LIGHT
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
    /// Convert line style into a [`LineSymbols`].
    pub fn as_line(self) -> LineSymbols {
        match self {
            LineStyle::Light => LineSymbols::LIGHT,
            LineStyle::Bold => LineSymbols::BOLD,
            LineStyle::Rounded => LineSymbols::ROUNDED,
            LineStyle::Double => LineSymbols::DOUBLE,
            LineStyle::Dashed => LineSymbols::DASHED_DOUBLE,
            LineStyle::DashedBold => LineSymbols::DASHED_DOUBLE_BOLD,
            LineStyle::DashedTriple => LineSymbols::DASHED_TRIPLE,
            LineStyle::DashedTripleBold => LineSymbols::DASHED_TRIPLE_BOLD,
            LineStyle::DashedQuadruple => LineSymbols::DASHED_QUADRUPLE,
            LineStyle::DashedQuadrupleBold => LineSymbols::DASHED_QUADRUPLE_BOLD,
            LineStyle::LightHorizontalBoldVertical => LineSymbols::LIGHT_HORIZONTAL_BOLD_VERTICAL,
            LineStyle::BoldHorizontalLightVertical => LineSymbols::BOLD_HORIZONTAL_LIGHT_VERTICAL,
            LineStyle::SingleHorizontalDoubleVertical => LineSymbols::SINGLE_HORIZONTAL_DOUBLE_VERTICAL,
            LineStyle::DoubleHorizontalSingleVertical => LineSymbols::DOUBLE_HORIZONTAL_SINGLE_VERTICAL,
        }
    }
}
impl From<LineStyle> for LineSymbols {
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
