use crate::symbols::Symbol;
use derive_more::Deref;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deref)]
pub struct Circle(Symbol);

impl Circle {
    pub const FILLED: Circle = Circle(Symbol::new('●'));
    pub const OUTLINED: Circle = Circle(Symbol::new('○'));
    pub const DOTTED: Circle = Circle(Symbol::new('◌'));
    pub const DOUBLE: Circle = Circle(Symbol::new('◎'));
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deref)]
pub struct Diamond(Symbol);

impl Diamond {
    pub const FILLED: Diamond = Diamond(Symbol::new('◆'));
    pub const OUTLINED: Diamond = Diamond(Symbol::new('◇'));
    pub const SMALL: Diamond = Diamond(Symbol::new('⋄'));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub filled: Symbol,
    pub outline: Symbol,
}

impl Square {
    pub const DEFAULT: Self = Self {
        filled: Symbol::new('■'),
        outline: Symbol::new('□'),
    };
    pub const SMALL: Self = Self {
        filled: Symbol::new('▪'),
        outline: Symbol::new('▫'),
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Triangle {
    pub top: Symbol,
    pub left: Symbol,
    pub right: Symbol,
    pub bottom: Symbol,
}

impl Triangle {
    pub const FILLED: Self = Self {
        top: Symbol::new('▲'),
        left: Symbol::new('◀'),
        right: Symbol::new('▶'),
        bottom: Symbol::new('▼'),
    };
    pub const OUTLINED: Self = Self {
        top: Symbol::new('△'),
        left: Symbol::new('◁'),
        right: Symbol::new('▷'),
        bottom: Symbol::new('▽'),
    };
}
