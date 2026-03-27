// Copyright (c) 2025. Licensed under MIT or Apache-2.0.

use std::borrow::Cow;
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

// ============================================================================
// Coordinate Types - Strongly typed to prevent mixing up byte/grapheme/column
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ByteIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct GraphemeIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ColumnIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ColumnWidth(pub u8);

impl ColumnWidth {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const TWO: Self = Self(2);

    pub fn is_wide(self) -> bool {
        self.0 >= 2
    }
}

// ============================================================================
// Segment Metadata - Immutable, compact representation of one grapheme cluster
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub byte_start: ByteIndex,
    pub byte_len: u8,       // Max 255 bytes (enough for any grapheme)
    pub width: ColumnWidth, // Display width (0, 1, or 2)
    pub grapheme_index: GraphemeIndex,
    pub col_start: ColumnIndex, // Cumulative display column at start
}

impl Segment {
    pub fn byte_end(&self) -> ByteIndex {
        ByteIndex(self.byte_start.0 + self.byte_len as usize)
    }

    pub fn col_end(&self) -> ColumnIndex {
        ColumnIndex(self.col_start.0 + self.width.0 as usize)
    }

    pub fn range(&self) -> Range<ByteIndex> {
        self.byte_start..self.byte_end()
    }
}

// ============================================================================
// SmartString - Small String Optimization (SSO)
// Stores up to 23 bytes inline, heap for larger
// ============================================================================

pub const SSO_CAPACITY: usize = 23;

#[derive(Clone, PartialEq, Eq)]
pub enum SmartString {
    Inline { len: u8, buf: [u8; SSO_CAPACITY] },
    Heap(String),
}

