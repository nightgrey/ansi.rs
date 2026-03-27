
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum TextDecorationLine {
    #[default]
    None,
    Underline,
    LineThrough,
}

impl From<bool> for TextDecorationLine {
    fn from(value: bool) -> Self {
        if value { Self::Underline } else { Self::None }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

impl From<usize> for FontWeight {
    fn from(value: usize) -> Self {
        match value {
            100..=400 => Self::Normal,
            500..=800 => Self::Bold,
            _ => Self::Normal,
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}
