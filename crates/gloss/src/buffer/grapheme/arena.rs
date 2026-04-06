/// An arena for extended grapheme clusters.
///
/// When a grapheme cluster exceeds 4 UTF-8 bytes (e.g., emoji ZWJ sequences),
/// it's stored here and referenced by a 24-bit offset from the
/// [`Grapheme`](Grapheme) handle.
///
/// Inspired by notcurses' `egcpool`, this arena is designed for the specific
/// access pattern of a terminal framebuffer:
///
/// - **Append-first allocation**: new entries are appended to the end (amortised
///   O(1) via doubling growth).
/// - **Free-list reclamation**: released entries are tracked in a free list,
///   enabling O(n_free) gap reuse without scanning the entire arena.
/// - **Bulk clear**: calling [`clear`](Self::clear) resets the entire arena,
///   invalidating all outstanding grapheme handles. This is the "erase plane"
///   operation.
///
/// ## Storage format
///heme_arena.rs
/// Each entry is stored as a 2-byte little-endian length prefix followed by the
/// raw UTF-8 bytes. This avoids the fragility of NUL-terminated scanning (where
/// adjacent released entries could merge) and enables O(1) release.
///
/// ```text
/// [len_lo] [len_hi] [utf8 bytes ...]
/// ```
///
/// The arena is **not** a deduplication interner — each `stash` gets its own
/// slot. This avoids HashMap overhead and keeps the common path (inline
/// graphemes that never touch the arena) at zero cost.
#[derive(Clone)]
pub struct Arena {
    /// Contiguous byte storage. Each entry is a length-prefixed UTF-8 string.
    inner: Vec<u8>,

    /// Bytes actively occupied by stored graphemes (including length prefixes).
    count: usize,

    /// Free list of released regions: `(offset, total_len)` including prefix.
    /// Sorted by size ascending for best-fit allocation.
    free: Vec<Slot>,
}

/// A released region in the arena available for reuse.
#[derive(Debug, Clone, Copy)]
struct Slot {
    offset: usize,
    len: usize,
}

/// Size of the length prefix for each arena entry (2 bytes, little-endian u16).
const PREFIX_SIZE: usize = 2;

/// Minimum allocation size for the arena backing storage.
/// Avoids repeated tiny allocations for the first few extended graphemes.
const MINIMUM_ALLOC: usize = 1024;

/// Maximum string payload that fits in a u16 length prefix.
const MAX_ENTRY_LEN: usize = u16::MAX as usize;

impl Arena {
    pub const EMPTY: Self = Self {
        inner: Vec::new(),
        count: 0,
        free: Vec::new(),
    };
    /// Maximum arena size: 16 MiB. This is the addressable range of the
    /// 24-bit offset stored in an extended [`Grapheme`](crate::Grapheme).
    pub const MAX_CAPACITY: usize = (1 << 24) - 1; // 0x00FF_FFFF = 16,777,215

