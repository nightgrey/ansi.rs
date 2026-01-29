use unicode_segmentation::{UnicodeSegmentation};
use unicode_display_width::is_double_width;

pub trait DisplayWidth {
    fn display_width(&self) -> usize;
    fn cluster_display_width(&self) -> usize;
}

impl<T: AsRef<str>> DisplayWidth for T {
    fn display_width(&self) -> usize {
        self.as_ref().graphemes(true).fold(0, |acc, grapheme_cluster| {
            acc + (grapheme_cluster.cluster_display_width())
        })
    }

    fn cluster_display_width(&self) -> usize {
        for char in self.as_ref().chars() {
            // emoji style variation selector
            if char == '\u{FE0F}' {
                return 2;
            }

            if is_double_width(char) {
                return 2;
            }
        }

        1
    }

}

pub fn split(s: &str, width: usize) -> Split<'_> {
    Split::new(s, width)
}

/// Truncates a string by display width while preserving grapheme cluster boundaries.
pub fn truncate(s: &str, width: usize) -> &str {
    split(s, width).next().unwrap_or("")
}

/// Iterator that splits a string into lines based on display width,
/// preserving grapheme cluster boundaries.
pub struct Split<'a> {
    remaining: &'a str,
    max_width: usize,
}

impl<'a> Split<'a> {
    pub fn new(source: &'a str, max_width: usize) -> Self {
        Self {
            remaining: source,
            max_width,
        }
    }
}

impl<'a> Iterator for Split<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let mut current_width = 0;
        let mut end_byte = 0;

        for g in UnicodeSegmentation::graphemes(self.remaining, true) {
            let w = g.cluster_display_width();

            // If adding this grapheme exceeds width AND we have content,
            // yield what we have so far
            if current_width + w > self.max_width && end_byte > 0 {
                let line = &self.remaining[..end_byte];
                self.remaining = &self.remaining[end_byte..];
                return Some(line);
            }

            current_width += w;
            end_byte += g.len();
        }

        // Yield the remaining content
        let line = self.remaining;
        self.remaining = "";
        Some(line)
    }
}
