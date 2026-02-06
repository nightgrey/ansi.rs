use  super::GraphemePoolError;

/// A per-plane byte arena for extended grapheme clusters.
///
/// When a grapheme cluster exceeds 4 UTF-8 bytes (e.g., emoji ZWJ sequences),
/// it's stored here as a NUL-terminated string and referenced by a 24-bit
/// offset from the [`Grapheme`](crate::Grapheme) handle.
///
/// Inspired by notcurses' `egcpool`, this pool is designed for the specific
/// access pattern of a terminal framebuffer:
///
/// - **Append-first allocation**: new entries are appended to the end (O(1)).
/// - **Gap reclamation**: when the pool approaches its 16 MiB limit, freed
///   regions (zeroed by [`release`](Self::release)) are reclaimed via a linear
///   scan from a write cursor.
/// - **Bulk clear**: calling [`clear`](Self::clear) resets the entire pool,
///   invalidating all outstanding grapheme handles. This is the "erase plane"
///   operation.
///
/// The pool is **not** a deduplication interner — each `insert` gets its own
/// slot. This avoids HashMap overhead and keeps the common path (inline
/// graphemes that never touch the pool) at zero cost.
pub struct GraphemePool {
    /// Contiguous byte storage. Each entry is a NUL-terminated UTF-8 string.
    pool: Vec<u8>,

    /// Bytes actively occupied by stored graphemes (excluding freed gaps).
    used: usize,

    /// Write cursor for gap scanning. When the pool is near capacity,
    /// the allocator scans forward from here looking for contiguous zero
    /// regions to reuse.
    write_cursor: usize,
}

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

    /// Store a NUL-terminated UTF-8 grapheme cluster and return its offset.
    ///
    /// This is called by [`Grapheme::new`](crate::Grapheme::new) when the
    /// cluster exceeds 4 bytes.
    pub fn stash(&mut self, s: &str) -> Result<usize, GraphemePoolError> {
        let needed = s.len() + 1; // +1 for NUL terminator
        let offset = self.allocate(needed)?;

        self.pool[offset..offset + s.len()].copy_from_slice(s.as_bytes());
        self.pool[offset + s.len()] = 0; // NUL terminator
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
        debug_assert!(offset < self.pool.len(), "pool offset out of bounds");

        // Find the NUL terminator.
        let end = self.pool[offset..]
            .iter()
            .position(|&b| b == 0)
            .map(|i| offset + i)
            .unwrap_or(self.pool.len());

        // SAFETY: We only store valid UTF-8 via `stash`.
        unsafe { std::str::from_utf8_unchecked(&self.pool[offset..end]) }
    }

    /// Release storage at the given offset by zeroing it out.
    ///
    /// The freed region becomes available for future allocations when gap
    /// reclamation runs.
    pub fn release(&mut self, offset: usize) {
        debug_assert!(offset < self.pool.len(), "releasing out-of-bounds offset");

        let mut i = offset;
        while i < self.pool.len() && self.pool[i] != 0 {
            self.pool[i] = 0;
            i += 1;
        }

        // Also clear the NUL terminator itself so the region is fully zeroed.
        if i < self.pool.len() {
            self.pool[i] = 0;
        }

        let freed = i - offset + 1;
        self.used = self.used.saturating_sub(freed);

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

    // ── Allocation internals ───────────────────────────────────────────

    /// Allocate `needed` contiguous bytes in the pool.
    ///
    /// Fast path: append to the end (O(1)).
    /// Slow path: scan for a contiguous gap of freed (zero) bytes.
    fn allocate(&mut self, needed: usize) -> Result<usize, GraphemePoolError> {
        // Fast path: space to append.
        if self.pool.len() + needed <= Self::MAX_POOL_SIZE {
            let offset = self.pool.len();
            self.pool.resize(offset + needed, 0);
            return Ok(offset);
        }

        // Slow path: pool is at or near capacity — scan for a gap.
        self.find_gap(needed).ok_or(GraphemePoolError::Full)
    }

    /// Scan the pool for a contiguous run of `needed` zero bytes, starting
    /// from the write cursor and wrapping around once.
    fn find_gap(&mut self, needed: usize) -> Option<usize> {
        let len = self.pool.len();
        if len == 0 || needed > len {
            return None;
        }

        let start = self.write_cursor % len;
        let mut scanned = 0;
        let mut pos = start;
        let mut gap_start: Option<usize> = None;
        let mut gap_len = 0;

        while scanned < len {
            if self.pool[pos] == 0 {
                if gap_start.is_none() {
                    gap_start = Some(pos);
                    gap_len = 0;
                }
                gap_len += 1;
                if gap_len >= needed {
                    let offset = gap_start.unwrap();
                    self.write_cursor = offset + needed;
                    return Some(offset);
                }
            } else {
                gap_start = None;
                gap_len = 0;
            }

            pos = (pos + 1) % len;
            scanned += 1;
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
        assert_eq!(pool.used(), s.len() + 1); // +1 for NUL
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

        let s1 = "hello!"; // 6 bytes + NUL = 7
        let offset1 = pool.stash(s1).unwrap();
        let used_after_s1 = pool.used();

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

        // Pool length unchanged but used is halved.
        assert_eq!(pool.len(), len_when_full);
        assert!(pool.used() < len_when_full);
    }
}