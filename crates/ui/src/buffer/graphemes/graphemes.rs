use super::{Entry, Grapheme, Slot};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

/// An arena for extended grapheme clusters.
///
/// When a grapheme cluster exceeds 4 UTF-8 bytes (e.g. emoji ZWJ sequences),
/// it is stored here and referenced by a 24-bit slot carried in an extended
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
/// - `by_slot: slot -> size` — supports O(log n) neighbour lookup for
///   coalescing.
/// - `by_size: (size, slot)` — supports O(log n) best-fit selection.
///
/// Free regions are kept disjoint, never adjacent (always coalesced), and
/// never touching the tail (always truncated). An intrusive free list (links
/// stored in the freed bytes, à la `malloc`) is deliberately avoided: the
/// smallest entry is 3 bytes, far too small to hold link pointers.
///
/// The arena is **not** a deduplicating interner — each insert gets its own
/// region. This keeps the common path (inline graphemes that never touch the
/// arena) at zero cost.
#[derive(Default, Clone, PartialEq)]
pub struct Graphemes {
    /// Contiguous byte storage; `len()` is the high-water mark.
    inner: Vec<u8>,
    /// Bytes occupied by *live* entries (including their length prefixes).
    len: usize,
    /// Free regions keyed by slot, for coalescing. `slot -> size`.
    slots: BTreeMap<usize, usize>,
    /// Free regions keyed by `(size, slot)`, for best-fit allocation.
    sizes: BTreeSet<(usize, usize)>,
}

impl Graphemes {
    /// Size of the length prefix for each entry (2 bytes, little-endian `u16`).
    const PREFIX: usize = 2;
    /// Maximum string payload addressable by the `u16` length prefix.
    const MAX_LEN: usize = u16::MAX as usize;
    /// An empty arena usable in const contexts.
    pub const EMPTY: Self = Self {
        inner: Vec::new(),
        len: 0,
        slots: BTreeMap::new(),
        sizes: BTreeSet::new(),
    };

    /// Maximum arena size: 16 MiB − 1, the addressable range of the 24-bit
    /// slot stored in an extended [`Grapheme`].
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

    /// Resolve a grapheme handle to a `&str` or `None`.
    ///
    /// - Inline graphemes borrow from the grapheme handle.
    /// - Extended graphemes borrow from the arena.
    /// - Empty graphemes yield `None`.
    #[inline]
    pub fn get<'a>(&'a self, grapheme: &'a Grapheme) -> Option<&'a str> {
        if grapheme.is_empty() {
            None
        } else if grapheme.is_extended() {
            self.try_resolve(Slot::try_from(grapheme).ok()?)
                .map(|meta| unsafe {
                    std::str::from_utf8_unchecked(self.inner.get_unchecked(meta.start..meta.end))
                }).ok()
        } else {
            Some(grapheme.as_inline_str())
        }
    }