    /// Create a new, empty arena.
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            count: 0,
            free: Vec::new(),
        }
    }

    /// Create a arena with pre-allocated capacity (in bytes).
    ///
    /// Capacity is clamped to [`MAX_POOL_SIZE`](Self::MAX_CAPACITY).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity.min(Self::MAX_CAPACITY)),
            count: 0,
            free: Vec::new(),
        }
    }

    /// Retrieve a UTF-8 grapheme cluster by offset.
    pub fn get(&self, offset: impl AsOffset) -> &str {
        let offset = offset.as_offset();
        debug_assert!(
            offset + PREFIX_SIZE <= self.inner.len(),
            "arena offset {offset} out of bounds (arena len {})",
            self.inner.len(),
        );

        let str_len = self.get_len(offset);

        debug_assert!(
            str_len > 0,
            "resolving a released or invalid entry at offset {offset}",
        );
        debug_assert!(
            offset + PREFIX_SIZE + str_len <= self.inner.len(),
            "arena entry at {offset} extends past end of arena",
        );

        // SAFETY: We only store valid UTF-8 via `stash`, and the debug
        // assertions above guard against released / corrupt entries.
        unsafe {
            std::str::from_utf8_unchecked(
                &self.inner[offset + PREFIX_SIZE..offset + PREFIX_SIZE + str_len],
            )
        }
    }

    /// Read the string length from the 2-byte LE prefix at `offset`.
    #[inline]
    pub fn get_len(&self, offset: impl AsOffset) -> usize {
        let offset = offset.as_offset();
        u16::from_le_bytes([self.inner[offset], self.inner[offset + 1]]) as usize
    }

    /// Store a length-prefixed UTF-8 grapheme cluster and return its offset.
    ///
    /// This is called by [`Grapheme::encode`](crate::Grapheme::extended) when the
    /// cluster exceeds 4 bytes.
    pub fn insert(&mut self, value: &str) -> Grapheme {
        self.try_insert(value).unwrap()
    }

    /// Inserts a UTF-8 grapheme cluster and returns its offset.
    ///
    /// This is called by [`Grapheme::encode`](crate::Grapheme::extended) when the
    /// cluster exceeds 4 bytes.
    pub fn try_insert(&mut self, value: &str) -> Result<Grapheme, GraphemeError> {
        let len = value.len();
        if len > MAX_ENTRY_LEN {
            return Err(GraphemeError::TooLong {
                len,
                max: MAX_ENTRY_LEN,
            });
        }
        let needed = PREFIX_SIZE + len;
        let offset = self.allocate(needed)?;

        let len_bytes = (len as u16).to_le_bytes();

        let slice = self.inner.as_mut_slice();

        // Write length prefix + payload into the allocated region.
        // For the gap path, the region is already within `self.arena.len()`.
        // For the append path, we extended via `extend_from_slice` in `allocate`.
        slice[offset] = len_bytes[0];
        slice[offset + 1] = len_bytes[1];
        slice[offset + PREFIX_SIZE..offset + needed].copy_from_slice(value.as_bytes());

        self.count += needed;
        Ok(Grapheme::offset(offset))
    }

    /// Remove stored grapheme
    ///
    /// Zeroes the entry and adds the region to the free list.
    pub fn remove(&mut self, offset: impl AsOffset) {
        let offset = offset.as_offset();
        debug_assert!(
            offset + PREFIX_SIZE <= self.inner.len(),
            "releasing out-of-bounds offset {offset}",
        );

        let str_len = self.get_len(offset);

        debug_assert!(
            str_len > 0,
            "double-release detected at offset {offset} (entry_len is 0)",
        );

        let total = PREFIX_SIZE + str_len;

        // Zero only the 2-byte length prefix (stash overwrites the payload).
        self.inner[offset..offset + PREFIX_SIZE].fill(0);
        self.count = self.count.saturating_sub(total);

        // Insert into the free list, maintaining sort by ascending size
        // for best-fit allocation.
        let slot = Slot { offset, len: total };
        let pos = self.free.partition_point(|s| s.len < total);
        self.free.insert(pos, slot);
    }

    /// Reset the arena entirely, invalidating **all** outstanding grapheme
    /// handles that reference this arena.
    ///
    /// This is the "erase plane" operation — fast O(1) via `Vec::clear`.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.free.clear();
        self.count = 0;
    }

    /// Number of bytes actively used by stored graphemes.
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Total byte capacity currently allocated by the arena's backing storage.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Total number of bytes in the arena (including freed gaps).
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the arena has no live entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    // ── Allocation internals ───────────────────────────────────────────

    /// Allocate `needed` contiguous bytes in the arena.
    ///
    /// 1. **Free-list path**: find the smallest free slot that fits (best-fit).
    ///    Leftover space is re-inserted into the free list if large enough.
    /// 2. **Append path**: extend the backing vec with doubling growth.
    /// 3. **Full**: arena has reached `MAX_POOL_SIZE` with no usable gaps.
    fn allocate(&mut self, len: usize) -> Result<usize, GraphemeError> {
        // Path 1: try the free list (best-fit).
        if let Some(offset) = self.allocate_free(len) {
            return Ok(offset);
        }

        // Path 2: append to the end.
        if self.inner.len() + len <= Self::MAX_CAPACITY {
            let offset = self.inner.len();
            self.ensure(offset + len);
            // Extend with zeros; stash will overwrite immediately.
            self.inner.resize(offset + len, 0);
            return Ok(offset);
        }

        // Path 3: no room.
        Err(GraphemeError::Full)
    }

    /// Search the free list for the smallest slot >= `needed` bytes.
    ///
    /// Because the free list is sorted by size, the first match is best-fit.
    /// If the slot is significantly larger, the remainder is re-inserted.
    fn allocate_free(&mut self, needed: usize) -> Option<usize> {
        // Binary search for the first slot with len >= needed.
        let idx = self.free.partition_point(|s| s.len < needed);
        if idx >= self.free.len() {
            return None;
        }

        let slot = self.free.remove(idx);
        let leftover = slot.len - needed;

        // If the leftover is large enough to hold at least a minimal entry
        // (PREFIX_SIZE + 1 byte), re-insert it as a new free slot.
        if leftover >= PREFIX_SIZE + 1 {
            let remainder = Slot {
                offset: slot.offset + needed,
                len: leftover,
            };
            let pos = self.free.partition_point(|s| s.len < leftover);
            self.free.insert(pos, remainder);
        }

        Some(slot.offset)
    }

    /// Ensure the backing vec has capacity for at least `min_capacity` bytes,
    /// using a doubling growth strategy clamped to `MAX_POOL_SIZE`.
     fn ensure(&mut self, min_capacity: usize) {
        if self.inner.capacity() >= min_capacity {
            return;
        }

        // Double current capacity, but at least MINIMUM_ALLOC, at most MAX_POOL_SIZE.
        let target = (self.inner.capacity() * 2)
            .max(MINIMUM_ALLOC)
            .max(min_capacity)
            .min(Self::MAX_CAPACITY);

        self.inner.reserve(target - self.inner.len());
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Arena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphemePool")
            .field("len", &self.inner.len())
            .field("used", &self.count)
            .field("capacity", &self.inner.capacity())
            .field("free_slots", &self.free.len())
            .finish()
    }
}

