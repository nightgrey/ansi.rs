use crate::symbols::Symbol;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rule {
    pub line: Symbol,
    pub major: Symbol,
    pub minor: Symbol,
    pub start: Symbol,
    pub end: Symbol,
}

impl Rule {
    pub const THIN: Self = Self {
        start: Symbol { inner: '├', width: 1 },
        end: Symbol { inner: '┤', width: 1 },
        line: Symbol { inner: '─', width: 1 },
        major: Symbol { inner: '┼', width: 1 },
        minor: Symbol { inner: '┴', width: 1 },
    };

    pub const BOLD: Self = Self {
        line: Symbol { inner: '━', width: 1 },
        major: Symbol { inner: '╋', width: 1 },
        minor: Symbol { inner: '┷', width: 1 },
        start: Symbol { inner: '┣', width: 1 },
        end: Symbol { inner: '┫', width: 1 },
    };

    pub const ASCII: Self = Self {
        line: Symbol { inner: '-', width: 1 },
        major: Symbol { inner: '+', width: 1 },
        minor: Symbol { inner: '|', width: 1 },
        start: Symbol { inner: '+', width: 1 },
        end: Symbol { inner: '+', width: 1 },
    };

    pub const DOTS: Self = Self {
        line: Symbol { inner: '·', width: 1 },
        major: Symbol { inner: '•', width: 1 },
        minor: Symbol { inner: '∘', width: 1 },
        start: Symbol { inner: '•', width: 1 },
        end: Symbol { inner: '•', width: 1 },
    };
}

impl Default for Rule {
    fn default() -> Self {
        Self::THIN
    }
}