    /// Resolve a grapheme handle to a `&str`.
    ///
    /// - Inline graphemes borrow from the grapheme handle.
    /// - Extended graphemes borrow from the arena.
    /// - Empty graphemes yield `default`.
    #[inline]
    pub fn get_or<'a>(&'a self, grapheme: &'a Grapheme, default: &'a str) -> &'a str {
        self.get(grapheme).unwrap_or(default)
    }


    /// Returns `true` if the arena contains the given slot.
    #[inline]
    pub fn contains(&self, grapheme: Grapheme) -> bool {
        match Slot::try_from(grapheme) {
            Ok(slot) => self.try_get_len(slot).is_ok(),
            Err(_) => false,
        }
    }

    /// Insert a UTF-8 grapheme cluster, returning a [`Grapheme`].
    pub fn try_insert(&mut self, s: &str) -> Result<Grapheme, GraphemesError> {
        let len = s.len();
        // An empty payload would write a 0 length-prefix, which is the freed
        // sentinel — the handle would be unresolvable. Empty has no cell, so
        // map it to the inline empty grapheme without touching the arena.
        if len == 0 {
            return Ok(Grapheme::EMPTY);
        }
        if len > Self::MAX_LEN {
            return Err(GraphemesError::TooLong {
                len,
                max: Self::MAX_LEN,
            });
        }

        let needed = Self::PREFIX + len;
        let slot = self.allocate(needed)?;
        let index = slot.as_usize();

        let prefix = (len as u16).to_le_bytes();
        self.inner[index] = prefix[0];
        self.inner[index + 1] = prefix[1];
        self.inner[index + Self::PREFIX..index + needed].copy_from_slice(s.as_bytes());

        self.len += needed;
        Ok(slot.into())
    }

    /// Infallible [`try_insert`](Self::try_insert).
    ///
    /// # Panics
    ///
    /// Panics if the arena is full or the value exceeds the entry-length limit.
    pub fn insert(&mut self, s: &str) -> Grapheme {
        self.try_insert(s)
            .unwrap_or_else(|e| panic!("failed to insert grapheme: {e}"))
    }

    /// Release a stored grapheme, coalescing with adjacent free regions and
    /// truncating the buffer if the result reaches the tail.
    ///
    /// Returns `Err` for an out-of-bounds handle or a double-release, leaving
    /// the arena unchanged.
    pub fn try_remove(&mut self, g: Grapheme) -> Result<(), GraphemesError> {
        let slot = Slot::try_from(g)?;
        let meta = self.try_resolve(slot)?;
        let index = meta.index();

        // Poison the prefix so a stale handle to *this* slot reads length 0
        // and panics, rather than yielding stale content. Best-effort: once a
        // region is reused, a stale handle into it resolves to the new entry.
        self.inner[index] = 0;
        self.inner[index + 1] = 0;
        self.len -= meta.len + Self::PREFIX;

        let mut index = index;
        let mut size = meta.len + Self::PREFIX;

        // Coalesce with the immediately-preceding free region, if adjacent.
        if let Some((&prev_index, &prev_size)) = self.slots.range(..index).next_back()
            && prev_index + prev_size == index
        {
            self.free_remove(prev_index, prev_size);
            index = prev_index;
            size += prev_size;
        }

        // Coalesce with the immediately-following free region, if present.
        if let Some(&next_size) = self.slots.get(&(index + size)) {
            self.free_remove(index + size, next_size);
            size += next_size;
        }

        if index + size == self.count_total() {
            // Region reaches the tail — shed it. Capacity is retained.
            self.inner.truncate(index);
        } else {
            self.free_insert(index, size);
        }

        Ok(())
    }

    /// Release a stored grapheme, ignoring invalid or already-freed handles.
    /// Use [`try_remove`](Self::try_remove) to observe those errors.
    pub fn remove(&mut self, g: Grapheme) {
        let _ = self.try_remove(g);
    }

    /// High-water mark of the backing buffer, in bytes (live + interior free).
    #[inline]
    pub fn count_total(&self) -> usize {
        self.inner.len()
    }

    /// Interior free bytes available for reuse. With tail-truncation this is
    /// exactly `len() - count()`.
    /// Interior free bytes available for reuse.
    ///
    /// Equal to [`count_total`](Self::count_total) −
    /// [`count_occupied`](Self::count_occupied). With tail truncation this
    /// includes only interior gaps, never the region past the high-water
    /// mark.
    #[inline]
    pub fn count_free(&self) -> usize {
        self.count_total() - self.len
    }

    /// Number of distinct (coalesced) free regions — a fragmentation gauge.
    /// Number of distinct coalesced free regions — a fragmentation gauge.
    ///
    /// A healthy arena should have 0 or 1 free regions at steady state;
    /// higher counts indicate fragmentation. Zero-cost to query.
    #[inline]
    pub fn count_regions(&self) -> usize {
        self.slots.len()
    }

    /// Total allocated capacity of the backing buffer, in bytes.
    ///
    /// This is the `Vec<u8>` capacity — it may exceed
    /// [`count_total`](Self::count_total).
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// `true` if no live entries remain.
    /// `true` if no live entries remain.
    ///
    /// After [`clear`](Self::clear) this is always `true`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Bytes occupied by live entries (including length prefixes).
    /// Bytes occupied by live entries (including their 2-byte length
    /// prefixes).
    #[inline]
    pub fn count_occupied(&self) -> usize {
        self.len
    }

    /// Reset the arena entirely, invalidating **all** outstanding handles that
    /// reference it. O(1) — the "erase plane" operation.
    /// Reset the arena entirely, invalidating **all** outstanding handles.
    ///
    /// O(1) — this is the "erase plane" operation. The backing allocation
    /// is dropped (capacity goes to zero). All [`Grapheme`] handles
    /// previously obtained from this arena become dangling.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.slots.clear();
        self.sizes.clear();
        self.len = 0;
    }

    /// Resolve a slot to its entry metadata.
    ///
    /// Returns `Err` for out-of-bounds, already-freed, or truncated entries.
    fn try_resolve(&self, slot: Slot) -> Result<Entry, GraphemesError> {
        let start = slot.as_usize() + Self::PREFIX;
        if start > self.count_total() {
            return Err(GraphemesError::OutOfBounds(slot));
        }

        let len = self.try_get_len(slot)?;

        let end = start + len;
        if end > self.count_total() {
            return Err(GraphemesError::OutOfBounds(slot));
        }

        Ok(Entry {
            slot,
            start,
            len,
            end,
        })
    }

    /// Resolve a slot to its entry's length.
    ///
    /// Returns `Err` for out-of-bounds or already-freed slots.
    fn try_get_len(&self, slot: Slot) -> Result<usize, GraphemesError> {
        let one = self
            .inner
            .get(slot.as_usize())
            .ok_or(GraphemesError::OutOfBounds(slot))
            .copied()?;
        let two = self
            .inner
            .get(slot.as_usize() + 1)
            .ok_or(GraphemesError::OutOfBounds(slot))
            .copied()?;

        let len = u16::from_le_bytes([one, two]) as usize;

        if len == 0 {
            return Err(GraphemesError::DoubleRelease(slot));
        }

        Ok(len)
    }

    /// Register a free region in both indexes.
    #[inline]
    fn free_insert(&mut self, slot: usize, size: usize) {
        debug_assert!(size > 0);
        self.slots.insert(slot, size);
        self.sizes.insert((size, slot));
    }

    /// Remove a free region from both indexes.
    #[inline]
    fn free_remove(&mut self, slot: usize, size: usize) {
        self.slots.remove(&slot);
        self.sizes.remove(&(size, slot));
    }

    /// Allocate `needed` contiguous bytes: best-fit reuse, else append.
    fn allocate(&mut self, needed: usize) -> Result<Slot, GraphemesError> {
        // Best-fit: smallest free region with `size >= needed`; ties broken by
        // lowest slot (address-ordered, friendlier to locality).
        if let Some(&(size, slot)) = self.sizes.range((needed, 0)..).next() {
            self.free_remove(slot, size);
            if size > needed {
                // Track the remainder — even a sub-prefix sliver — so it can
                // coalesce later instead of leaking until `clear`.
                self.free_insert(slot + needed, size - needed);
            }
            return Ok(Slot::new(slot as u32));
        }

        let offset = self.count_total();
        let new_len = offset + needed;
        if new_len > Self::MAX_CAPACITY {
            return Err(GraphemesError::Full);
        }
        self.inner.reserve(new_len - self.count_total());
        // SAFETY: capacity >= new_len; try_insert writes all `needed` bytes
        //         before anyone reads them.
        unsafe {
            self.inner.set_len(new_len);
        }
        Ok(Slot::new(offset as u32))
    }
}

