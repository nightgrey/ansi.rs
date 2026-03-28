use crate::symbols::Symbol;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Blocks {
    pub one_eighth: Symbol<char>,
    pub two_eighth: Symbol<char>,
    pub three_eighth: Symbol<char>,
    pub four_eighth: Symbol<char>,
    pub five_eighth: Symbol<char>,
    pub six_eighth: Symbol<char>,
    pub seven_eighth: Symbol<char>,
    pub eight_eighth: Symbol<char>,
}

impl Blocks {
    pub const TOP: Blocks = Blocks {
        one_eighth: Symbol { inner: '▔', width: 1 },
        two_eighth: Symbol { inner: ' ', width: 1 },
        three_eighth: Symbol { inner: ' ', width: 1 },
        four_eighth: Symbol { inner: '▀', width: 1 },
        five_eighth: Symbol { inner: ' ', width: 1 },
        six_eighth: Symbol { inner: ' ', width: 1 },
        seven_eighth: Symbol { inner: ' ', width: 1 },
        eight_eighth: Symbol { inner: ' ', width: 1 },
    };
    pub const RIGHT: Blocks = Blocks {
        one_eighth: Symbol { inner: '▕', width: 1 },
        two_eighth: Symbol { inner: '🮇', width: 1 },
        three_eighth: Symbol { inner: '🮈', width: 1 },
        four_eighth: Symbol { inner: '▐', width: 1 },
        five_eighth: Symbol { inner: '🮉', width: 1 },
        six_eighth: Symbol { inner: '🮊', width: 1 },
        seven_eighth: Symbol { inner: '🮋', width: 1 },
        eight_eighth: Symbol { inner: '█', width: 1 },
    };
    pub const BOTTOM: Blocks = Blocks {
        one_eighth: Symbol { inner: '▁', width: 1 },
        two_eighth: Symbol { inner: '▂', width: 1 },
        three_eighth: Symbol { inner: '▃', width: 1 },
        four_eighth: Symbol { inner: '▄', width: 1 },
        five_eighth: Symbol { inner: '▅', width: 1 },
        six_eighth: Symbol { inner: '▆', width: 1 },
        seven_eighth: Symbol { inner: '▇', width: 1 },
        eight_eighth: Symbol { inner: '█', width: 1 },
    };
    pub const LEFT: Blocks = Blocks {
        one_eighth: Symbol { inner: '▏', width: 1 },
        two_eighth: Symbol { inner: '▎', width: 1 },
        three_eighth: Symbol { inner: '▍', width: 1 },
        four_eighth: Symbol { inner: '▌', width: 1 },
        five_eighth: Symbol { inner: '▋', width: 1 },
        six_eighth: Symbol { inner: '▊', width: 1 },
        seven_eighth: Symbol { inner: '▉', width: 1 },
        eight_eighth: Symbol { inner: '█', width: 1 },
    };

    pub const CORNERS: Corners = Corners {
        top_left: Symbol { inner: '▘', width: 1 },
        top_right: Symbol { inner: '▝', width: 1 },
        bottom_left: Symbol { inner: '▖', width: 1 },
        bottom_right: Symbol { inner: '▗', width: 1 },
    };

    pub const BOLD_CORNERS: Corners = Corners {
        top_left: Symbol { inner: '▛', width: 1 },
        top_right: Symbol { inner: '▜', width: 1 },
        bottom_left: Symbol { inner: '▙', width: 1 },
        bottom_right: Symbol { inner: '▟', width: 1 },
    };

    pub const DIAGONAL: Diagonal = Diagonal {
        left: Symbol { inner: '▚', width: 1 },
        right: Symbol { inner: '▞', width: 1 },
    };

    pub const fn top(&self) -> Self {
        Self::TOP
    }

    pub const fn right(&self) -> Self {
        Self::RIGHT
    }

    pub const fn bottom(&self) -> Self {
        Self::BOTTOM
    }

    pub const fn left(&self) -> Self {
        Self::LEFT
    }

    pub const fn corners(&self) -> Corners {
        Self::CORNERS
    }

    pub const fn bold_corners(&self) -> Corners {
        Self::BOLD_CORNERS
    }

    pub const fn diagonal(&self) -> Diagonal {
        Self::DIAGONAL
    }

    #[inline]
    pub const fn quarter(&self) -> char {
        self.four_eighth.symbol()
    }

    #[inline]
    pub const fn half(&self) -> char {
        self.eight_eighth.symbol()
    }

    #[inline]
    pub const fn three_quarter(&self) -> char {
        self.three_eighth.symbol()
    }

    #[inline]
    pub const fn full(&self) -> char {
        self.eight_eighth.symbol()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Corners {
    pub top_left: Symbol<char>,
    pub top_right: Symbol<char>,
    pub bottom_left: Symbol<char>,
    pub bottom_right: Symbol<char>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Diagonal {
    pub left: Symbol<char>,
    pub right: Symbol<char>,
}