pub const trait AsOffset {
    #[inline]
    fn as_offset(self) -> usize;
}

impl AsOffset for usize {
    #[inline]
    fn as_offset(self) -> usize {
        self
    }
}

impl AsOffset for Grapheme {
    #[inline]
    fn as_offset(self) -> usize {
        Grapheme::as_offset(&self)
    }
}

impl AsOffset for &Grapheme {
    #[inline]
    fn as_offset(self) -> usize {
        Grapheme::as_offset(&self)
    }
}

impl AsOffset for &mut Grapheme {
    #[inline]
    fn as_offset(self) -> usize {
        Grapheme::as_offset(&self)
    }
}


use packed_struct::prelude::bits::ByteArray;
// ── GraphemePoolError ───────────────────────────────────────────────────────
use crate::Grapheme;
use thiserror::Error;

#[derive_const(Error)]
#[derive(Debug)]
pub enum GraphemeError {
    /// The arena has reached its 16 MiB limit with no reclaimable gaps.
    #[error("grapheme arena is full (16 MiB limit reached)")]
    Full,

    /// The given string exceeds the maximum length encodable.
    #[error("length {len} exceeds maximum ({max} bytes)")]
    TooLong { len: usize, max: usize },

    /// The given string requires an arena to encode.
    #[error("length {len} exceeds maximum ({max} bytes)")]
    ArenaRequired { len: usize, max: usize },

    /// An unexpected error occurred.
    #[error("unknown error")]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_arena_is_empty() {
        let arena = Arena::new();
        assert_eq!(arena.count(), 0);
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn stash_and_resolve() {
        let mut arena = Arena::new();
        let s = "hello, 世界!";

        let offset = arena.try_insert(s).unwrap();
        assert_eq!(arena.get(offset), s);
        assert_eq!(arena.count(), PREFIX_SIZE + s.len());
    }

    #[test]
    fn stash_multiple() {
        let mut arena = Arena::new();
        let entries = [
            "alpha",
            "bravo",
            "charlie",
            "👨\u{200D}👩\u{200D}👧\u{200D}👦",
        ];

        let offsets: Vec<_> = entries.iter().map(|s| arena.try_insert(s).unwrap()).collect();

        for (offset, expected) in offsets.iter().zip(entries.iter()) {
            assert_eq!(arena.get(*offset), *expected);
        }
    }

    #[test]
    fn release_and_reuse_via_free_list() {
        let mut arena = Arena::new();

        let s1 = "hello!"; // 6 bytes + 2 prefix = 8
        let s2 = "world!"; // 6 bytes + 2 prefix = 8
        let offset1 = arena.try_insert(s1).unwrap();
        let _offset2 = arena.try_insert(s2).unwrap();

        let len_before = arena.len();

        // Release s1, creating a free slot.
        arena.remove(offset1);
        assert_eq!(arena.len(), len_before); // arena hasn't shrunk

        // Stash something that fits in the freed slot.
        let s3 = "reuse!"; // same size — should land at offset1
        let offset3 = arena.try_insert(s3).unwrap();
        assert_eq!(offset3, offset1);
        assert_eq!(arena.get(offset3), s3);

        // Pool length should not have grown.
        assert_eq!(arena.len(), len_before);
    }

    #[test]
    fn free_list_best_fit() {
        let mut arena = Arena::new();

        // Create slots of varying sizes.
        let small = arena.try_insert("ab").unwrap(); // 4 total
        let _medium_offset = arena.try_insert("medium").unwrap(); // 8 total
        let large = arena.try_insert("a]larger-entry").unwrap(); // 16 total

        // Release small and large, keeping medium alive.
        arena.remove(small);
        arena.remove(large);

        // A 4-byte request should pick the small slot (best-fit).
        let s = "xy"; // 2 + 2 = 4 total
        let offset = arena.try_insert(s).unwrap();
        assert_eq!(offset, small);
    }

