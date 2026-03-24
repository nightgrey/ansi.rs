#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Blocks {
    pub one_eighth: &'static str,
    pub two_eighth: &'static str,
    pub three_eighth: &'static str,
    pub four_eighth: &'static str,
    pub five_eighth: &'static str,
    pub six_eighth: &'static str,
    pub seven_eighth: &'static str,
    pub eight_eighth: &'static str,
}

impl Blocks {
    pub const TOP: Blocks = Blocks {
        one_eighth: "▔",
        two_eighth: " ",
        three_eighth: " ",
        four_eighth: "▀",
        five_eighth: " ",
        six_eighth: " ",
        seven_eighth: " ",
        eight_eighth: " ",
    };
    pub const RIGHT: Blocks = Blocks {
        one_eighth: "▕",
        two_eighth: "🮇",
        three_eighth: "🮈",
        four_eighth: "▐",
        five_eighth: "🮉",
        six_eighth: "🮊",
        seven_eighth: "🮋",
        eight_eighth: "█",
    };
    pub const BOTTOM: Blocks = Blocks {
        one_eighth: "▁",
        two_eighth: "▂",
        three_eighth: "▃",
        four_eighth: "▄",
        five_eighth: "▅",
        six_eighth: "▆",
        seven_eighth: "▇",
        eight_eighth: "█",
    };
    pub const LEFT: Blocks = Blocks {
        one_eighth: "▏",
        two_eighth: "▎",
        three_eighth: "▍",
        four_eighth: "▌",
        five_eighth: "▋",
        six_eighth: "▊",
        seven_eighth: "▉",
        eight_eighth: "█",
    };

    pub const CORNERS: Corners = Corners {
        top_left: "▘",
        top_right: "▝",
        bottom_left: "▖",
        bottom_right: "▗",
    };

    pub const BOLD_CORNERS: Corners = Corners {
        top_left: "▛",
        top_right: "▜",
        bottom_left: "▙",
        bottom_right: "▟",
    };

    pub const DIAGONAL: Diagonal = Diagonal {
        left: "▚",
        right: "▞",
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
    pub const fn quarter(&self) -> &'static str {
        self.four_eighth
    }

    #[inline]
    pub const fn half(&self) -> &'static str {
        self.eight_eighth
    }

    #[inline]
    pub const fn three_quarter(&self) -> &'static str {
        self.three_eighth
    }

    #[inline]
    pub const fn full(&self) -> &'static str {
        self.eight_eighth
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Corners {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Diagonal {
    pub left: &'static str,
    pub right: &'static str,
}
