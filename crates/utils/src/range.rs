use std::ops;
use std::ops::{Range, RangeBounds};
fn range_fail(start: usize, end: usize, len: usize) -> ! {
    if start > len {
        panic!(
            "range start index {start} out of range for slice of length {len}",
        )
    }

    if end > len {
        panic!(
            "range end index {end} out of range for slice of length {len}",
        )
    }

    if start > end {
        panic!(
            "slice index starts at {start} but ends at {end}",
        )
    }

    // Only reachable if the range was a `RangeInclusive` or a
    // `RangeToInclusive`, with `end == len`.
    panic!(
        "range end index {end} out of range for slice of length {len}",
    )
}

#[inline]
pub fn into_range(
    bounds: impl RangeBounds<usize>,
    len: usize,
) -> Range<usize> {
    let end = match bounds.end_bound().copied() {
        ops::Bound::Included(end) if end >= len => range_fail(0, end, len),
        // Cannot overflow because `end < len` implies `end < usize::MAX`.
        ops::Bound::Included(end) => end + 1,

        ops::Bound::Excluded(end) if end > len => range_fail(0, end, len),
        ops::Bound::Excluded(end) => end,

        ops::Bound::Unbounded => len,
    };

    let start = match bounds.start_bound().copied() {
        ops::Bound::Excluded(start) if start >= end => range_fail(start, end, len),
        // Cannot overflow because `start < end` implies `start < usize::MAX`.
        ops::Bound::Excluded(start) => start + 1,

        ops::Bound::Included(start) if start > end => range_fail(start, end, len),
        ops::Bound::Included(start) => start,

        ops::Bound::Unbounded => 0,
    };

    start..end
}


#[inline]
pub fn into_range_unchecked(
    bounds: impl RangeBounds<usize>,
    len: usize,
) -> Range<usize> {
    let end = match bounds.end_bound().copied() {
        ops::Bound::Included(end) => end + 1,
        ops::Bound::Excluded(end) => end,
        ops::Bound::Unbounded => len,
    };

    let start = match bounds.start_bound().copied() {
        ops::Bound::Excluded(start) => start + 1,
        ops::Bound::Included(start) => start,
        ops::Bound::Unbounded => 0,
    };

    start..end
}