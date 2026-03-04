use geometry::Row;

use crate::buffer::{Buffer, Cell};

use super::capabilities::Capabilities;
use super::cursor::Cursor;
use super::sequences as seq;

/// Diff a single line between `buffer` (new) and `prev` (old), emitting only
/// the changed cells. Uses left→right and right→left scanning to find the
/// minimal dirty region, plus a trailing EL optimization.
pub(crate) fn transform_line(
    buf: &mut Vec<u8>,
    cursor: &mut Cursor,
    buffer: &Buffer,
    prev: &Buffer,
    y: usize,
    width: usize,
    caps: Capabilities,
) {
    let new_row: &[Cell] = &buffer[Row(y)];
    let old_row: &[Cell] = &prev[Row(y)];

    // Scan left→right for first differing cell.
    let first = match (0..width).find(|&x| new_row[x] != old_row[x]) {
        Some(col) => col,
        None => return, // Entire line is identical.
    };

    // Scan right→left for last differing cell.
    let last = (0..width)
        .rev()
        .find(|&x| new_row[x] != old_row[x])
        .unwrap();

    // Trailing EL optimization: within the diff range [first, last], find the
    // last cell that has non-empty new content. If the tail of the diff range
    // consists of empty new cells (replacing old content), we can use EL
    // instead of emitting spaces.
    let last_content_in_range = (first..=last)
        .rev()
        .find(|&x| !new_row[x].is_empty());

    let (emit_end, need_eol) = match last_content_in_range {
        Some(lc) if lc < last => (lc, true),
        _ => (last, false),
    };

    // Move cursor to start of changed region.
    cursor.move_to(buf, y, first, caps);

    // Emit changed cells from first through emit_end.
    let mut col = first;
    while col <= emit_end {
        let cell = &new_row[col];
        put_cell(buf, cursor, buffer, cell);
        let w = cell.columns() as usize;
        col += w;
        cursor.col += w;

        // REP optimization: if this cell is single-width, single-byte content
        // and followed by identical cells, use REP to repeat it.
        if caps.contains(Capabilities::REP) && w == 1 {
            let s = cell.as_str(&buffer.arena);
            if s.len() == 1 || cell.is_empty() {
                let mut rep_count = 0usize;
                while col + rep_count <= emit_end && new_row[col + rep_count] == *cell {
                    rep_count += 1;
                }
                // REP threshold: `\x1B[Nb` is 4+ bytes, so only worth it for 4+.
                if rep_count >= 4 {
                    seq::rep(buf, rep_count);
                    col += rep_count;
                    cursor.col += rep_count;
                }
            }
        }
    }

    // Clear to end of line.
    if need_eol {
        cursor.reset_pen(buf);
        seq::el(buf);
    }
}

/// Like `transform_line` but uses relative-only cursor movement (no CUP/VPA/CHA).
/// Used in inline mode where absolute screen position is unknown.
pub(crate) fn transform_line_relative(
    buf: &mut Vec<u8>,
    cursor: &mut Cursor,
    buffer: &Buffer,
    prev: &Buffer,
    y: usize,
    width: usize,
    caps: Capabilities,
) {
    let new_row: &[Cell] = &buffer[Row(y)];
    let old_row: &[Cell] = &prev[Row(y)];

    // Scan left→right for first differing cell.
    let first = match (0..width).find(|&x| new_row[x] != old_row[x]) {
        Some(col) => col,
        None => return,
    };

    // Scan right→left for last differing cell.
    let last = (0..width)
        .rev()
        .find(|&x| new_row[x] != old_row[x])
        .unwrap();

    let last_content_in_range = (first..=last)
        .rev()
        .find(|&x| !new_row[x].is_empty());

    let (emit_end, need_eol) = match last_content_in_range {
        Some(lc) if lc < last => (lc, true),
        _ => (last, false),
    };

    // Relative-only cursor movement.
    cursor.move_to_relative(buf, y, first);

    let mut col = first;
    while col <= emit_end {
        let cell = &new_row[col];
        put_cell(buf, cursor, buffer, cell);
        let w = cell.columns() as usize;
        col += w;
        cursor.col += w;

        if caps.contains(Capabilities::REP) && w == 1 {
            let s = cell.as_str(&buffer.arena);
            if s.len() == 1 || cell.is_empty() {
                let mut rep_count = 0usize;
                while col + rep_count <= emit_end && new_row[col + rep_count] == *cell {
                    rep_count += 1;
                }
                if rep_count >= 4 {
                    seq::rep(buf, rep_count);
                    col += rep_count;
                    cursor.col += rep_count;
                }
            }
        }
    }

    if need_eol {
        cursor.reset_pen(buf);
        seq::el(buf);
    }
}

/// Write a single cell's content, updating the pen first.
#[inline]
fn put_cell(buf: &mut Vec<u8>, cursor: &mut Cursor, buffer: &Buffer, cell: &Cell) {
    cursor.update_pen(buf, cell.style());

    if cell.is_empty() {
        buf.push(b' ');
    } else {
        let s = cell.as_str(&buffer.arena);
        buf.extend_from_slice(s.as_bytes());
    }
}