impl SmartString {
    pub fn new(s: &str) -> Self {
        if s.len() <= SSO_CAPACITY {
            let mut buf = [0u8; SSO_CAPACITY];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            Self::Inline {
                len: s.len() as u8,
                buf,
            }
        } else {
            Self::Heap(s.to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Inline { len, buf } => unsafe {
                std::str::from_utf8_unchecked(&buf[..*len as usize])
            },
            Self::Heap(s) => s.as_str(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len as usize,
            Self::Heap(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SmartString").field(&self.as_str()).finish()
    }
}

impl fmt::Display for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for SmartString {
    fn default() -> Self {
        Self::Inline {
            len: 0,
            buf: [0; SSO_CAPACITY],
        }
    }
}

// ============================================================================
// SegmentedString - The main type with Arc<[Segment]> for cheap cloning
// ============================================================================

pub struct SegmentedString {
    text: SmartString,
    segments: Arc<[Segment]>,
    total_width: ColumnWidth,
    grapheme_count: GraphemeIndex,
    byte_len: usize,
    has_wide: bool,
}

impl SegmentedString {
    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    pub fn new(s: impl AsRef<str>) -> Self {
        let text = SmartString::new(s.as_ref());
        let (segments, total_width, has_wide) = Self::parse_segments(&text);

        Self {
            byte_len: text.len(),
            grapheme_count: GraphemeIndex(segments.len()),
            text,
            segments: Arc::from(segments.into_boxed_slice()),
            total_width,
            has_wide,
        }
    }

    fn parse_segments(text: &SmartString) -> (Vec<Segment>, ColumnWidth, bool) {
        let s = text.as_str();
        let mut segments = Vec::new();
        let mut byte_pos = 0;
        let mut col_pos = 0;
        let mut grapheme_idx = 0;
        let mut has_wide = false;

        // In real implementation, use unicode-segmentation crate here
        // This simplified version handles ASCII and basic UTF-8
        while byte_pos < s.len() {
            let (byte_len, width) = Self::measure_grapheme(&s[byte_pos..]);

            if width.0 >= 2 {
                has_wide = true;
            }

            segments.push(Segment {
                byte_start: ByteIndex(byte_pos),
                byte_len: byte_len as u8,
                width,
                grapheme_index: GraphemeIndex(grapheme_idx),
                col_start: ColumnIndex(col_pos),
            });

            byte_pos += byte_len;
            col_pos += width.0 as usize;
            grapheme_idx += 1;
        }

        (segments, ColumnWidth(col_pos as u8), has_wide)
    }

    fn measure_grapheme(s: &str) -> (usize, ColumnWidth) {
        // Simplified: in real impl use unicode-segmentation + unicode-width
        // For now, basic ASCII/UTF-8 handling
        let first_byte = s.as_bytes()[0];

        // ASCII fast path
        if first_byte < 0x80 {
            return (
                1,
                if first_byte == 0 {
                    ColumnWidth(0)
                } else {
                    ColumnWidth(1)
                },
            );
        }

        // Multi-byte UTF-8 (simplified)
        let char_len = if first_byte < 0xE0 {
            2
        } else if first_byte < 0xF0 {
            3
        } else {
            4
        };

        // Approximate width (real impl would use unicode-width crate)
        let ch = s[..char_len].chars().next().unwrap();
        let width = if ch.is_ascii() { 1 } else { 2 }; // CJK/emoji approx

        (char_len, ColumnWidth(width))
    }

    // -------------------------------------------------------------------------
    // Basic accessors
    // -------------------------------------------------------------------------

    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub fn grapheme_count(&self) -> GraphemeIndex {
        self.grapheme_count
    }

    pub fn display_width(&self) -> ColumnWidth {
        self.total_width
    }

    pub fn has_wide_chars(&self) -> bool {
        self.has_wide
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    // -------------------------------------------------------------------------
    // Coordinate Conversions
    // -------------------------------------------------------------------------

    pub fn byte_to_grapheme(&self, pos: ByteIndex) -> Option<GraphemeIndex> {
        self.segments
            .binary_search_by_key(&pos, |s| s.byte_start)
            .map(|i| GraphemeIndex(i))
            .ok()
    }

    pub fn grapheme_to_byte(&self, idx: GraphemeIndex) -> Option<ByteIndex> {
        self.segments.get(idx.0).map(|s| s.byte_start)
    }

    pub fn grapheme_to_column(&self, idx: GraphemeIndex) -> Option<ColumnIndex> {
        self.segments.get(idx.0).map(|s| s.col_start)
    }

    pub fn column_to_grapheme(&self, col: ColumnIndex) -> Option<GraphemeIndex> {
        self.segments
            .binary_search_by_key(&col, |s| s.col_start)
            .map(|i| GraphemeIndex(i))
            .ok()
            .or_else(|| {
                // If exact not found, find containing segment
                self.segments
                    .iter()
                    .position(|s| s.col_start.0 <= col.0 && col.0 < s.col_end().0)
                    .map(GraphemeIndex)
            })
    }

    // -------------------------------------------------------------------------
    // Access individual graphemes
    // -------------------------------------------------------------------------

    pub fn at(&self, idx: GraphemeIndex) -> Option<GraphemeRef> {
        self.segments.get(idx.0).map(|seg| self.grapheme_ref(seg))
    }

    pub fn at_column(&self, col: ColumnIndex) -> Option<GraphemeRef> {
        let idx = self.column_to_grapheme(col)?;
        self.at(idx)
    }

    pub fn first(&self) -> Option<GraphemeRef> {
        self.segments.first().map(|seg| self.grapheme_ref(seg))
    }

    pub fn last(&self) -> Option<GraphemeRef> {
        self.segments.last().map(|seg| self.grapheme_ref(seg))
    }

    fn grapheme_ref(&self, seg: &Segment) -> GraphemeRef {
        let text = &self.text.as_str()[seg.byte_start.0..seg.byte_end().0];
        GraphemeRef {
            text,
            index: seg.grapheme_index,
            col: seg.col_start,
            width: seg.width,
        }
    }

    // -------------------------------------------------------------------------
    // Slicing (zero-copy views)
    // -------------------------------------------------------------------------

    pub fn slice(&self, range: Range<GraphemeIndex>) -> SegmentedSlice {
        let start_idx = range.start.0.min(self.segments.len());
        let end_idx = range.end.0.min(self.segments.len());

        let col_start = self
            .segments
            .get(start_idx)
            .map(|s| s.col_start)
            .unwrap_or(ColumnIndex(self.total_width.0 as usize));

        SegmentedSlice {
            source: self,
            grapheme_range: GraphemeIndex(start_idx)..GraphemeIndex(end_idx),
            col_start,
        }
    }

    pub fn columns(&self, range: Range<ColumnIndex>) -> SegmentedSlice {
        let start = self
            .column_to_grapheme(range.start)
            .unwrap_or(GraphemeIndex(self.segments.len()));
        let end = self
            .column_to_grapheme(range.end)
            .unwrap_or(GraphemeIndex(self.segments.len()));
        self.slice(start..end)
    }

    pub fn split_at(&self, idx: GraphemeIndex) -> (SegmentedSlice, SegmentedSlice) {
        (
            self.slice(GraphemeIndex(0)..idx),
            self.slice(idx..self.grapheme_count),
        )
    }

    // -------------------------------------------------------------------------
    // Functional mutations (return new instances)
    // -------------------------------------------------------------------------

    pub fn appended(&self, other: impl AsRef<str>) -> Self {
        let mut new_text = String::with_capacity(self.byte_len + other.as_ref().len());
        new_text.push_str(self.as_str());
        new_text.push_str(other.as_ref());
        Self::new(new_text)
    }

    pub fn inserted(&self, idx: GraphemeIndex, text: impl AsRef<str>) -> Self {
        if idx.0 > self.segments.len() {
            return self.appended(text);
        }

        let byte_pos = self
            .grapheme_to_byte(idx)
            .unwrap_or(ByteIndex(self.byte_len));
        let mut new_text = String::with_capacity(self.byte_len + text.as_ref().len());
        new_text.push_str(&self.text.as_str()[..byte_pos.0]);
        new_text.push_str(text.as_ref());
        new_text.push_str(&self.text.as_str()[byte_pos.0..]);
        Self::new(new_text)
    }

    pub fn deleted_range(&self, range: Range<GraphemeIndex>) -> Self {
        let start = range.start.0.min(self.segments.len());
        let end = range.end.0.min(self.segments.len());
        if start >= end {
            return self.clone();
        }

        let byte_start = self.segments[start].byte_start.0;
        let byte_end = self.segments[end.saturating_sub(1)].byte_end().0;

        let mut new_text = String::with_capacity(self.byte_len - (byte_end - byte_start));
        new_text.push_str(&self.text.as_str()[..byte_start]);
        new_text.push_str(&self.text.as_str()[byte_end..]);
        Self::new(new_text)
    }

    pub fn truncated_to_width(&self, width: ColumnWidth) -> SegmentedSlice {
        self.columns(ColumnIndex(0)..ColumnIndex(width.0 as usize))
    }

    pub fn repeated(&self, times: usize) -> Self {
        if times == 0 {
            return Self::new("");
        }
        if times == 1 {
            return self.clone();
        }

        let mut new_text = String::with_capacity(self.byte_len * times);
        for _ in 0..times {
            new_text.push_str(self.as_str());
        }
        Self::new(new_text)
    }
}

// ============================================================================
// SegmentedSlice - Zero-copy view into a SegmentedString
// ============================================================================

pub struct SegmentedSlice<'a> {
    source: &'a SegmentedString,
    grapheme_range: Range<GraphemeIndex>,
    col_start: ColumnIndex,
}

impl<'a> SegmentedSlice<'a> {
    pub fn as_str(&self) -> &'a str {
        if self.is_empty() {
            return "";
        }

        let first = self.grapheme_range.start.0;
        let last = self.grapheme_range.end.0.saturating_sub(1);

        let start_byte = self
            .source
            .segments
            .get(first)
            .map(|s| s.byte_start.0)
            .unwrap_or(0);
        let end_byte = self
            .source
            .segments
            .get(last)
            .map(|s| s.byte_end().0)
            .unwrap_or(start_byte);

        &self.source.text.as_str()[start_byte..end_byte]
    }

    pub fn grapheme_count(&self) -> GraphemeIndex {
        GraphemeIndex(
            self.grapheme_range
                .end
                .0
                .saturating_sub(self.grapheme_range.start.0),
        )
    }

    pub fn display_width(&self) -> ColumnWidth {
        let cols = self
            .grapheme_range
            .end
            .0
            .saturating_sub(self.grapheme_range.start.0) as u8;
        ColumnWidth(cols) // Simplified - should sum actual widths
    }

    pub fn is_empty(&self) -> bool {
        self.grapheme_range.start >= self.grapheme_range.end
    }

    pub fn to_owned(&self) -> SegmentedString {
        SegmentedString::new(self.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = GraphemeRef<'a>> + '_ {
        self.source.segments[self.grapheme_range.start.0..self.grapheme_range.end.0]
            .iter()
            .map(|seg| self.source.grapheme_ref(seg))
    }
}

impl<'a> fmt::Display for SegmentedSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> fmt::Debug for SegmentedSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SegmentedSlice")
            .field(&self.as_str())
            .finish()
    }
}

// ============================================================================
// GraphemeRef - Reference to a single grapheme cluster
// ============================================================================

#[derive(Clone, Copy)]
pub struct GraphemeRef<'a> {
    pub text: &'a str,
    pub index: GraphemeIndex,
    pub col: ColumnIndex,
    pub width: ColumnWidth,
}

impl<'a> fmt::Debug for GraphemeRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphemeRef")
            .field("text", &self.text)
            .field("index", &self.index.0)
            .field("col", &self.col.0)
            .field("width", &self.width.0)
            .finish()
    }
}

