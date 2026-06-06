use crate::symbols::Symbol;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Block {
    pub one_eighth: Symbol,
    pub two_eighth: Symbol,
    pub three_eighth: Symbol,
    pub four_eighth: Symbol,
    pub five_eighth: Symbol,
    pub six_eighth: Symbol,
    pub seven_eighth: Symbol,
    pub eight_eighth: Symbol,
}

impl Block {
    pub const TOP: Block = Block {
        one_eighth: Symbol::new('▔'),
        two_eighth: Symbol::new(' '),
        three_eighth: Symbol::new(' '),
        four_eighth: Symbol::new('▀'),
        five_eighth: Symbol::new(' '),
        six_eighth: Symbol::new(' '),
        seven_eighth: Symbol::new(' '),
        eight_eighth: Symbol::new(' '),
    };
    pub const RIGHT: Block = Block {
        one_eighth: Symbol::new('▕'),
        two_eighth: Symbol::new('🮇'),
        three_eighth: Symbol::new('🮈'),
        four_eighth: Symbol::new('▐'),
        five_eighth: Symbol::new('🮉'),
        six_eighth: Symbol::new('🮊'),
        seven_eighth: Symbol::new('🮋'),
        eight_eighth: Symbol::new('█'),
    };
    pub const BOTTOM: Block = Block {
        one_eighth: Symbol::new('▁'),
        two_eighth: Symbol::new('▂'),
        three_eighth: Symbol::new('▃'),
        four_eighth: Symbol::new('▄'),
        five_eighth: Symbol::new('▅'),
        six_eighth: Symbol::new('▆'),
        seven_eighth: Symbol::new('▇'),
        eight_eighth: Symbol::new('█'),
    };
    pub const LEFT: Block = Block {
        one_eighth: Symbol::new('▏'),
        two_eighth: Symbol::new('▎'),
        three_eighth: Symbol::new('▍'),
        four_eighth: Symbol::new('▌'),
        five_eighth: Symbol::new('▋'),
        six_eighth: Symbol::new('▊'),
        seven_eighth: Symbol::new('▉'),
        eight_eighth: Symbol::new('█'),
    };

    pub const CORNER: Corner = Corner {
        top_left: Symbol::new('▘'),
        top_right: Symbol::new('▝'),
        bottom_left: Symbol::new('▖'),
        bottom_right: Symbol::new('▗'),
    };

    pub const BOLD_CORNER: Corner = Corner {
        top_left: Symbol::new('▛'),
        top_right: Symbol::new('▜'),
        bottom_left: Symbol::new('▙'),
        bottom_right: Symbol::new('▟'),
    };

    pub const DIAGONAL: Diagonal = Diagonal {
        left: Symbol::new('▚'),
        right: Symbol::new('▞'),
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

    pub const fn corners(&self) -> Corner {
        Self::CORNER
    }

    pub const fn bold_corners(&self) -> Corner {
        Self::BOLD_CORNER
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
pub struct Corner {
    pub top_left: Symbol,
    pub top_right: Symbol,
    pub bottom_left: Symbol,
    pub bottom_right: Symbol,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Diagonal {
    pub left: Symbol,
    pub right: Symbol,
}
