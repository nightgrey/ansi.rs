/// A per-plane byte arena for extended grapheme clusters.
///
/// When a grapheme cluster exceeds 4 UTF-8 bytes (e.g., emoji ZWJ sequences),
/// it's stored here and referenced by a 24-bit offset from the
/// [`Grapheme`](crate::Grapheme) handle.
///
/// Inspired by notcurses' `egcpool`, this pool is designed for the specific
/// access pattern of a terminal framebuffer:
///
/// - **Append-first allocation**: new entries are appended to the end (amortised
///   O(1) via doubling growth).
/// - **Gap reclamation**: when the pool approaches its 16 MiB limit, freed
///   regions (zeroed by [`release`](Self::release)) are reclaimed via a linear
///   scan from a write cursor.
/// - **Bulk clear**: calling [`clear`](Self::clear) resets the entire pool,
///   invalidating all outstanding grapheme handles. This is the "erase plane"
///   operation.
///
/// ## Storage format
///
/// Each entry is stored as a 2-byte little-endian length prefix followed by the
/// raw UTF-8 bytes. This avoids the fragility of NUL-terminated scanning (where
/// adjacent released entries could merge) and enables O(1) release via a single
/// `fill(0)`.
///
/// ```text
/// [len_lo] [len_hi] [utf8 bytes ...]
/// ```
///
/// The pool is **not** a deduplication interner — each `stash` gets its own
/// slot. This avoids HashMap overhead and keeps the common path (inline
/// graphemes that never touch the pool) at zero cost.
pub struct GraphemePool {
    /// Contiguous byte storage. Each entry is a length-prefixed UTF-8 string.
    pool: Vec<u8>,

    /// Bytes actively occupied by stored graphemes (including length prefixes).
    used: usize,

    /// Write cursor for gap scanning. When the pool is near capacity,
    /// the allocator scans forward from here looking for contiguous zero
    /// regions to reuse.
    write_cursor: usize,
}

/// Size of the length prefix for each pool entry (2 bytes, little-endian u16).
const PREFIX_SIZE: usize = 2;

/// Minimum allocation size for the pool backing storage.
/// Avoids repeated tiny allocations for the first few extended graphemes.
const MINIMUM_ALLOC: usize = 1024;

impl GraphemePool {
    /// Maximum pool size: 16 MiB. This is the addressable range of the
    /// 24-bit offset stored in an extended [`Grapheme`](crate::Grapheme).
    pub const MAX_POOL_SIZE: usize = (1 << 24) - 1; // 0x00FF_FFFF = 16,777,215

    /// Create a new, empty pool.
    pub fn new() -> Self {
        Self {
            pool: Vec::new(),
            used: 0,
            write_cursor: 0,
        }
    }

