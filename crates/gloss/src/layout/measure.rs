use geometry::Size;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{Display, Element, ElementKind};

pub fn measure(
    known: taffy::Size<Option<f32>>,
    available: taffy::Size<taffy::AvailableSpace>,
    node: &Element<'_>,
) -> taffy::Size<f32> {
    match &node.kind {
        ElementKind::Span(text) => {
            let wrap_width = resolve_wrap_width(known.width, available.width, node.style.display);

            let size = measure_text_block(text, wrap_width, node.style.display);

            taffy::Size {
                width: size.width as f32,
                height: size.height as f32,
            }
        }

        // Non-leaf containers are sized by Taffy from children.
        ElementKind::Div => taffy::Size::ZERO,
    }
}

// If width is definite, wrap to it.
// If inline, never wrap.
fn resolve_wrap_width(
    known_width: Option<f32>,
    available_width: taffy::AvailableSpace,
    display: Display,
) -> Option<u32> {
    if matches!(display, Display::Inline) {
        return None;
    }

    if let Some(w) = known_width {
        return Some((w as u32).max(0));
    }

    match available_width {
        taffy::AvailableSpace::Definite(w) => Some(w.max(0.0) as u32),
        taffy::AvailableSpace::MinContent | taffy::AvailableSpace::MaxContent => None,
    }
}

pub fn measure_text_block(text: &str, wrap_width: Option<u32>, display: Display) -> Size {
    if text.is_empty() {
        return Size {
            width: 0,
            height: 0,
        };
    }

    let mut x = 0;
    let mut y = 1;
    let mut max_x = 0;

    for ch in text.chars() {
        if ch == '\n' && !matches!(display, Display::Inline) {
            max_x = max_x.max(x);
            x = 0;
            y += 1;
            continue;
        }

        let w = char_cell_width(ch, x, 4);

        if let Some(limit) = wrap_width {
            if limit > 0 && x > 0 && x + w > limit as usize {
                max_x = max_x.max(x);
                x = 0;
                y += 1;
            }
        }

        x += w;
        max_x = max_x.max(x);
    }

    Size {
        width: max_x,
        height: y,
    }
}

fn char_cell_width(ch: char, x: usize, tab_width: usize) -> usize {
    match ch {
        '\t' => {
            if tab_width == 0 {
                0
            } else {
                let next_tab_stop = ((x / tab_width) + 1) * tab_width;
                next_tab_stop.saturating_sub(x)
            }
        }
        _ => UnicodeWidthChar::width(ch).unwrap_or(0),
    }
}
