use crate::{Display, Element, ElementKind};
use shape::ShapedText;

const DEFAULT_TAB_WIDTH: usize = 4;

pub fn measure_next(
    known: taffy::Size<Option<f32>>,
    available: taffy::Size<taffy::AvailableSpace>,
    node: &Element<'_>,
    shaped: Option<&ShapedText>,
) -> taffy::Size<f32> {
    match &node.kind {
        ElementKind::Span(text) => {
            let wrap_width = resolve_wrap_width(known.width, available.width, node.layout.display);
            // Fall back to on-the-fly shaping if no pre-shaped cache entry is available
            // (e.g. when the measure function is invoked outside a `compute_layout` pass).
            let owned;
            let shape = match shaped {
                Some(s) => s,
                None => {
                    owned = ShapedText::new(text);
                    &owned
                }
            };
            let size = shape.measure(wrap_width, node.layout.display);

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

mod shape {
    //! Text shaping: one-time analysis of a span's content.
    //!
    //! [`ShapedText`] walks the source string once, computes per-grapheme widths,
    //! and annotates clusters with UAX #14 break opportunities. The result is
    //! reusable — measurement, min/max-content queries, and line wrapping all
    //! consume the same cluster vector without re-scanning the text.

    use super::DEFAULT_TAB_WIDTH;
    use crate::Display;
    use geometry::Size;
    use unicode_linebreak::{BreakOpportunity, linebreaks};
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;

    /// Whether the Unicode line-breaking algorithm permits a break after a cluster.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BreakAfter {
        No,
        Allowed,
        Mandatory,
    }

    /// One grapheme cluster plus the data needed to measure or wrap it.
    #[derive(Debug, Clone, Copy)]
    pub struct Cluster {
        pub byte_start: u32,
        pub byte_len: u16,
        pub width: u16,
        pub break_after: BreakAfter,
        pub is_newline: bool,
    }

    /// A laid-out line produced by [`LineWrapper`]. Byte range references the source text.
    #[derive(Debug, Clone, Copy)]
    pub struct Line {
        pub byte_start: u32,
        pub byte_end: u32,
        pub width: u32,
        /// True when the line terminated on an explicit newline (as opposed to soft-wrap or EOT).
        pub hard_break: bool,
    }

    #[derive(Debug, Clone)]
    pub struct ShapedText {
        pub clusters: Vec<Cluster>,
        /// Width of the longest unbreakable run (largest single "word").
        pub min_content_width: u32,
        /// Width of the longest line if nothing soft-wrapped (respecting only mandatory breaks).
        pub max_content_width: u32,
        pub source_len: u32,
    }

    impl ShapedText {
        pub fn new(text: &str) -> Self {
            Self::with_tab_width(text, DEFAULT_TAB_WIDTH)
        }

        pub fn with_tab_width(text: &str, tab_width: usize) -> Self {
            if text.is_empty() {
                return Self {
                    clusters: Vec::new(),
                    min_content_width: 0,
                    max_content_width: 0,
                    source_len: 0,
                };
            }

            // linebreaks yields (byte_pos, kind) where byte_pos is the first byte of the next line.
            // We want to tag the cluster whose byte_end == byte_pos.
            let mut breaks = linebreaks(text).peekable();
            let mut clusters: Vec<Cluster> = Vec::new();
            let mut x: usize = 0;

            for (byte_start, cluster) in text.grapheme_indices(true) {
                let byte_len = cluster.len();
                let byte_end = byte_start + byte_len;

                let is_newline = cluster.contains('\n') || cluster == "\r";

                let width = if cluster == "\t" {
                    if tab_width == 0 {
                        0
                    } else {
                        let next_stop = ((x / tab_width) + 1) * tab_width;
                        next_stop.saturating_sub(x)
                    }
                } else if is_newline {
                    0
                } else {
                    UnicodeWidthStr::width(cluster)
                };

                let mut break_after = BreakAfter::No;
                while let Some(&(pos, kind)) = breaks.peek() {
                    if pos < byte_end {
                        breaks.next();
                    } else if pos == byte_end {
                        break_after = match kind {
                            BreakOpportunity::Mandatory => BreakAfter::Mandatory,
                            BreakOpportunity::Allowed => BreakAfter::Allowed,
                        };
                        breaks.next();
                        break;
                    } else {
                        break;
                    }
                }

                clusters.push(Cluster {
                    byte_start: byte_start as u32,
                    byte_len: byte_len as u16,
                    width: width as u16,
                    break_after,
                    is_newline,
                });

                if is_newline {
                    x = 0;
                } else {
                    x += width;
                }
            }

            let (min_content_width, max_content_width) = compute_min_max(&clusters);

            Self {
                clusters,
                min_content_width,
                max_content_width,
                source_len: text.len() as u32,
            }
        }

        pub fn is_empty(&self) -> bool {
            self.clusters.is_empty()
        }

        pub fn measure(&self, wrap_width: Option<u32>, display: Display) -> Size {
            if self.clusters.is_empty() {
                return Size {
                    width: 0,
                    height: 0,
                };
            }

            let mut max_w: u32 = 0;
            let mut lines: u32 = 0;
            let mut last_hard = false;

            for line in self.wrap(wrap_width, display) {
                max_w = max_w.max(line.width);
                lines += 1;
                last_hard = line.hard_break;
            }

            if last_hard {
                lines += 1;
            }

            Size {
                width: max_w as u16,
                height: lines.max(1) as u16,
            }
        }

        pub fn wrap(&self, wrap_width: Option<u32>, display: Display) -> LineWrapper<'_> {
            let inline = matches!(display, Display::Inline);
            LineWrapper {
                clusters: &self.clusters,
                idx: 0,
                wrap_width: if inline { None } else { wrap_width },
                inline,
            }
        }
    }

    pub struct LineWrapper<'a> {
        clusters: &'a [Cluster],
        idx: usize,
        wrap_width: Option<u32>,
        inline: bool,
    }

    impl<'a> Iterator for LineWrapper<'a> {
        type Item = Line;

        fn next(&mut self) -> Option<Line> {
            if self.idx >= self.clusters.len() {
                return None;
            }

            let first = &self.clusters[self.idx];
            let line_start = first.byte_start;
            let mut width: u32 = 0;

            // Track the most recent UAX #14 allowed-break point within this line so we can
            // retreat to it when we overflow the wrap width.
            let mut last_break_cluster: Option<usize> = None;
            let mut last_break_width: u32 = 0;
            let mut last_break_byte_end: u32 = line_start;

            while self.idx < self.clusters.len() {
                let c = &self.clusters[self.idx];

                // Hard newline — emit the line up to (but not including) the newline, consume it.
                if !self.inline && c.is_newline {
                    let byte_end = c.byte_start;
                    self.idx += 1;
                    return Some(Line {
                        byte_start: line_start,
                        byte_end,
                        width,
                        hard_break: true,
                    });
                }

                let w = c.width as u32;

                if let Some(limit) = self.wrap_width {
                    if limit > 0 && width > 0 && width + w > limit {
                        if let Some(break_idx) = last_break_cluster {
                            let byte_end = last_break_byte_end;
                            self.idx = break_idx;
                            return Some(Line {
                                byte_start: line_start,
                                byte_end,
                                width: last_break_width,
                                hard_break: false,
                            });
                        } else {
                            // No break opportunity yet — fall back to char-wrap at current position.
                            return Some(Line {
                                byte_start: line_start,
                                byte_end: c.byte_start,
                                width,
                                hard_break: false,
                            });
                        }
                    }
                }

                width += w;
                let cluster_end = c.byte_start + c.byte_len as u32;
                self.idx += 1;

                if matches!(c.break_after, BreakAfter::Allowed) {
                    last_break_cluster = Some(self.idx);
                    last_break_width = width;
                    last_break_byte_end = cluster_end;
                }
            }

            let last = self.clusters.last().unwrap();
            Some(Line {
                byte_start: line_start,
                byte_end: last.byte_start + last.byte_len as u32,
                width,
                hard_break: false,
            })
        }
    }

    // min_content = longest run between any two break opportunities (Allowed or Mandatory).
    // max_content = longest line between Mandatory breaks (ignoring allowed breaks — no soft wrap).
    fn compute_min_max(clusters: &[Cluster]) -> (u32, u32) {
        let mut min_cw: u32 = 0;
        let mut max_cw: u32 = 0;
        let mut run: u32 = 0;
        let mut line: u32 = 0;

        for c in clusters {
            if c.is_newline {
                max_cw = max_cw.max(line);
                min_cw = min_cw.max(run);
                run = 0;
                line = 0;
                continue;
            }

            let w = c.width as u32;
            run += w;
            line += w;

            match c.break_after {
                BreakAfter::Allowed => {
                    min_cw = min_cw.max(run);
                    run = 0;
                }
                BreakAfter::Mandatory => {
                    min_cw = min_cw.max(run);
                    max_cw = max_cw.max(line);
                    run = 0;
                    line = 0;
                }
                BreakAfter::No => {}
            }
        }

        min_cw = min_cw.max(run);
        max_cw = max_cw.max(line);
        (min_cw, max_cw)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::Display;

        #[test]
        fn empty_text_is_zero_size() {
            let s = ShapedText::new("");
            assert_eq!(s.measure(None, Display::Block).width, 0);
            assert_eq!(s.measure(None, Display::Block).height, 0);
        }

        #[test]
        fn plain_ascii_single_line() {
            let s = ShapedText::new("hello");
            let size = s.measure(None, Display::Block);
            assert_eq!(size.width, 5);
            assert_eq!(size.height, 1);
        }

        #[test]
        fn explicit_newline_adds_a_line() {
            let s = ShapedText::new("ab\ncd");
            let size = s.measure(None, Display::Block);
            assert_eq!(size.width, 2);
            assert_eq!(size.height, 2);
        }

        #[test]
        fn trailing_newline_preserves_old_behavior() {
            let s = ShapedText::new("hello\n");
            let size = s.measure(None, Display::Block);
            assert_eq!(size.height, 2);
        }

        #[test]
        fn inline_ignores_newlines_and_wrap_width() {
            let s = ShapedText::new("a\nb c d");
            let size = s.measure(Some(2), Display::Inline);
            // inline: newlines are plain zero-width chars, no wrap
            assert_eq!(size.height, 1);
        }

        #[test]
        fn word_aware_wrap_breaks_at_space() {
            let s = ShapedText::new("hello world");
            // width 7 fits "hello " (6 with trailing space) or "hello" (5), then "world" on next line.
            let size = s.measure(Some(7), Display::Block);
            assert_eq!(size.height, 2);
            assert!(size.width <= 7);
        }

        #[test]
        fn long_word_falls_back_to_char_wrap() {
            let s = ShapedText::new("abcdefgh");
            let size = s.measure(Some(3), Display::Block);
            assert_eq!(size.height, 3);
            assert!(size.width <= 3);
        }

        #[test]
        fn tab_snaps_to_tab_stop() {
            let s = ShapedText::new("a\tb");
            // 'a' at x=0 width 1; \t at x=1 advances to next stop (4) → width 3; 'b' at x=4 width 1.
            assert_eq!(s.measure(None, Display::Block).width, 5);
        }

        #[test]
        fn grapheme_clusters_measure_once() {
            // 'é' as e + combining acute — one cluster, width 1.
            let s = ShapedText::new("e\u{0301}");
            assert_eq!(s.measure(None, Display::Block).width, 1);
        }

        #[test]
        fn min_max_content_widths() {
            let s = ShapedText::new("foo bar baz");
            assert_eq!(s.min_content_width, 3);
            assert_eq!(s.max_content_width, 11);
        }

        #[test]
        fn wrap_iterator_yields_expected_lines() {
            let s = ShapedText::new("a\nb");
            let lines: Vec<_> = s.wrap(None, Display::Block).collect();
            assert_eq!(lines.len(), 2);
            assert!(lines[0].hard_break);
            assert!(!lines[1].hard_break);
        }
    }
}