impl<'a> fmt::Display for GraphemeRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text)
    }
}

// ============================================================================
// Iterators
// ============================================================================

pub struct Graphemes<'a> {
    string: &'a SegmentedString,
    current: GraphemeIndex,
    end: GraphemeIndex,
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = GraphemeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 >= self.end.0 {
            return None;
        }
        let result = self.string.at(self.current)?;
        self.current.0 += 1;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end.0.saturating_sub(self.current.0);
        (remaining, Some(remaining))
    }
}

impl<'a> DoubleEndedIterator for Graphemes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current.0 >= self.end.0 {
            return None;
        }
        self.end.0 -= 1;
        self.string.at(self.end)
    }
}

impl<'a> ExactSizeIterator for Graphemes<'a> {}

// Make SegmentedString iterable
impl<'a> IntoIterator for &'a SegmentedString {
    type Item = GraphemeRef<'a>;
    type IntoIter = Graphemes<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Graphemes {
            string: self,
            current: GraphemeIndex(0),
            end: self.grapheme_count,
        }
    }
}

// ============================================================================
// Trait implementations
// ============================================================================

impl Clone for SegmentedString {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            segments: Arc::clone(&self.segments), // Cheap!
            total_width: self.total_width,
            grapheme_count: self.grapheme_count,
            byte_len: self.byte_len,
            has_wide: self.has_wide,
        }
    }
}