    /// Create a pool with pre-allocated capacity (in bytes).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity.min(Self::MAX_POOL_SIZE)),
            used: 0,
            write_cursor: 0,
        }
    }

    /// Store a length-prefixed UTF-8 grapheme cluster and return its offset.
    ///
    /// This is called by [`Grapheme::new`](crate::Grapheme::new) when the
    /// cluster exceeds 4 bytes.
    pub fn stash(&mut self, s: &str) -> Result<usize, GraphemePoolError> {
        let str_len = s.len();
        let needed = PREFIX_SIZE + str_len;
        let offset = self.allocate(needed)?;

        // Write length prefix (little-endian u16).
        let len_bytes = (str_len as u16).to_le_bytes();
        self.pool[offset] = len_bytes[0];
        self.pool[offset + 1] = len_bytes[1];

        // Write UTF-8 payload.
        self.pool[offset + PREFIX_SIZE..offset + needed].copy_from_slice(s.as_bytes());
        self.used += needed;

        Ok(offset)
    }

    /// Resolve a pool offset to a `&str`.
    ///
    /// # Safety contract
    ///
    /// The offset must have been returned by a prior `stash` call on this pool,
    /// and the entry must not have been released. This is upheld internally by
    /// [`Grapheme::resolve`](crate::Grapheme::resolve).
    pub fn resolve(&self, offset: usize) -> &str {
        debug_assert!(
            offset + PREFIX_SIZE <= self.pool.len(),
            "pool offset out of bounds"
        );

        let str_len = self.entry_len(offset);

        debug_assert!(
            offset + PREFIX_SIZE + str_len <= self.pool.len(),
            "pool entry extends past end of pool"
        );

        // SAFETY: We only store valid UTF-8 via `stash`.
        unsafe {
            std::str::from_utf8_unchecked(
                &self.pool[offset + PREFIX_SIZE..offset + PREFIX_SIZE + str_len],
            )
        }
    }

    /// Release storage at the given offset by zeroing the entire entry.
    ///
    /// The freed region becomes available for future allocations when gap
    /// reclamation runs. O(1) thanks to the length prefix — no scanning needed.
    pub fn release(&mut self, offset: usize) {
        debug_assert!(
            offset + PREFIX_SIZE <= self.pool.len(),
            "releasing out-of-bounds offset"
        );

        let str_len = self.entry_len(offset);
        let total = PREFIX_SIZE + str_len;

        // Zero the entire entry (prefix + payload).
        self.pool[offset..offset + total].fill(0);
        self.used = self.used.saturating_sub(total);

        // Hint the write cursor: if this freed region starts before the
        // current cursor, move the cursor back to allow earlier reuse.
        if offset < self.write_cursor {
            self.write_cursor = offset;
        }
    }

    /// Reset the pool entirely, invalidating **all** outstanding grapheme
    /// handles that reference this pool.
    ///
    /// This is the "erase plane" operation — fast O(1) via `Vec::clear`.
    pub fn clear(&mut self) {
        self.pool.clear();
        self.used = 0;
        self.write_cursor = 0;
    }

    /// Number of bytes actively used by stored graphemes.
    #[inline]
    pub fn used(&self) -> usize {
        self.used
    }

    /// Total byte capacity currently allocated by the pool's backing storage.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.pool.capacity()
    }

    /// Total number of bytes in the pool (including freed gaps).
    #[inline]
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Returns `true` if the pool has no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Read the string length from the 2-byte LE prefix at `offset`.
    #[inline]
    fn entry_len(&self, offset: usize) -> usize {
        u16::from_le_bytes([self.pool[offset], self.pool[offset + 1]]) as usize
    }

    // ── Allocation internals ───────────────────────────────────────────

    /// Allocate `needed` contiguous bytes in the pool.
    ///
    /// Fast path: append to the end with doubling growth (amortised O(1)).
    /// Slow path: scan for a contiguous gap of freed (zero) bytes.
    fn allocate(&mut self, needed: usize) -> Result<usize, GraphemePoolError> {
        // Fast path: space to append.
        if self.pool.len() + needed <= Self::MAX_POOL_SIZE {
            let offset = self.pool.len();

            // Doubling growth strategy (à la notcurses' egcpool_grow).
            let min_capacity = offset + needed;
            let target_capacity = self
                .pool
                .capacity()
                .max(MINIMUM_ALLOC)
                .max(self.pool.len() * 2)
                .min(Self::MAX_POOL_SIZE)
                .max(min_capacity);

            if self.pool.capacity() < target_capacity {
                self.pool.reserve(target_capacity - self.pool.len());
            }

            self.pool.resize(offset + needed, 0);
            return Ok(offset);
        }

        // Slow path: pool is at or near capacity — scan for a gap.
        self.find_gap(needed).ok_or(GraphemePoolError::Full)
    }

    /// Scan the pool for a contiguous run of `needed` zero bytes.
    ///
    /// Scans forward from the write cursor, then from offset 0 if nothing
    /// was found. Does **not** wrap mid-gap, avoiding the bug where a gap
    /// straddling the pool boundary would produce a non-contiguous region.
    fn find_gap(&mut self, needed: usize) -> Option<usize> {
        let len = self.pool.len();
        if len == 0 || needed > len {
            return None;
        }

        let start = self.write_cursor.min(len);

        // Try twice: first from the cursor, then from the beginning.
        for scan_start in [start, 0] {
            let mut i = scan_start;

            while i + needed <= len {
                if self.pool[i] == 0 {
                    // Count how many contiguous zero bytes starting at i.
                    let gap_len = self.pool[i..]
                        .iter()
                        .take(needed)
                        .take_while(|&&b| b == 0)
                        .count();

                    if gap_len >= needed {
                        self.write_cursor = i + needed;
                        return Some(i);
                    }

                    // Skip past the partial gap.
                    i += gap_len;
                } else {
                    // Skip past this live entry. Read its length prefix if
                    // it looks like a valid entry, otherwise advance byte by byte.
                    if i + PREFIX_SIZE <= len {
                        let entry_str_len = self.entry_len(i);
                        if entry_str_len > 0 && i + PREFIX_SIZE + entry_str_len <= len {
                            i += PREFIX_SIZE + entry_str_len;
                            continue;
                        }
                    }
                    i += 1;
                }
            }

            // Don't re-scan the same range.
            if scan_start == 0 {
                break;
            }
        }

        None
    }
}

impl Default for GraphemePool {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for GraphemePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphemePool")
            .field("len", &self.pool.len())
            .field("used", &self.used)
            .field("capacity", &self.pool.capacity())
            .finish()
    }
}

