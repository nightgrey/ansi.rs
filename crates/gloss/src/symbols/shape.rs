use std::ops::Deref;
use derive_more::Deref;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deref)]
pub struct Circle(&'static str);

impl Circle {
    pub const FILLED: Circle = Circle("●");
    pub const OUTLINED: Circle = Circle("○");
    pub const DOTTED: Circle = Circle("◌");
    pub const DOUBLE: Circle = Circle("◎");
}


#[derive(Debug, Clone, Copy, Eq, PartialEq, Deref)]
pub struct Diamond(&'static str);

impl Diamond {
    pub const FILLED: Diamond = Diamond("◆");
    pub const OUTLINED: Diamond = Diamond("◇");
    pub const SMALL: Diamond = Diamond("⋄");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub filled: &'static str,
    pub outline: &'static str,
}

impl Square {
    pub const DEFAULT: Self = Self {
        filled: "■",
        outline: "□",
    };
    pub const SMALL: Self = Self {
        filled: "▪",
        outline: "▫",
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Triangle {
    pub top: &'static str,
    pub left: &'static str,
    pub right: &'static str,
    pub bottom: &'static str,
}

impl Triangle {
    pub const FILLED: Self = Self {
        top: "▲",
        left: "◀",
        right: "▶",
        bottom: "▼",
    };
    pub const OUTLINED: Self = Self {
        top: "△",
        left: "◁",
        right: "▷",
        bottom: "▽",
    };
}