impl fmt::Debug for SegmentedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentedString")
            .field("text", &self.text.as_str())
            .field("segments", &self.segments.len())
            .field("width", &self.total_width.0)
            .field("graphemes", &self.grapheme_count.0)
            .finish()
    }
}

impl fmt::Display for SegmentedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for SegmentedString {
    fn default() -> Self {
        Self::new("")
    }
}

impl From<&str> for SegmentedString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SegmentedString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<SmartString> for SegmentedString {
    fn from(s: SmartString) -> Self {
        Self::new(s.as_str())
    }
}

impl PartialEq for SegmentedString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SegmentedString {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_construction() {
        let s = SegmentedString::new("Hello");
        assert_eq!(s.grapheme_count().0, 5);
        assert_eq!(s.display_width().0, 5);
        assert!(!s.has_wide_chars());
    }

    #[test]
    fn test_coord_conversions() {
        let s = SegmentedString::new("Hello");

        // Byte to grapheme
        assert_eq!(s.byte_to_grapheme(ByteIndex(0)), Some(GraphemeIndex(0)));
        assert_eq!(s.byte_to_grapheme(ByteIndex(4)), Some(GraphemeIndex(4)));

        // Grapheme to column (1:1 for ASCII)
        assert_eq!(s.grapheme_to_column(GraphemeIndex(2)), Some(ColumnIndex(2)));

        // Column to grapheme
        assert_eq!(s.column_to_grapheme(ColumnIndex(2)), Some(GraphemeIndex(2)));
    }

    #[test]
    fn test_slicing() {
        let s = SegmentedString::new("Hello World");
        let slice = s.slice(GraphemeIndex(6)..GraphemeIndex(11));
        assert_eq!(slice.as_str(), "World");
    }

    #[test]
    fn test_column_slicing() {
        let s = SegmentedString::new("Hello World");
        let slice = s.columns(ColumnIndex(0)..ColumnIndex(5));
        assert_eq!(slice.as_str(), "Hello");
    }

    #[test]
    fn test_functional_mutations() {
        let s = SegmentedString::new("Hello");

        let s2 = s.inserted(GraphemeIndex(5), " World");
        assert_eq!(s2.as_str(), "Hello World");

        let s3 = s2.deleted_range(GraphemeIndex(5)..GraphemeIndex(11));
        assert_eq!(s3.as_str(), "Hello");

        let s4 = s.appended("!");
        assert_eq!(s4.as_str(), "Hello!");
    }

    #[test]
    fn test_iteration() {
        let s = SegmentedString::new("Hi");
        let mut iter = s.into_iter();

        let g1 = iter.next().unwrap();
        assert_eq!(g1.text, "H");
        assert_eq!(g1.index.0, 0);

        let g2 = iter.next().unwrap();
        assert_eq!(g2.text, "i");
        assert_eq!(g2.index.0, 1);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_cheap_clone() {
        let s1 = SegmentedString::new("Hello World this is a longer string");
        let s2 = s1.clone();

        // Both point to same Arc data
        assert!(Arc::ptr_eq(&s1.segments, &s2.segments));
        assert_eq!(s1.as_str(), s2.as_str());
    }

    #[test]
    fn test_split_at() {
        let s = SegmentedString::new("Hello World");
        let (left, right) = s.split_at(GraphemeIndex(5));

        assert_eq!(left.as_str(), "Hello");
        assert_eq!(right.as_str(), " World");
    }

    #[test]
    fn test_empty() {
        let s = SegmentedString::new("");
        assert!(s.is_empty());
        assert_eq!(s.grapheme_count().0, 0);
        assert_eq!(s.display_width().0, 0);
    }
}
