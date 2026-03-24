#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rule {
    pub line: char,
    pub major: char,
    pub minor: char,
    pub start: char,
    pub end: char,
}

impl Rule {
    pub const THIN: Self = Self {
        start: '├',
        end: '┤',
        line: '─',
        major: '┼',
        minor: '┴',
    };

    pub const BOLD: Self = Self {
        line: '━',
        major: '╋',
        minor: '┷',
        start: '┣',
        end: '┫',
    };

    pub const ASCII: Self = Self {
        line: '-',
        major: '+',
        minor: '|',
        start: '+',
        end: '+',
    };

    pub const DOTS: Self = Self {
        line: '·',
        major: '•',
        minor: '∘',
        start: '•',
        end: '•',
    };
}

impl Default for Rule {
    fn default() -> Self {
        Self::THIN
    }
}
