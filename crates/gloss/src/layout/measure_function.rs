use geometry::Size;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{Display, Element, ElementKind};

const DEFAULT_TAB_WIDTH: usize = 4;

pub fn measure(
    known: taffy::Size<Option<f32>>,
    available: taffy::Size<taffy::AvailableSpace>,
    node: &Element<'_>,
) -> taffy::Size<f32> {
    match &node.kind {
        ElementKind::Span(text) => {
            let wrap_width = resolve_wrap_width(known.width, available.width, node.layout.display);

            let size = measure_text_block(text, wrap_width, node.layout.display);

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
        return Some(w.floor().max(0.0) as u32);
    }

    match available_width {
        taffy::AvailableSpace::Definite(w) => Some(w.floor().max(0.0) as u32),
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

    let mut x: usize = 0;
    let mut lines: usize = 1;
    let mut max_x: usize = 0;

    for cluster in text.graphemes(true) {
        if cluster == "\n" && !matches!(display, Display::Inline) {
            max_x = max_x.max(x);
            x = 0;
            lines += 1;
            continue;
        }

        let w = grapheme_cell_width(cluster, x, DEFAULT_TAB_WIDTH);

        if let Some(limit) = wrap_width {
            if limit > 0 && x > 0 && x + w > limit as usize {
                max_x = max_x.max(x);
                x = 0;
                lines += 1;
            }
        }

        x += w;
        max_x = max_x.max(x);
    }

    Size {
        width: max_x as u16,
        height: lines as u16,
    }
}

fn grapheme_cell_width(cluster: &str, x: usize, tab_width: usize) -> usize {
    if cluster == "\t" {
        if tab_width == 0 {
            return 0;
        }
        let next_tab_stop = ((x / tab_width) + 1) * tab_width;
        return next_tab_stop.saturating_sub(x);
    }

    // Control chars (CR, ESC, etc.) measure as 0 — they don't advance the cursor.
    UnicodeWidthStr::width(cluster)
}
