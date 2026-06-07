use crate::Grapheme;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

/// An arena for extended grapheme clusters.
///
/// When a grapheme cluster exceeds 4 UTF-8 bytes (e.g. emoji ZWJ sequences),
/// it is stored here and referenced by a 24-bit offset carried in an extended
/// [`Grapheme`] handle.
///
/// Inspired by notcurses' `egcpool`, this arena is tuned for a *damage-based*
/// terminal framebuffer: cells are overwritten and released individually
/// between full clears, so reclamation has to actually defragment rather than
/// merely leak space until the next `clear`.
///
/// ## Allocation strategy
///
/// - **Best-fit reuse** of freed regions, then **append** at the high-water
///   mark, then `Full`.
/// - **Coalescing** on release: a freed region merges with its immediately
///   adjacent free neighbours, so repeated churn cannot fragment the arena
///   into permanently-unusable slivers.
/// - **Tail truncation**: when a freed (or coalesced) region reaches the end
///   of the backing buffer, the buffer length shrinks (capacity is retained).
///   This keeps `len` at the true high-water mark instead of growing forever.
/// - **Bulk clear** via [`clear`](Self::clear) resets everything — the "erase
///   plane" operation — invalidating all outstanding handles.
///
/// ## Storage format
///
/// Each *live* entry is a 2-byte little-endian length prefix followed by the
/// raw UTF-8 bytes:
///
/// ```text
/// [len_lo] [len_hi] [utf8 bytes ...]
/// ```
///
/// The explicit prefix (rather than NUL-termination) gives O(1) release and
/// keeps adjacent released entries from being mistaken for one. Free regions
/// store *no* in-band metadata; their bounds live in the two indexes below.
///
/// ## Free-region indexes
///
/// Two ordered indexes describe the same set of free regions:
///
/// - `by_offset: offset -> size` — supports O(log n) neighbour lookup for
///   coalescing.
/// - `by_size: (size, offset)` — supports O(log n) best-fit selection.
///
/// Free regions are kept disjoint, never adjacent (always coalesced), and
/// never touching the tail (always truncated). An intrusive free list (links
/// stored in the freed bytes, à la `malloc`) is deliberately avoided: the
/// smallest entry is 3 bytes, far too small to hold link pointers.
///
/// The arena is **not** a deduplicating interner — each insert gets its own
/// region. This keeps the common path (inline graphemes that never touch the
/// arena) at zero cost.
#[derive(Clone, Default)]
pub struct Arena {
    /// Contiguous byte storage; `len()` is the high-water mark.
    inner: Vec<u8>,
    /// Bytes occupied by *live* entries (including their length prefixes).
    count: usize,
    /// Free regions keyed by offset, for coalescing. `offset -> size`.
    by_offset: BTreeMap<usize, usize>,
    /// Free regions keyed by `(size, offset)`, for best-fit allocation.
    by_size: BTreeSet<(usize, usize)>,
}

/// Size of the length prefix for each entry (2 bytes, little-endian `u16`).
const PREFIX_SIZE: usize = 2;

/// Minimum backing-buffer allocation, to avoid a flurry of tiny reallocations
/// for the first few extended graphemes.
const MINIMUM_ALLOC: usize = 1024;

/// Maximum string payload addressable by the `u16` length prefix.
const MAX_ENTRY_LEN: usize = u16::MAX as usize;

impl Arena {
    /// An empty arena usable in const contexts.
    pub const EMPTY: Self = Self {
        inner: Vec::new(),
        count: 0,
        by_offset: BTreeMap::new(),
        by_size: BTreeSet::new(),
    };

    /// Maximum arena size: 16 MiB − 1, the addressable range of the 24-bit
    /// offset stored in an extended [`Grapheme`].
    pub const MAX_CAPACITY: usize = (1 << 24) - 1; // 0x00FF_FFFF = 16,777,215

