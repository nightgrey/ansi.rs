use std::io::Write;

// ── Cursor movement ─────────────────────────────────────────────────

/// CUP — cursor position (1-indexed).
#[inline]
pub fn cup(buf: &mut Vec<u8>, row: usize, col: usize) {
    let _ = write!(buf, "\x1B[{};{}H", row + 1, col + 1);
}

/// CUU — cursor up `n` rows.
#[inline]
pub fn cuu(buf: &mut Vec<u8>, n: usize) {
    if n == 1 {
        buf.extend_from_slice(b"\x1B[A");
    } else if n > 1 {
        let _ = write!(buf, "\x1B[{}A", n);
    }
}

/// CUD — cursor down `n` rows.
#[inline]
pub fn cud(buf: &mut Vec<u8>, n: usize) {
    if n == 1 {
        buf.extend_from_slice(b"\x1B[B");
    } else if n > 1 {
        let _ = write!(buf, "\x1B[{}B", n);
    }
}

/// CUF — cursor forward `n` columns.
#[inline]
pub fn cuf(buf: &mut Vec<u8>, n: usize) {
    if n == 1 {
        buf.extend_from_slice(b"\x1B[C");
    } else if n > 1 {
        let _ = write!(buf, "\x1B[{}C", n);
    }
}

/// CUB — cursor back `n` columns.
#[inline]
pub fn cub(buf: &mut Vec<u8>, n: usize) {
    if n == 1 {
        buf.extend_from_slice(b"\x1B[D");
    } else if n > 1 {
        let _ = write!(buf, "\x1B[{}D", n);
    }
}

/// CR — carriage return.
#[inline]
pub fn cr(buf: &mut Vec<u8>) {
    buf.push(b'\r');
}

// ── Erase ───────────────────────────────────────────────────────────

/// EL — erase to end of line.
#[inline]
pub fn el(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[K");
}

/// ED — erase to end of screen.
#[inline]
pub fn ed(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[J");
}

/// ED 2 — erase entire screen.
#[inline]
pub fn ed_all(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[2J");
}

// ── Screen modes ────────────────────────────────────────────────────

/// DECSET 1049 — enter alternate screen buffer.
#[inline]
pub fn enter_alt_screen(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[?1049h");
}

/// DECRST 1049 — exit alternate screen buffer.
#[inline]
pub fn exit_alt_screen(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[?1049l");
}

/// DECTCEM — show cursor.
#[inline]
pub fn show_cursor(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[?25h");
}

/// DECTCEM — hide cursor.
#[inline]
pub fn hide_cursor(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[?25l");
}

/// CUP(1,1) — home position.
#[inline]
pub fn home(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[H");
}

/// SGR 0 — reset all attributes.
#[inline]
pub fn sgr_reset(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[0m");
}

// ── Save / restore ──────────────────────────────────────────────────

/// DECSC — save cursor position.
#[inline]
pub fn save_cursor(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B7");
}

/// DECRC — restore cursor position.
#[inline]
pub fn restore_cursor(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B8");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cup_sequence() {
        let mut buf = Vec::new();
        cup(&mut buf, 0, 0);
        assert_eq!(buf, b"\x1B[1;1H");

        buf.clear();
        cup(&mut buf, 5, 10);
        assert_eq!(buf, b"\x1B[6;11H");
    }

    #[test]
    fn relative_moves_single() {
        let cases: &[(&str, fn(&mut Vec<u8>, usize), &[u8])] = &[
            ("cuu", cuu, b"\x1B[A"),
            ("cud", cud, b"\x1B[B"),
            ("cuf", cuf, b"\x1B[C"),
            ("cub", cub, b"\x1B[D"),
        ];
        for (name, f, expected) in cases {
            let mut buf = Vec::new();
            f(&mut buf, 1);
            assert_eq!(&buf, expected, "{name} n=1");
        }
    }

    #[test]
    fn relative_moves_multi() {
        let mut buf = Vec::new();
        cuu(&mut buf, 3);
        assert_eq!(buf, b"\x1B[3A");

        buf.clear();
        cuf(&mut buf, 10);
        assert_eq!(buf, b"\x1B[10C");
    }

    #[test]
    fn relative_moves_zero_is_noop() {
        let mut buf = Vec::new();
        cuu(&mut buf, 0);
        cud(&mut buf, 0);
        cuf(&mut buf, 0);
        cub(&mut buf, 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn erase_sequences() {
        let mut buf = Vec::new();
        el(&mut buf);
        assert_eq!(buf, b"\x1B[K");

        buf.clear();
        ed(&mut buf);
        assert_eq!(buf, b"\x1B[J");

        buf.clear();
        ed_all(&mut buf);
        assert_eq!(buf, b"\x1B[2J");
    }

    #[test]
    fn screen_mode_sequences() {
        let mut buf = Vec::new();
        enter_alt_screen(&mut buf);
        assert_eq!(buf, b"\x1B[?1049h");

        buf.clear();
        exit_alt_screen(&mut buf);
        assert_eq!(buf, b"\x1B[?1049l");
    }
}
