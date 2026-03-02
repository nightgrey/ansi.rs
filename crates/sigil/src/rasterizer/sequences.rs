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

/// CHA — Cursor Horizontal Absolute (1-indexed).
#[inline]
pub fn cha(buf: &mut Vec<u8>, col: usize) {
    let _ = write!(buf, "\x1B[{}G", col + 1);
}

/// VPA — Vertical Position Absolute (1-indexed).
#[inline]
pub fn vpa(buf: &mut Vec<u8>, row: usize) {
    let _ = write!(buf, "\x1B[{}d", row + 1);
}

/// HPA — Horizontal Position Absolute (1-indexed).
#[inline]
pub fn hpa(buf: &mut Vec<u8>, col: usize) {
    let _ = write!(buf, "\x1B[{}`", col + 1);
}

// ── Erase ───────────────────────────────────────────────────────────

/// EL — erase to end of line.
#[inline]
pub fn el(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[K");
}

/// EL 1 — erase to start of line.
#[inline]
pub fn el_left(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[1K");
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

/// REP — repeat the preceding character `n` times.
#[inline]
pub fn rep(buf: &mut Vec<u8>, n: usize) {
    if n > 0 {
        let _ = write!(buf, "\x1B[{}b", n);
    }
}

/// ECH — erase `n` characters (without moving cursor).
#[inline]
pub fn ech(buf: &mut Vec<u8>, n: usize) {
    if n > 0 {
        let _ = write!(buf, "\x1B[{}X", n);
    }
}

// ── Scroll ──────────────────────────────────────────────────────────

/// DECSTBM — set scrolling region (1-indexed, inclusive).
#[inline]
pub fn decstbm(buf: &mut Vec<u8>, top: usize, bottom: usize) {
    let _ = write!(buf, "\x1B[{};{}r", top + 1, bottom + 1);
}

/// Reset scrolling region to full screen.
#[inline]
pub fn decstbm_reset(buf: &mut Vec<u8>) {
    buf.extend_from_slice(b"\x1B[r");
}

/// SU — scroll up `n` lines.
#[inline]
pub fn su(buf: &mut Vec<u8>, n: usize) {
    if n == 1 {
        buf.extend_from_slice(b"\x1B[S");
    } else if n > 1 {
        let _ = write!(buf, "\x1B[{}S", n);
    }
}

/// SD — scroll down `n` lines.
#[inline]
pub fn sd(buf: &mut Vec<u8>, n: usize) {
    if n == 1 {
        buf.extend_from_slice(b"\x1B[T");
    } else if n > 1 {
        let _ = write!(buf, "\x1B[{}T", n);
    }
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

// ── Cost helpers ────────────────────────────────────────────────────

/// Number of decimal digits in `n` (minimum 1).
#[inline]
pub fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut v = n;
    while v > 0 {
        count += 1;
        v /= 10;
    }
    count
}

/// Byte length of a CUP sequence `\x1B[{row+1};{col+1}H`.
#[inline]
pub fn cup_len(row: usize, col: usize) -> usize {
    // \x1B [ row+1 ; col+1 H
    2 + digit_count(row + 1) + 1 + digit_count(col + 1) + 1
}

/// Byte length of a VPA sequence `\x1B[{row+1}d`.
#[inline]
pub fn vpa_len(row: usize) -> usize {
    // \x1B [ row+1 d
    2 + digit_count(row + 1) + 1
}

/// Byte length of a CHA sequence `\x1B[{col+1}G`.
#[inline]
pub fn cha_len(col: usize) -> usize {
    // \x1B [ col+1 G
    2 + digit_count(col + 1) + 1
}

/// Byte length of a relative movement sequence `\x1B[{n}X` (A/B/C/D).
/// Special case: n=1 produces `\x1B[X` (3 bytes), n=0 produces 0.
#[inline]
pub fn relative_len(n: usize) -> usize {
    match n {
        0 => 0,
        1 => 3, // \x1B[X
        _ => 2 + digit_count(n) + 1, // \x1B[nX
    }
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

    #[test]
    fn cha_sequence() {
        let mut buf = Vec::new();
        cha(&mut buf, 0);
        assert_eq!(buf, b"\x1B[1G");

        buf.clear();
        cha(&mut buf, 9);
        assert_eq!(buf, b"\x1B[10G");
    }

    #[test]
    fn vpa_sequence() {
        let mut buf = Vec::new();
        vpa(&mut buf, 0);
        assert_eq!(buf, b"\x1B[1d");

        buf.clear();
        vpa(&mut buf, 4);
        assert_eq!(buf, b"\x1B[5d");
    }

    #[test]
    fn el_left_sequence() {
        let mut buf = Vec::new();
        el_left(&mut buf);
        assert_eq!(buf, b"\x1B[1K");
    }

    #[test]
    fn scroll_sequences() {
        let mut buf = Vec::new();
        su(&mut buf, 1);
        assert_eq!(buf, b"\x1B[S");

        buf.clear();
        su(&mut buf, 3);
        assert_eq!(buf, b"\x1B[3S");

        buf.clear();
        sd(&mut buf, 1);
        assert_eq!(buf, b"\x1B[T");

        buf.clear();
        sd(&mut buf, 5);
        assert_eq!(buf, b"\x1B[5T");
    }

    #[test]
    fn decstbm_sequence() {
        let mut buf = Vec::new();
        decstbm(&mut buf, 0, 23);
        assert_eq!(buf, b"\x1B[1;24r");

        buf.clear();
        decstbm_reset(&mut buf);
        assert_eq!(buf, b"\x1B[r");
    }

    #[test]
    fn cost_helpers() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(100), 3);

        // CUP(0,0) = "\x1B[1;1H" = 6 bytes
        assert_eq!(cup_len(0, 0), 6);
        // CUP(9,99) = "\x1B[10;100H" = 9 bytes
        assert_eq!(cup_len(9, 99), 9);

        assert_eq!(relative_len(0), 0);
        assert_eq!(relative_len(1), 3); // \x1B[A
        assert_eq!(relative_len(5), 4); // \x1B[5A
        assert_eq!(relative_len(10), 5); // \x1B[10A

        assert_eq!(vpa_len(0), 4); // \x1B[1d
        assert_eq!(cha_len(0), 4); // \x1B[1G
    }
}