    /// Create a new, empty arena.
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Create an arena with pre-allocated capacity (in bytes), clamped to
    /// [`MAX_CAPACITY`](Self::MAX_CAPACITY).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity.min(Self::MAX_CAPACITY)),
            ..Self::EMPTY
        }
    }

    // ── Queries ─────────────────────────────────────────────────────────

    /// Resolve an extended [`Grapheme`] to its stored `&str`.
    ///
    /// # Panics
    ///
    /// Panics if the handle does not refer to a live entry in this arena
    /// (out of bounds, or a freed/`clear`ed slot). In debug builds, also
    /// asserts the handle is actually extended.
    pub fn get(&self, grapheme: Grapheme) -> &str {
        let offset = self.offset_of(grapheme);

        // Bounds checks run *before* any indexing so a bad handle yields a
        // clear panic rather than an opaque index-out-of-bounds.
        let payload_start = offset + PREFIX_SIZE;
        assert!(payload_start <= self.inner.len(), "arena offset out of bounds");

        let len = self.entry_len(offset);
        assert_ne!(len, 0, "resolving a freed or empty arena entry");

        let payload_end = payload_start + len;
        assert!(payload_end <= self.inner.len(), "arena entry extends past end");

        // SAFETY: only valid UTF-8 is ever stored (via `try_insert`), and the
        // bounds above guarantee `payload_start..payload_end` is in range.
        unsafe { std::str::from_utf8_unchecked(self.inner.get_unchecked(payload_start..payload_end)) }
    }

    /// Bytes occupied by live entries (including length prefixes).
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    /// High-water mark of the backing buffer, in bytes (live + interior free).
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Capacity currently allocated by the backing buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// `true` if no live entries remain.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Interior free bytes available for reuse. With tail-truncation this is
    /// exactly `len() - count()`.
    #[inline]
    pub fn free_bytes(&self) -> usize {
        self.inner.len() - self.count
    }

    /// Number of distinct (coalesced) free regions — a fragmentation gauge.
    #[inline]
    pub fn free_region_count(&self) -> usize {
        self.by_offset.len()
    }

    // ── Mutation ────────────────────────────────────────────────────────

    /// Insert a UTF-8 grapheme cluster, returning an extended [`Grapheme`].
    ///
    /// Called by [`Grapheme::extended`](Grapheme::extended) when a
    /// cluster exceeds 4 bytes.
    pub fn try_insert(&mut self, value: &str) -> Result<Grapheme, GraphemeError> {
        let len = value.len();
        if len > MAX_ENTRY_LEN {
            return Err(GraphemeError::TooLong { len, max: MAX_ENTRY_LEN });
        }

        let needed = PREFIX_SIZE + len;
        let offset = self.allocate(needed)?;

        let prefix = (len as u16).to_le_bytes();
        self.inner[offset] = prefix[0];
        self.inner[offset + 1] = prefix[1];
        self.inner[offset + PREFIX_SIZE..offset + needed].copy_from_slice(value.as_bytes());

        self.count += needed;
        Ok(Grapheme::from_offset(offset))
    }

    /// Infallible [`try_insert`](Self::try_insert).
    ///
    /// # Panics
    ///
    /// Panics if the arena is full or the value exceeds the entry-length limit.
    pub fn insert(&mut self, value: &str) -> Grapheme {
        match self.try_insert(value) {
            Ok(grapheme) => grapheme,
            Err(err) => panic!("failed to insert grapheme: {err}"),
        }
    }

    /// Release a stored grapheme, coalescing with adjacent free regions and
    /// truncating the buffer if the result reaches the tail.
    ///
    /// # Panics
    ///
    /// Panics on an out-of-bounds handle or a double-release.
    pub fn remove(&mut self, grapheme: Grapheme) {
        let offset = self.offset_of(grapheme);

        let payload_start = offset + PREFIX_SIZE;
        assert!(payload_start <= self.inner.len(), "releasing out-of-bounds offset");

        let entry_len = self.entry_len(offset);
        assert_ne!(entry_len, 0, "double-release detected (length prefix is zero)");

        let total = PREFIX_SIZE + entry_len;
        assert!(offset + total <= self.inner.len(), "releasing entry past end");

        // Poison the prefix so a stale handle to *this* offset reads length 0
        // and panics, rather than yielding stale content. Best-effort: once a
        // region is reused, a stale handle into it resolves to the new entry.
        self.inner[offset] = 0;
        self.inner[offset + 1] = 0;
        self.count -= total;

        let mut start = offset;
        let mut size = total;

        // Coalesce with the immediately-preceding free region, if adjacent.
        if let Some((&prev_offset, &prev_size)) = self.by_offset.range(..start).next_back() {
            if prev_offset + prev_size == start {
                self.free_remove(prev_offset, prev_size);
                start = prev_offset;
                size += prev_size;
            }
        }

        // Coalesce with the immediately-following free region, if present.
        if let Some(&next_size) = self.by_offset.get(&(start + size)) {
            self.free_remove(start + size, next_size);
            size += next_size;
        }

        if start + size == self.inner.len() {
            // Region reaches the tail — shed it. Capacity is retained.
            self.inner.truncate(start);
        } else {
            self.free_insert(start, size);
        }
    }

    /// Reset the arena entirely, invalidating **all** outstanding handles that
    /// reference it. O(1) — the "erase plane" operation.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.by_offset.clear();
        self.by_size.clear();
        self.count = 0;
    }

    // ── Internals ───────────────────────────────────────────────────────

    /// Resolve a handle to its arena offset, asserting it is extended.
    #[inline]
    fn offset_of(&self, grapheme: Grapheme) -> usize {
        debug_assert!(
            grapheme.is_extended(),
            "passed a non-extended grapheme to the arena; its low 24 bits are not an offset"
        );
        grapheme.as_offset()
    }

    /// Read the entry length from the 2-byte little-endian prefix at `offset`.
    #[inline]
    fn entry_len(&self, offset: usize) -> usize {
        u16::from_le_bytes([self.inner[offset], self.inner[offset + 1]]) as usize
    }

    /// Register a free region in both indexes.
    #[inline]
    fn free_insert(&mut self, offset: usize, size: usize) {
        debug_assert!(size > 0);
        self.by_offset.insert(offset, size);
        self.by_size.insert((size, offset));
    }

    /// Remove a free region from both indexes.
    #[inline]
    fn free_remove(&mut self, offset: usize, size: usize) {
        self.by_offset.remove(&offset);
        self.by_size.remove(&(size, offset));
    }

    /// Allocate `needed` contiguous bytes: best-fit reuse, else append.
    fn allocate(&mut self, needed: usize) -> Result<usize, GraphemeError> {
        // Best-fit: smallest free region with `size >= needed`; ties broken by
        // lowest offset (address-ordered, friendlier to locality).
        if let Some(&(size, offset)) = self.by_size.range((needed, 0)..).next() {
            self.free_remove(offset, size);
            if size > needed {
                // Track the remainder — even a sub-prefix sliver — so it can
                // coalesce later instead of leaking until `clear`.
                self.free_insert(offset + needed, size - needed);
            }
            return Ok(offset);
        }

        // Append at the high-water mark with doubling growth.
        if self.inner.len() + needed <= Self::MAX_CAPACITY {
            let offset = self.inner.len();
            self.grow_to(offset + needed);
            // `resize` zero-fills; `try_insert` overwrites the whole region.
            self.inner.resize(offset + needed, 0);
            return Ok(offset);
        }

        Err(GraphemeError::Full)
    }

    /// Ensure the backing buffer can hold `min_len` bytes without reallocating
    /// on the subsequent `resize`, using doubling growth clamped to capacity.
    fn grow_to(&mut self, min_len: usize) {
        if self.inner.capacity() >= min_len {
            return;
        }
        let target = (self.inner.capacity() * 2)
            .max(MINIMUM_ALLOC)
            .max(min_len)
            .min(Self::MAX_CAPACITY);
        self.inner.reserve(target - self.inner.len());
    }
}