    #[test]
    fn clear_resets_everything() {
        let mut arena = Arena::new();
        arena.try_insert("some text here").unwrap();
        arena.try_insert("more text").unwrap();

        // Release one to populate the free list.
        let off = arena.try_insert("temp").unwrap();
        arena.remove(off);

        assert!(!arena.is_empty());

        arena.clear();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
        assert_eq!(arena.count(), 0);
    }

    #[test]
    fn with_capacity_preallocates() {
        let arena = Arena::with_capacity(1024);
        assert!(arena.capacity() >= 1024);
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn release_is_exact() {
        let mut arena = Arena::new();

        let s1 = "alpha-entry";
        let s2 = "bravo-entry";

        let off1 = arena.try_insert(s1).unwrap();
        let off2 = arena.try_insert(s2).unwrap();

        let total_used = arena.count();
        let s1_cost = PREFIX_SIZE + s1.len();
        let s2_cost = PREFIX_SIZE + s2.len();
        assert_eq!(total_used, s1_cost + s2_cost);

        // Releasing s1 should subtract exactly its cost.
        arena.remove(off1);
        assert_eq!(arena.count(), s2_cost);

        // s2 should still be intact.
        assert_eq!(arena.get(off2), s2);

        // Releasing s2 should bring us to zero.
        arena.remove(off2);
        assert_eq!(arena.count(), 0);
    }

    #[test]
    fn free_list_splits_large_slots() {
        let mut arena = Arena::new();

        // Stash a large entry, then release it.
        let big = "a]]relatively-large-entry-here!"; // 30 bytes + 2 = 32 total
        let big_offset = arena.try_insert(big).unwrap();
        arena.remove(big_offset);

        // Stash something much smaller — should reuse the slot and split.
        let small = "hi"; // 2 + 2 = 4 total
        let small_offset = arena.try_insert(small).unwrap();
        assert_eq!(small_offset, big_offset);

        // The remainder (32 - 4 = 28 bytes) should be on the free list.
        assert_eq!(arena.free.len(), 1);
        assert_eq!(
            arena.free[0].len,
            PREFIX_SIZE + big.len() - (PREFIX_SIZE + small.len())
        );
    }

    #[test]
    fn doubling_growth_strategy() {
        let mut arena = Arena::new();

        // First allocation should jump to at least MINIMUM_ALLOC.
        arena.try_insert("hello").unwrap();
        assert!(
            arena.capacity() >= MINIMUM_ALLOC,
            "expected capacity >= {MINIMUM_ALLOC}, got {}",
            arena.capacity()
        );

        // Subsequent allocations shouldn't cause capacity to grow linearly.
        let cap_after_first = arena.capacity();
        for i in 0..10 {
            arena.try_insert(&format!("entry-{i:04}-some-padding")).unwrap();
        }
        assert!(arena.capacity() <= cap_after_first * 4);
    }

    #[test]
    fn string_too_long_is_rejected() {
        let mut arena = Arena::new();

        // A string just at the limit should succeed.
        // (We can't easily create a 65535-byte string in a test, so test the
        // error path with a mock check.)
        let long = "x".repeat(MAX_ENTRY_LEN + 1);
        let result = arena.try_insert(&long);
        assert!(matches!(
            result,
            Err(GraphemeError::TooLong { .. })
        ));
    }

    #[test]
    fn gap_reclamation_under_pressure() {
        let mut arena = Arena::new();

        let mut offsets = Vec::new();
        for i in 0..100 {
            let s = format!("entry-{i:04}-padding-to-be-longer");
            offsets.push(arena.try_insert(&s).unwrap());
        }

        let len_when_full = arena.len();

        // Release every other entry.
        for i in (0..100).step_by(2) {
            arena.remove(offsets[i]);
        }

        // Pool length unchanged but used is roughly halved.
        assert_eq!(arena.len(), len_when_full);
        assert!(arena.count() < len_when_full);

        // Re-stashing entries of the same size should reuse freed slots.
        let arena_len_before = arena.len();
        for i in (0..100).step_by(2) {
            let s = format!("entry-{i:04}-padding-to-be-longer");
            arena.try_insert(&s).unwrap();
        }
        // Pool should not have grown — everything went into gaps.
        assert_eq!(arena.len(), arena_len_before);
    }
}