// ── GraphemePoolError ───────────────────────────────────────────────────────
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphemePoolError {
    /// Error returned when the [`GraphemePool`] cannot allocate space for an
    /// extended grapheme cluster (pool has reached its 16 MiB limit with no
    /// reclaimable gaps).
    #[error("grapheme pool is full (16 MiB limit reached)")]
    Full,
    #[error("unknown error")]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_is_empty() {
        let pool = GraphemePool::new();
        assert_eq!(pool.used(), 0);
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn stash_and_resolve() {
        let mut pool = GraphemePool::new();
        let s = "hello, 世界!";

        let offset = pool.stash(s).unwrap();
        assert_eq!(pool.resolve(offset), s);
        assert_eq!(pool.used(), PREFIX_SIZE + s.len());
    }

    #[test]
    fn stash_multiple() {
        let mut pool = GraphemePool::new();
        let entries = ["alpha", "bravo", "charlie", "👨\u{200D}👩\u{200D}👧\u{200D}👦"];

        let offsets: Vec<_> = entries.iter().map(|s| pool.stash(s).unwrap()).collect();

        for (offset, expected) in offsets.iter().zip(entries.iter()) {
            assert_eq!(pool.resolve(*offset), *expected);
        }
    }

    #[test]
    fn release_and_reuse() {
        let mut pool = GraphemePool::new();

        let s1 = "hello!"; // 6 bytes + 2 prefix = 8
        let offset1 = pool.stash(s1).unwrap();
        let used_after_s1 = pool.used();
        assert_eq!(used_after_s1, PREFIX_SIZE + s1.len());

        // Release s1.
        pool.release(offset1);
        assert_eq!(pool.used(), 0);

        // The pool length hasn't shrunk (we don't truncate), but the space
        // is reclaimable once append-space runs out.
        assert_eq!(pool.len(), used_after_s1);
    }

    #[test]
    fn clear_resets_everything() {
        let mut pool = GraphemePool::new();
        pool.stash("some text here").unwrap();
        pool.stash("more text").unwrap();
        assert!(!pool.is_empty());

        pool.clear();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
        assert_eq!(pool.used(), 0);
    }

    #[test]
    fn with_capacity_preallocates() {
        let pool = GraphemePool::with_capacity(1024);
        assert!(pool.capacity() >= 1024);
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn gap_reclamation_under_pressure() {
        // Simulate pool near capacity: fill, release some, then insert again.
        let mut pool = GraphemePool::new();

        // Fill with some entries.
        let mut offsets = Vec::new();
        for i in 0..100 {
            let s = format!("entry-{i:04}-padding-to-be-longer");
            offsets.push(pool.stash(&s).unwrap());
        }

        let len_when_full = pool.len();

        // Release every other entry.
        for i in (0..100).step_by(2) {
            pool.release(offsets[i]);
        }

        // Pool length unchanged but used is roughly halved.
        assert_eq!(pool.len(), len_when_full);
        assert!(pool.used() < len_when_full);
    }

    #[test]
    fn doubling_growth_strategy() {
        let mut pool = GraphemePool::new();

        // First allocation should jump to at least MINIMUM_ALLOC.
        pool.stash("hello").unwrap();
        assert!(
            pool.capacity() >= MINIMUM_ALLOC,
            "expected capacity >= {MINIMUM_ALLOC}, got {}",
            pool.capacity()
        );

        // Subsequent allocations shouldn't cause capacity to grow linearly.
        let cap_after_first = pool.capacity();
        for i in 0..10 {
            pool.stash(&format!("entry-{i:04}-some-padding")).unwrap();
        }
        // Should have at most doubled, not grown 10 times.
        assert!(pool.capacity() <= cap_after_first * 4);
    }

    #[test]
    fn release_is_exact() {
        let mut pool = GraphemePool::new();

        let s1 = "alpha-entry";
        let s2 = "bravo-entry";

        let off1 = pool.stash(s1).unwrap();
        let off2 = pool.stash(s2).unwrap();

        let total_used = pool.used();
        let s1_cost = PREFIX_SIZE + s1.len();
        let s2_cost = PREFIX_SIZE + s2.len();
        assert_eq!(total_used, s1_cost + s2_cost);

        // Releasing s1 should subtract exactly its cost.
        pool.release(off1);
        assert_eq!(pool.used(), s2_cost);

        // s2 should still be intact.
        assert_eq!(pool.resolve(off2), s2);

        // Releasing s2 should bring us to zero.
        pool.release(off2);
        assert_eq!(pool.used(), 0);
    }
}