impl fmt::Debug for Graphemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graphemes")
            .field("len", &self.count_total())
            .field("live", &self.len)
            .field("free", &self.count_free())
            .field("free_regions", &self.slots.len())
            .field("capacity", &self.inner.capacity())
            .finish()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum GraphemesError {
    #[error("arena is full")]
    Full,

    #[error("requires an arena for {len} bytes")]
    RequiresArena { len: usize },

    #[error("{len} bytes exceeds the maximum entry length ({max} bytes)")]
    TooLong { len: usize, max: usize },

    #[error("{_0:?} is out of bounds")]
    OutOfBounds(Slot),

    #[error("{_0:?} was already freed")]
    DoubleRelease(Slot),

    #[error("{_0:?} not found")]
    NotFound(Slot),

    #[error("invalid")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use crate::Source;
    use super::*;

    /// Cross-check the two free indexes and the `free_bytes == len - count`
    /// invariant after an operation.
    fn check_invariants(arena: &Graphemes) {
        assert_eq!(arena.slots.len(), arena.sizes.len());
        let mut sum = 0;
        for (&slot, &size) in &arena.slots {
            assert!(arena.sizes.contains(&(size, slot)));
            assert!(
                slot + size < arena.count_total(),
                "free region must not touch the tail"
            );
            sum += size;
        }
        assert_eq!(sum, arena.count_free());
    }

    #[test]
    fn new_arena_is_empty() {
        let arena = Graphemes::new();
        assert_eq!(arena.count_occupied(), 0);
        assert_eq!(arena.count_total(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn insert_and_resolve() {
        let mut arena = Graphemes::new();
        let s = "hello, 世界!";
        let g = arena.try_insert(s).unwrap();
        assert_eq!(arena.get(&g), Some(s));
        assert_eq!(arena.count_occupied(), Graphemes::PREFIX + s.len());
        check_invariants(&arena);
    }

    #[test]
    fn insert_multiple() {
        let mut arena = Graphemes::new();
        let entries = [
            "alpha",
            "bravo",
            "charlie",
            "👨\u{200D}👩\u{200D}👧\u{200D}👦",
        ];
        let handles: Vec<_> = entries.iter().map(|s| arena.insert(s)).collect();
        for (h, &expected) in handles.iter().zip(entries.iter()) {
            assert_eq!(arena.get(h), Some(expected));
        }
        check_invariants(&arena);
    }

    #[test]
    fn reuse_interior_slot() {
        let mut arena = Graphemes::new();
        let g1 = arena.try_insert("hello!").unwrap(); // [0,8)
        let _g2 = arena.try_insert("world!").unwrap(); // [8,16) — holds the tail
        let len_before = arena.count_total();

        arena.remove(g1); // interior free (0,8)
        assert_eq!(
            arena.count_total(),
            len_before,
            "interior remove does not shrink len"
        );

        let g3 = arena.try_insert("reuse!").unwrap(); // best-fit -> slot 0
        assert_eq!(Slot::try_from(g3), Slot::try_from(g1));
        assert_eq!(
            arena.count_total(),
            len_before,
            "reuse did not grow the arena"
        );
        assert_eq!(arena.get(&g3), Some("reuse!"));
        check_invariants(&arena);
    }

    #[test]
    fn tail_remove_truncates() {
        let mut arena = Graphemes::new();
        let g1 = arena.try_insert("aaaaaa").unwrap(); // [0,8)
        let g2 = arena.try_insert("bbbbbb").unwrap(); // [8,16)
        assert_eq!(arena.count_total(), 16);

        arena.remove(g2);
        assert_eq!(arena.count_total(), 8, "tail release shrinks len");
        arena.remove(g1);
        assert_eq!(arena.count_total(), 0);
        assert!(arena.is_empty());
        check_invariants(&arena);
    }

    #[test]
    fn coalesce_enables_large_alloc() {
        // Two adjacent 8-byte gaps must merge to fit a 16-byte entry without
        // growing the buffer. (A non-coalescing allocator would append.)
        let mut arena = Graphemes::new();
        let ga = arena.try_insert("aaaaaa").unwrap(); // [0,8)
        let gb = arena.try_insert("bbbbbb").unwrap(); // [8,16)
        let _gc = arena.try_insert("cccccc").unwrap(); // [16,24) tail guard
        let len_before = arena.count_total();

        arena.remove(ga);
        arena.remove(gb); // coalesces -> single free region (0,16)
        assert_eq!(arena.count_free(), 16);

        let big = "DDDDDDDDDDDDDD"; // 14 + 2 = 16 total
        let gd = arena.try_insert(big).unwrap();
        assert_eq!(Slot::try_from(gd), Slot::try_from(ga));
        assert_eq!(
            arena.count_total(),
            len_before,
            "coalesced gap absorbed the entry"
        );
        assert_eq!(arena.get(&gd), Some(big));
        check_invariants(&arena);
    }

    #[test]
    fn best_fit_picks_smallest() {
        let mut arena = Graphemes::new();
        let small = arena.try_insert("ab").unwrap(); // [0,4)
        let _med = arena.try_insert("medium").unwrap(); // [4,12)
        let big = arena.try_insert("bigger-entry!!").unwrap(); // 14+2 -> [12,28)
        let _guard = arena.try_insert("g").unwrap(); // [28,31) tail guard

        arena.remove(small); // free (0,4)
        arena.remove(big); // free (12,16)

        let g = arena.try_insert("xy").unwrap(); // needs 4 -> best-fit (0,4)
        assert_eq!(Slot::try_from(g), Slot::try_from(small));
        check_invariants(&arena);
    }

    #[test]
    fn sub_prefix_sliver_is_not_leaked() {
        // A 2-byte remainder (< Graphemes::PREFIX+1, so never independently usable)
        // must stay trackable and coalesce later, not vanish.
        let mut arena = Graphemes::new();
        let ge = arena.try_insert("EEEEEE").unwrap(); // [0,8)
        let gg = arena.try_insert("g").unwrap(); // [8,11) guard
        arena.remove(ge); // free (0,8)

        let gf = arena.try_insert("FFFF").unwrap(); // 4+2=6 -> slot 0, sliver (6,2)
        assert_eq!(Slot::try_from(gf), Slot::try_from(ge));
        assert_eq!(arena.count_free(), 2, "sliver tracked, not dropped");

        // Releasing the guard coalesces with the sliver and truncates the tail
        // down to the end of F (6). If the sliver had leaked, len would be 8.
        arena.remove(gg);
        assert_eq!(arena.count_total(), 6);
        assert_eq!(arena.count_free(), 0);
        check_invariants(&arena);
    }

    #[test]
    fn churn_returns_to_high_water_mark() {
        let mut arena = Graphemes::new();
        let mut handles = Vec::new();
        for i in 0..200 {
            handles.push(arena.insert(&format!("entry-{i:04}-padding")));
        }
        let peak = arena.count_total();

        for h in handles.drain(..) {
            arena.remove(h);
        }
        assert_eq!(arena.count_total(), 0, "full drain truncates to nothing");

        for i in 0..200 {
            arena.insert(&format!("entry-{i:04}-padding"));
        }
        assert_eq!(
            arena.count_total(),
            peak,
            "no unbounded growth across churn"
        );
        check_invariants(&arena);
    }

    #[test]
    fn clear_resets_everything() {
        let mut arena = Graphemes::new();
        arena.insert("some text here");
        arena.insert("more text");
        let temp = arena.insert("temp");
        arena.remove(temp);

        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.count_total(), 0);
        assert_eq!(arena.count_occupied(), 0);
        assert_eq!(arena.count_free(), 0);
        check_invariants(&arena);
    }

    #[test]
    fn with_capacity_preallocates() {
        let arena = Graphemes::with_capacity(1024);
        assert!(arena.capacity() >= 1024);
        assert_eq!(arena.count_total(), 0);
    }

    #[test]
    fn too_long_is_rejected() {
        let mut arena = Graphemes::new();
        let long = "x".repeat(Graphemes::MAX_LEN + 1);
        assert!(matches!(
            arena.try_insert(&long),
            Err(GraphemesError::TooLong { .. })
        ));
    }
}
