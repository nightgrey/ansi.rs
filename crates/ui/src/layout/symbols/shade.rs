use std::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum Shade {
    #[default]
    None,
    Light,
    Medium,
    Strong,
    Full,
}

impl Shade {
    pub const fn new(value: f32) -> Self {
        match value.clamp(0.0, 1.0) {
            ..=0.0 => Self::None,
            0.0..0.5 => Self::Light,
            0.5..0.75 => Self::Medium,
            0.75..1.0 => Self::Strong,
            _ => Self::Full,
        }
    }
    pub const fn lighten(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Light => Self::None,
            Self::Medium => Self::Light,
            Self::Strong => Self::Medium,
            Self::Full => Self::Strong,
        }
    }

    pub const fn darken(self) -> Self {
        match self {
            Self::None => Self::Light,
            Self::Light => Self::Medium,
            Self::Medium => Self::Strong,
            Self::Strong => Self::Full,
            Self::Full => Self::Full,
        }
    }

    pub const fn invert(self) -> Self {
        match self {
            Self::None => Self::Full,
            Self::Light => Self::Strong,
            Self::Medium => Self::Medium,
            Self::Strong => Self::Light,
            Self::Full => Self::None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        static SHADES: [&str; 5] = [" ", "░", "▒", "▓", "█"];

        SHADES[self as usize]
    }
}

impl From<Shade> for &str {
    fn from(shade: Shade) -> Self {
        shade.as_str()
    }
}

impl AsRef<str> for Shade {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Shade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
