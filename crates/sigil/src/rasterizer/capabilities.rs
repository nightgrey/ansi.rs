use bitflags::bitflags;

bitflags! {
    /// Terminal capability flags controlling which escape sequences the
    /// rasterizer is allowed to emit.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Capabilities: u16 {
        /// Cursor Horizontal Absolute (`\x1B[G`).
        const CHA           = 1 << 0;
        /// Vertical Position Absolute (`\x1B[d`).
        const VPA           = 1 << 1;
        /// Horizontal Position Absolute (`` \x1B[` ``).
        const HPA           = 1 << 2;
        /// Repeat Character (`\x1B[b`).
        const REP           = 1 << 3;
        /// Erase Character (`\x1B[X`).
        const ECH           = 1 << 4;
        /// DECSTBM scroll region (`\x1B[r`).
        const SCROLL_REGION = 1 << 5;
        /// Scroll Up/Down (`\x1B[S` / `\x1B[T`).
        const SCROLL        = 1 << 6;
        /// Synchronized output (mode 2026).
        const SYNC_OUTPUT   = 1 << 7;
        /// Insert Line (`\x1B[L`).
        const INSERT_LINE   = 1 << 10;
        /// Delete Line (`\x1B[M`).
        const DELETE_LINE   = 1 << 11;
        /// 24-bit true color.
        const TRUECOLOR     = 1 << 8;
        /// 256 indexed colors.
        const COLORS_256    = 1 << 9;
    }
}

impl Capabilities {
    /// Modern xterm-compatible terminal.
    pub const DEFAULT: Self = Self::CHA
        .union(Self::VPA)
        .union(Self::SCROLL)
        .union(Self::TRUECOLOR)
        .union(Self::COLORS_256);

    /// No extended capabilities — only basic CUP and relative moves.
    pub const MINIMAL: Self = Self::empty();

    /// Detect capabilities from environment variables.
    pub fn detect() -> Self {
        let mut caps = Self::CHA.union(Self::VPA).union(Self::SCROLL);

        if let Ok(colorterm) = std::env::var("COLORTERM") {
            match colorterm.as_str() {
                "truecolor" | "24bit" => {
                    caps |= Self::TRUECOLOR | Self::COLORS_256;
                }
                _ => {}
            }
        }

        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                caps |= Self::COLORS_256;
            }
            if term.contains("xterm") || term.contains("screen") || term.contains("tmux") {
                caps |= Self::CHA | Self::VPA;
            }
        }

        caps
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self::DEFAULT
    }
}