impl fmt::Debug for Arena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arena")
            .field("len", &self.inner.len())
            .field("live", &self.count)
            .field("free", &self.free_bytes())
            .field("free_regions", &self.by_offset.len())
            .field("capacity", &self.inner.capacity())
            .finish()
    }
}

// ── Offset abstraction ──────────────────────────────────────────────────────

/// Types convertible to a 24-bit arena offset.
///
/// Kept as a `const` trait so the existing
/// [`Grapheme::offset`](crate::Grapheme::offset) constructor's `[const]` bound
/// keeps resolving. Note: the blanket impls for `Grapheme`/`&Grapheme` were
/// removed deliberately — they let an *inline* grapheme be passed where an
/// offset was expected, silently reinterpreting UTF-8 bytes as an offset.
/// The arena now takes `Grapheme` directly and gates on `is_extended`.
pub const trait Offsetted {
    fn offset(self) -> usize;
}

impl const Offsetted for usize {
    #[inline]
    fn offset(self) -> usize {
        self
    }
}

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum GraphemeError {
    /// The arena reached its 16 MiB limit with no reclaimable space.
    #[error("grapheme arena is full (16 MiB limit reached)")]
    Full,

    /// The string exceeds the maximum entry length encodable in the prefix.
    #[error("grapheme of {len} bytes exceeds the maximum entry length ({max} bytes)")]
    TooLong { len: usize, max: usize },

    /// The string is too long to inline and requires an arena to encode.
    #[error("grapheme of {len} bytes needs an arena (inline holds at most {max} bytes)")]
    ArenaRequired { len: usize, max: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-check the two free indexes and the `free_bytes == len - count`
    /// invariant after an operation.
    fn check_invariants(arena: &Arena) {
        assert_eq!(arena.by_offset.len(), arena.by_size.len());
        let mut sum = 0;
        for (&offset, &size) in &arena.by_offset {
            assert!(arena.by_size.contains(&(size, offset)));
            assert!(offset + size < arena.inner.len(), "free region must not touch the tail");
            sum += size;
        }
        assert_eq!(sum, arena.free_bytes());
    }

    #[test]
    fn new_arena_is_empty() {
        let arena = Arena::new();
        assert_eq!(arena.count(), 0);
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn insert_and_resolve() {
        let mut arena = Arena::new();
        let s = "hello, 世界!";
        let g = arena.try_insert(s).unwrap();
        assert_eq!(arena.get(g), s);
        assert_eq!(arena.count(), PREFIX_SIZE + s.len());
        check_invariants(&arena);
    }

    #[test]
    fn insert_multiple() {
        let mut arena = Arena::new();
        let entries = ["alpha", "bravo", "charlie", "👨\u{200D}👩\u{200D}👧\u{200D}👦"];
        let handles: Vec<_> = entries.iter().map(|s| arena.insert(s)).collect();
        for (h, expected) in handles.iter().zip(entries.iter()) {
            assert_eq!(arena.get(*h), *expected);
        }
        check_invariants(&arena);
    }

    #[test]
    fn reuse_interior_slot() {
        let mut arena = Arena::new();
        let g1 = arena.try_insert("hello!").unwrap(); // [0,8)
        let _g2 = arena.try_insert("world!").unwrap(); // [8,16) — holds the tail
        let len_before = arena.len();

        arena.remove(g1); // interior free (0,8)
        assert_eq!(arena.len(), len_before, "interior remove does not shrink len");

        let g3 = arena.try_insert("reuse!").unwrap(); // best-fit -> offset 0
        assert_eq!(g3.as_offset(), g1.as_offset());
        assert_eq!(arena.len(), len_before, "reuse did not grow the arena");
        assert_eq!(arena.get(g3), "reuse!");
        check_invariants(&arena);
    }

    #[test]
    fn tail_remove_truncates() {
        let mut arena = Arena::new();
        let g1 = arena.try_insert("aaaaaa").unwrap(); // [0,8)
        let g2 = arena.try_insert("bbbbbb").unwrap(); // [8,16)
        assert_eq!(arena.len(), 16);

        arena.remove(g2);
        assert_eq!(arena.len(), 8, "tail release shrinks len");
        arena.remove(g1);
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        check_invariants(&arena);
    }

    #[test]
    fn coalesce_enables_large_alloc() {
        // Two adjacent 8-byte gaps must merge to fit a 16-byte entry without
        // growing the buffer. (A non-coalescing allocator would append.)
        let mut arena = Arena::new();
        let ga = arena.try_insert("aaaaaa").unwrap(); // [0,8)
        let gb = arena.try_insert("bbbbbb").unwrap(); // [8,16)
        let _gc = arena.try_insert("cccccc").unwrap(); // [16,24) tail guard
        let len_before = arena.len();

        arena.remove(ga);
        arena.remove(gb); // coalesces -> single free region (0,16)
        assert_eq!(arena.free_bytes(), 16);

        let big = "DDDDDDDDDDDDDD"; // 14 + 2 = 16 total
        let gd = arena.try_insert(big).unwrap();
        assert_eq!(gd.as_offset(), ga.as_offset());
        assert_eq!(arena.len(), len_before, "coalesced gap absorbed the entry");
        assert_eq!(arena.get(gd), big);
        check_invariants(&arena);
    }

    #[test]
    fn best_fit_picks_smallest() {
        let mut arena = Arena::new();
        let small = arena.try_insert("ab").unwrap(); // [0,4)
        let _med = arena.try_insert("medium").unwrap(); // [4,12)
        let big = arena.try_insert("bigger-entry!!").unwrap(); // 14+2 -> [12,28)
        let _guard = arena.try_insert("g").unwrap(); // [28,31) tail guard

        arena.remove(small); // free (0,4)
        arena.remove(big); // free (12,16)

        let g = arena.try_insert("xy").unwrap(); // needs 4 -> best-fit (0,4)
        assert_eq!(g.as_offset(), small.as_offset());
        check_invariants(&arena);
    }

    #[test]
    fn sub_prefix_sliver_is_not_leaked() {
        // A 2-byte remainder (< PREFIX_SIZE+1, so never independently usable)
        // must stay trackable and coalesce later, not vanish.
        let mut arena = Arena::new();
        let ge = arena.try_insert("EEEEEE").unwrap(); // [0,8)
        let gg = arena.try_insert("g").unwrap(); // [8,11) guard
        arena.remove(ge); // free (0,8)

        let gf = arena.try_insert("FFFF").unwrap(); // 4+2=6 -> offset 0, sliver (6,2)
        assert_eq!(gf.as_offset(), ge.as_offset());
        assert_eq!(arena.free_bytes(), 2, "sliver tracked, not dropped");

        // Releasing the guard coalesces with the sliver and truncates the tail
        // down to the end of F (6). If the sliver had leaked, len would be 8.
        arena.remove(gg);
        assert_eq!(arena.len(), 6);
        assert_eq!(arena.free_bytes(), 0);
        check_invariants(&arena);
    }

    #[test]
    fn churn_returns_to_high_water_mark() {
        let mut arena = Arena::new();
        let mut handles = Vec::new();
        for i in 0..200 {
            handles.push(arena.insert(&format!("entry-{i:04}-padding")));
        }
        let peak = arena.len();

        for h in handles.drain(..) {
            arena.remove(h);
        }
        assert_eq!(arena.len(), 0, "full drain truncates to nothing");

        for i in 0..200 {
            arena.insert(&format!("entry-{i:04}-padding"));
        }
        assert_eq!(arena.len(), peak, "no unbounded growth across churn");
        check_invariants(&arena);
    }

    #[test]
    fn clear_resets_everything() {
        let mut arena = Arena::new();
        arena.insert("some text here");
        arena.insert("more text");
        let temp = arena.insert("temp");
        arena.remove(temp);

        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
        assert_eq!(arena.count(), 0);
        assert_eq!(arena.free_bytes(), 0);
        check_invariants(&arena);
    }

    #[test]
    fn with_capacity_preallocates() {
        let arena = Arena::with_capacity(1024);
        assert!(arena.capacity() >= 1024);
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn too_long_is_rejected() {
        let mut arena = Arena::new();
        let long = "x".repeat(MAX_ENTRY_LEN + 1);
        assert!(matches!(arena.try_insert(&long), Err(GraphemeError::TooLong { .. })));
    }
}