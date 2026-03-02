use crate::Attribute;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum UnderlineStyle {
    None,
    #[default]
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

impl UnderlineStyle {
    pub const MAX: Attribute = Attribute::new(
        Attribute::UnderlineStyleSingle.bits()
            | Attribute::UnderlineStyleDouble.bits()
            | Attribute::UnderlineStyleCurly.bits()
            | Attribute::UnderlineStyleDotted.bits()
            | Attribute::UnderlineStyleDashed.bits(),
    );
}

impl From<UnderlineStyle> for Attribute {
    fn from(value: UnderlineStyle) -> Self {
        match value {
            UnderlineStyle::None => Attribute::UnderlineStyleNone,
            UnderlineStyle::Single => Attribute::UnderlineStyleSingle,
            UnderlineStyle::Double => Attribute::UnderlineStyleDouble,
            UnderlineStyle::Curly => Attribute::UnderlineStyleCurly,
            UnderlineStyle::Dotted => Attribute::UnderlineStyleDotted,
            UnderlineStyle::Dashed => Attribute::UnderlineStyleDashed,
        }
    }
}