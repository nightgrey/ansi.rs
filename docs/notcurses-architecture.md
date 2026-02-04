# Notcurses architecture: A Rust developer's guide

**Notcurses is a terminal UI library designed around virtual planes composited via z-ordering, with aggressive output diffing that typically eliminates 90%+ of terminal writes.** The core insight driving its architecture is separating drawing (to virtual buffers) from rasterization (generating escape sequences), enabling both thread-safe concurrent operations and frame-to-frame diffing. This dossier explains the internal mechanics so you can apply similar patterns to your own Rust TUI implementation.

The library represents **the state of the art in terminal rendering optimization**—it achieves near-native graphics performance through careful memory layout decisions (16-byte cells with inline Unicode storage), a two-phase render pipeline, and stateful escape sequence elision. Nick Black's design choices favor memory density and cache efficiency over ergonomic APIs, which makes the C code challenging but the concepts transferable.

---

## Cells: The fundamental unit of the framebuffer

Every visible character in notcurses lives in an `nccell` structure—a carefully packed 16-byte data type that stores a Unicode grapheme cluster, styling information, and foreground/background colors. The structure's size isn't accidental: it's optimized for cache-line efficiency when iterating over framebuffers.

```rust
// notcurses: `nccell`
#[repr(C)]
struct NcCell {
    /// Extended Grapheme Cluster: either inline UTF-8 (≤4 bytes) 
    /// or a 24-bit offset into the plane's EGC pool
    gcluster: u32,
    
    /// Always zero—acts as NUL terminator when gcluster contains inline UTF-8
    gcluster_backstop: u8,
    
    /// Column width (1 for ASCII, 2 for CJK/emoji, 0 means "use 1")
    width: u8,
    
    /// NCSTYLE_* attributes: italic, bold, underline, etc.
    stylemask: u16,
    
    /// Packed foreground (upper 32 bits) and background (lower 32 bits)
    channels: u64,
}
```

The magic is in `gcluster`. Most Unicode characters—including all of Unicode 13—encode to **4 bytes or fewer** in UTF-8. Notcurses exploits this: simple characters are stored directly in the `gcluster` field, with `gcluster_backstop` providing NUL termination. Only complex sequences (emoji with skin tone modifiers, ZWJ sequences) overflow into an external pool.

### Detecting inline vs. pooled storage

```rust
impl NcCell {
    /// notcurses: `cell_extended_p()`
    fn is_extended(&self) -> bool {
        // High byte of 0x01 signals "this is a pool offset, not inline UTF-8"
        (self.gcluster.to_le() & 0xff000000) == 0x01000000
    }
    
    /// notcurses: `cell_egc_idx()`  
    fn egc_pool_offset(&self) -> u32 {
        // Lower 24 bits contain the pool offset
        self.gcluster.to_le() & 0x00ffffff
    }
    
    /// notcurses: `nccell_cols()`
    fn columns(&self) -> u8 {
        if self.width == 0 { 1 } else { self.width }
    }
}
```

This dual-mode encoding means the **common case** (ASCII, Latin, Cyrillic, most CJK) requires zero heap allocation per cell. Only the rare extended grapheme clusters—like 👨‍👩‍👧‍👦 (family emoji, 25 bytes in UTF-8)—hit the pool.

### The channel encoding: 64 bits of color information

The `channels` field packs both foreground and background colors into a single `u64`. This layout enables atomic comparisons during diffing and single-instruction color copying:

```
Upper 32 bits (foreground):
┌──┬──┬──────────┬─────────┬─────────┬─────────┐
│63│62│ 61-60    │   59    │   58    │ 55-32   │
│WA│FD│ FG_ALPHA │ PALETTE │ NOBG    │ FG_RGB  │
└──┴──┴──────────┴─────────┴─────────┴─────────┘

Lower 32 bits (background):  
┌──┬──┬──────────┬─────────┬─────────┐
│31│30│ 29-28    │   27    │  23-0   │
│ 0│BD│ BG_ALPHA │ PALETTE │ BG_RGB  │
└──┴──┴──────────┴─────────┴─────────┘

WA = Wide Asian character marker
FD = Foreground is NOT default color
BD = Background is NOT default color
NOBG = Glyph entirely foreground (no bg needed)
```

```rust
// notcurses: channel manipulation functions
impl NcCell {
    const FG_RGB_MASK: u64 = 0x00ffffff_00000000;
    const BG_RGB_MASK: u64 = 0x00000000_00ffffff;
    const FG_ALPHA_MASK: u64 = 0x30000000_00000000;
    const BG_ALPHA_MASK: u64 = 0x00000000_30000000;
    
    /// notcurses: `ncchannels_fg_rgb()`
    fn fg_rgb(&self) -> u32 {
        ((self.channels & Self::FG_RGB_MASK) >> 32) as u32
    }
    
    /// notcurses: `ncchannels_bg_rgb()`
    fn bg_rgb(&self) -> u32 {
        (self.channels & Self::BG_RGB_MASK) as u32
    }
    
    /// notcurses: `ncchannels_set_fg_rgb8()`
    fn set_fg_rgb(&mut self, r: u8, g: u8, b: u8) {
        let rgb = ((r as u64) << 48) | ((g as u64) << 40) | ((b as u64) << 32);
        self.channels = (self.channels & !Self::FG_RGB_MASK) | rgb;
        // Also set the "not default" bit
        self.channels |= 0x40000000_00000000;
    }
}
```

### Alpha and blending modes

Notcurses defines four alpha states per channel, encoded in 2 bits:

```rust
// notcurses: NCALPHA_* constants
const ALPHA_OPAQUE: u32 = 0x00000000;        // Fully opaque, stops compositing
const ALPHA_BLEND: u32 = 0x10000000;         // Blend with layers below
const ALPHA_TRANSPARENT: u32 = 0x20000000;   // Skip this color entirely  
const ALPHA_HIGHCONTRAST: u32 = 0x30000000;  // FG only: compute from solved BG
```

`ALPHA_HIGHCONTRAST` is particularly clever: the foreground color is computed **after** the background is resolved from compositing, guaranteeing readable text regardless of what's beneath.

### Style bits

The `stylemask` is a straightforward 16-bit bitfield:

```rust
// notcurses: NCSTYLE_* constants
bitflags! {
    struct NcStyle: u16 {
        const STRUCK = 0x0001;    // Strikethrough
        const BOLD = 0x0002;
        const UNDERCURL = 0x0004; // Wavy underline (terminal support varies)
        const UNDERLINE = 0x0008;
        const ITALIC = 0x0010;
    }
}
```

Note: reverse video isn't a style bit—it's achieved by swapping channels via `ncchannels_reverse()`. Blink is handled through `ncplane_pulse()` animation rather than terminal blink.

---

## The EGC pool: Handling complex Unicode

When a grapheme cluster exceeds 4 UTF-8 bytes, notcurses stores it in a per-plane **Extended Grapheme Cluster pool** (`egcpool`). This is essentially a string arena with reference counting.

```rust
// notcurses: `egcpool`
struct EgcPool {
    /// Backing storage (grows as needed, max 16 MiB)
    pool: Vec<u8>,
    
    /// Bytes actively containing EGCs
    pool_used: usize,
    
    /// Next position to probe for free space
    pool_write: usize,
}

const POOL_MAXIMUM_BYTES: usize = 1 << 24; // 16 MiB hard limit
```

### Pool operations

```rust
impl EgcPool {
    /// notcurses: `egcpool_stash()`
    /// Stores a NUL-terminated UTF-8 string, returns offset for cell's gcluster
    fn stash(&mut self, egc: &str) -> Option<u32> {
        let bytes = egc.as_bytes();
        let len = bytes.len() + 1; // Include NUL terminator
        
        // Find space in pool (may need to grow)
        let offset = self.find_space(len)?;
        
        // Copy EGC + NUL into pool
        self.pool[offset..offset + bytes.len()].copy_from_slice(bytes);
        self.pool[offset + bytes.len()] = 0;
        self.pool_used += len;
        
        // Return offset with 0x01 marker in high byte
        Some(0x01000000 | (offset as u32))
    }
    
    /// notcurses: `egcpool_release()`
    /// Frees an EGC by zeroing its storage
    fn release(&mut self, cell: &NcCell) {
        if cell.is_extended() {
            let offset = cell.egc_pool_offset() as usize;
            // Zero out until NUL (frees space for reuse)
            while self.pool[offset] != 0 {
                self.pool[offset] = 0;
                self.pool_used -= 1;
            }
        }
    }
    
    /// notcurses: `egcpool_extended_gcluster()`
    fn get(&self, cell: &NcCell) -> &str {
        let offset = cell.egc_pool_offset() as usize;
        let end = self.pool[offset..].iter().position(|&b| b == 0).unwrap();
        std::str::from_utf8(&self.pool[offset..offset + end]).unwrap()
    }
}
```

**Critical detail**: Calling `ncplane_erase()` destroys the pool, invalidating all cells. If you're holding cell references across an erase, you'll get garbage. Similarly, failing to release cells can exhaust the 16 MiB pool.

---

## Planes: Virtual drawing surfaces

An `ncplane` is notcurses' core abstraction—a rectangular framebuffer of `nccell` structures with its own coordinate system, cursor, and z-position. All drawing happens to planes; the terminal only sees composited output.

```rust
// notcurses: `ncplane` (conceptual—actual struct is internal)
struct NcPlane {
    /// Framebuffer: rows × cols cells stored contiguously
    fb: Vec<NcCell>,
    
    /// Dimensions
    rows: u32,
    cols: u32,
    
    /// Position relative to parent (or absolute if root)
    y: i32,
    x: i32,
    
    /// Virtual cursor position
    cursor_y: u32,
    cursor_x: u32,
    
    /// Default cell for empty positions
    base_cell: NcCell,
    
    /// EGC storage for this plane
    egcpool: EgcPool,
    
    /// Parent plane (None if root of pile)
    parent: Option<*mut NcPlane>,
    
    /// Z-axis links (doubly linked list within pile)
    above: Option<*mut NcPlane>,
    below: Option<*mut NcPlane>,
    
    /// Bound children (ownership tree)
    bound_children: Vec<*mut NcPlane>,
    
    /// Resize callback
    resize_cb: Option<fn(&mut NcPlane) -> i32>,
    
    /// User data
    userptr: *mut c_void,
    
    /// Debug name
    name: Option<String>,
}
```

### Cell storage layout

Cells are stored in row-major order, enabling efficient iteration and cache-friendly access:

```rust
impl NcPlane {
    /// Access cell at (y, x)
    fn cell_at(&self, y: u32, x: u32) -> &NcCell {
        &self.fb[(y * self.cols + x) as usize]
    }
    
    fn cell_at_mut(&mut self, y: u32, x: u32) -> &mut NcCell {
        &mut self.fb[(y * self.cols + x) as usize]
    }
}
```

### Coordinate systems

Notcurses uses **two distinct coordinate spaces**:

1. **Relative coordinates**: Position relative to parent plane. Returned by `ncplane_yx()`.
2. **Absolute coordinates**: Position relative to the pile's origin (typically screen origin). Returned by `ncplane_abs_yx()`.

```rust
impl NcPlane {
    /// notcurses: `ncplane_yx()` - relative to parent
    fn position(&self) -> (i32, i32) {
        (self.y, self.x)
    }
    
    /// notcurses: `ncplane_abs_yx()` - relative to pile origin
    fn absolute_position(&self) -> (i32, i32) {
        let mut abs_y = self.y;
        let mut abs_x = self.x;
        let mut current = self.parent;
        
        while let Some(parent) = current {
            abs_y += unsafe { (*parent).y };
            abs_x += unsafe { (*parent).x };
            current = unsafe { (*parent).parent };
        }
        
        (abs_y, abs_x)
    }
    
    /// notcurses: `ncplane_translate()`
    /// Convert coordinates from src's frame to dst's frame
    fn translate(src: &NcPlane, dst: &NcPlane, y: i32, x: i32) -> (i32, i32) {
        let (src_abs_y, src_abs_x) = src.absolute_position();
        let (dst_abs_y, dst_abs_x) = dst.absolute_position();
        
        (y + src_abs_y - dst_abs_y, x + src_abs_x - dst_abs_x)
    }
}
```

### The standard plane

Every notcurses context has exactly one **standard plane** (`stdplane`) that:

- Always matches terminal dimensions
- Cannot be destroyed, resized, moved, or reparented
- CAN be moved along the z-axis
- Serves as the coordinate reference for absolute positioning

```rust
// notcurses: `notcurses_stdplane()`, `notcurses_stddim_yx()`
impl Notcurses {
    fn stdplane(&self) -> &NcPlane {
        &self.standard_plane
    }
    
    fn stdplane_dimensions(&self) -> (u32, u32) {
        (self.standard_plane.rows, self.standard_plane.cols)
    }
}
```

---

## Z-ordering: How planes stack

Planes within a pile form a **total ordering** along the z-axis. Higher planes obscure lower planes during compositing. New planes are placed at the **top** by default.

```rust
// notcurses: z-axis manipulation functions
impl NcPlane {
    /// notcurses: `ncplane_move_top()` - O(1)
    fn move_to_top(&mut self, pile: &mut NcPile) {
        self.unlink_from_z_order(pile);
        self.above = None;
        self.below = pile.top;
        if let Some(old_top) = pile.top {
            unsafe { (*old_top).above = Some(self) };
        }
        pile.top = Some(self);
    }
    
    /// notcurses: `ncplane_move_bottom()` - O(1)  
    fn move_to_bottom(&mut self, pile: &mut NcPile);
    
    /// notcurses: `ncplane_move_above()` - O(1)
    fn move_above(&mut self, target: &mut NcPlane, pile: &mut NcPile);
    
    /// notcurses: `ncplane_move_below()` - O(1)
    fn move_below(&mut self, target: &mut NcPlane, pile: &mut NcPile);
    
    /// notcurses: `ncplane_above()`, `ncplane_below()` - navigation
    fn above(&self) -> Option<&NcPlane>;
    fn below(&self) -> Option<&NcPlane>;
}
```

### Family operations

When planes have parent-child relationships, "family" operations move the entire subtree:

```rust
// notcurses: `ncplane_move_family_top()` - O(N) where N = family size
// Moving plane C with child E: A B C D E → C E A B D
```

The key insight: **children maintain their relative z-positions** within the family during moves.

---

## Parent-child binding: Coordinated movement

Planes form a **directed acyclic forest** through parent-child bindings. When a plane is bound to a parent:

1. Its coordinates become **relative to the parent**
2. Moving the parent **moves all descendants**
3. Destroying the parent **destroys all descendants**

```rust
// notcurses: `ncplane_options`
struct NcPlaneOptions {
    /// Position relative to parent
    y: i32,
    x: i32,
    
    /// Dimensions (must be positive)
    rows: u32,
    cols: u32,
    
    /// Alignment flags (NCPLANE_OPTION_HORALIGNED, etc.)
    flags: u64,
    
    /// Resize callback
    resize_cb: Option<fn(&mut NcPlane) -> i32>,
    
    /// Margins (for NCPLANE_OPTION_MARGINALIZED)
    margin_b: u32,
    margin_r: u32,
}

// notcurses: option flags
const NCPLANE_OPTION_HORALIGNED: u64 = 0x0001;   // x is ncalign_e
const NCPLANE_OPTION_VERALIGNED: u64 = 0x0002;   // y is ncalign_e  
const NCPLANE_OPTION_MARGINALIZED: u64 = 0x0004; // Use margins
const NCPLANE_OPTION_FIXED: u64 = 0x0008;        // Don't resize with parent
const NCPLANE_OPTION_AUTOGROW: u64 = 0x0010;     // Grow to fit content
const NCPLANE_OPTION_VSCROLL: u64 = 0x0020;      // Enable vertical scrolling
```

### Reparenting

```rust
impl NcPlane {
    /// notcurses: `ncplane_reparent()`
    /// Moves plane to new parent. Children stay with OLD parent.
    fn reparent(&mut self, new_parent: &mut NcPlane);
    
    /// notcurses: `ncplane_reparent_family()`  
    /// Moves plane AND all descendants to new parent.
    fn reparent_family(&mut self, new_parent: &mut NcPlane);
}
```

If you reparent a plane to itself (`ncplane_reparent(n, n)`) and it wasn't already a root, it becomes the **root of a new pile**.

---

## Piles: Independent rendering contexts

A **pile** is a collection of planes with shared z-ordering. The critical property: **different piles can be rendered concurrently** by different threads.

```rust
// notcurses: `ncpile` (internal structure)
struct NcPile {
    /// Z-axis endpoints
    top: Option<*mut NcPlane>,
    bottom: Option<*mut NcPlane>,
    
    /// Root planes (unbound to any parent)
    roots: Vec<*mut NcPlane>,
    
    /// Render state (crender array)
    crender: Vec<CRender>,
    
    /// Dimensions at last render
    dim_y: u32,
    dim_x: u32,
    
    /// Pending scroll lines (CLI mode)
    scrolls: i32,
}
```

### Thread safety rules

| Operation | Safe across piles? | Safe within pile? |
|-----------|-------------------|-------------------|
| Concurrent render | ✓ Yes | ✗ No |
| Output to different planes | ✓ Yes | ✓ Yes |
| Output to same plane | ✓ Yes | ✗ No |
| Add/delete/reorder planes | ✓ Yes | ✗ No |
| Rasterize | Only one at a time globally | — |

### Creating piles

```rust
// notcurses: pile creation methods
impl Notcurses {
    /// notcurses: `ncpile_create()` - explicit creation
    fn create_pile(&mut self, opts: &NcPlaneOptions) -> *mut NcPlane;
}

impl NcPlane {
    /// Reparenting to self creates a new pile (if not already root)
    fn become_pile_root(&mut self) {
        ncplane_reparent(self, self);
    }
}
```

A pile is destroyed when its last plane is destroyed or reparented elsewhere. The **standard pile** always exists because the standard plane cannot be destroyed.

---

## The rendering pipeline: Planes to terminal output

Notcurses separates rendering into two distinct phases, enabling both concurrent operation and efficient diffing.

### Phase 1: Render (compositing)

`ncpile_render()` composites all planes in a pile into a single `crender` array—one entry per screen cell:

```rust
// notcurses: `crender` (per-cell render state)
struct CRender {
    /// Final computed cell
    cell: NcCell,
    
    /// Source plane that provided the glyph
    source_plane: Option<*const NcPlane>,
    
    /// Damage flag: true if cell differs from lastframe
    damaged: bool,
    
    /// High-contrast pre-computed foreground
    hc_fg: u32,
    
    /// Blend counts for proper alpha mixing
    fg_blends: u8,
    bg_blends: u8,
}
```

### The compositing algorithm

```rust
// notcurses: `paint()` - conceptual implementation
fn paint(pile: &NcPile, crender: &mut [CRender]) {
    // For each screen position...
    for y in 0..pile.dim_y {
        for x in 0..pile.dim_x {
            let cell_idx = (y * pile.dim_x + x) as usize;
            let mut cr = &mut crender[cell_idx];
            
            // Traverse planes top-to-bottom
            let mut plane = pile.top;
            let mut egc_locked = false;
            let mut fg_locked = false;
            let mut bg_locked = false;
            
            while let Some(p) = plane {
                let p = unsafe { &*p };
                
                // Check if plane intersects this cell
                if let Some(cell) = p.cell_at_screen_position(y, x) {
                    // EGC: first non-empty wins
                    if !egc_locked && cell.gcluster != 0 {
                        cr.cell.gcluster = cell.gcluster;
                        cr.cell.stylemask = cell.stylemask;
                        cr.cell.width = cell.width;
                        cr.source_plane = Some(p);
                        egc_locked = true;
                    }
                    
                    // Foreground color: depends on alpha
                    if !fg_locked {
                        match cell.fg_alpha() {
                            ALPHA_OPAQUE => {
                                cr.cell.set_fg(cell.fg_rgb());
                                fg_locked = true;
                            }
                            ALPHA_BLEND => {
                                cr.cell.blend_fg(cell.fg_rgb());
                                cr.fg_blends += 1;
                            }
                            ALPHA_TRANSPARENT => { /* skip */ }
                            ALPHA_HIGHCONTRAST => {
                                cr.hc_fg = cell.fg_rgb();
                                // Resolved in postpaint()
                            }
                        }
                    }
                    
                    // Background color: same logic
                    if !bg_locked {
                        // ... similar to foreground
                    }
                }
                
                // Stop if everything locked
                if egc_locked && fg_locked && bg_locked {
                    break;
                }
                
                plane = p.below;
            }
        }
    }
}
```

### Phase 2: Rasterize (output generation)

`ncpile_rasterize()` compares the `crender` array against `lastframe` and generates minimal escape sequences:

```rust
// notcurses: `lastframe` tracking in main context
struct Notcurses {
    /// Previous frame buffer for diffing
    lastframe: Vec<NcCell>,
    
    /// Dimensions of lastframe
    lf_dim_y: u32,
    lf_dim_x: u32,
    
    /// EGC pool for lastframe
    lastframe_pool: EgcPool,
}
```

### Rasterization state machine

```rust
// notcurses: `rasterstate` - tracks terminal state for elision
struct RasterState {
    /// Output buffer
    output: String,
    
    /// Current cursor position
    cursor_y: i32,
    cursor_x: i32,
    
    /// Last emitted colors (for elision)
    last_fg_rgb: (u8, u8, u8),
    last_bg_rgb: (u8, u8, u8),
    
    /// Last emitted attributes
    last_attrs: NcStyle,
    
    /// Elision flags
    fg_elidable: bool,
    bg_elidable: bool,
}
```

### The diffing algorithm

```rust
// notcurses: rasterization diffing (conceptual)
fn rasterize(
    crender: &[CRender], 
    lastframe: &mut [NcCell],
    state: &mut RasterState
) {
    for (i, cr) in crender.iter().enumerate() {
        let last = &lastframe[i];
        
        // Cell-level diff: if identical, skip entirely
        if cells_equal(&cr.cell, last) && !cr.damaged {
            continue; // ELIDED - no output generated
        }
        
        // Position cursor (may elide if already there)
        let y = i / width;
        let x = i % width;
        move_cursor_to(state, y, x);
        
        // Set foreground color (elide if unchanged)
        if cr.cell.fg_rgb() != state.last_fg_rgb || !state.fg_elidable {
            emit_fg_color(state, cr.cell.fg_rgb());
            state.fg_elidable = true;
        }
        
        // Set background color (elide if unchanged)
        if cr.cell.bg_rgb() != state.last_bg_rgb || !state.bg_elidable {
            emit_bg_color(state, cr.cell.bg_rgb());
            state.bg_elidable = true;
        }
        
        // Set attributes (elide if unchanged)
        if cr.cell.stylemask != state.last_attrs {
            emit_styles(state, cr.cell.stylemask);
        }
        
        // Emit the glyph
        emit_glyph(state, &cr.cell);
        
        // Update lastframe
        lastframe[i] = cr.cell.clone();
    }
}
```

### Performance characteristics

Typical elision rates demonstrate the approach's effectiveness:

- **Cell elision**: 90%+ (most cells don't change frame-to-frame)
- **FG color elision**: ~30-35%
- **BG color elision**: ~35-40%

The statistics are tracked in `ncstats`:

```rust
// notcurses: `ncstats`
struct NcStats {
    cell_elisions: u64,    // Cells skipped entirely
    cell_emissions: u64,   // Cells actually written
    fg_elisions: u64,      // FG color changes skipped
    fg_emissions: u64,     // FG color changes emitted
    bg_elisions: u64,      // BG color changes skipped  
    bg_emissions: u64,     // BG color changes emitted
}
```

---

## Text rendering and wide characters

Notcurses handles Unicode through the Extended Grapheme Cluster (EGC) concept from Unicode Standard Annex #29. The library uses **libunistring's `uc_is_grapheme_break()`** to segment input.

### Width calculation

```rust
// notcurses: `ncstrwidth()`, `utf8_egc_len()`
fn egc_width(egc: &str) -> (usize, usize) {
    // Returns (byte_count, column_count)
    // Uses wcwidth() internally for each codepoint
    // Handles ZWJ sequences, variation selectors, combining chars
}

impl NcPlane {
    /// notcurses: `ncplane_putstr()`
    fn put_str(&mut self, s: &str) -> i32 {
        let mut cols_written = 0;
        
        for egc in grapheme_clusters(s) {
            let width = egc_display_width(egc);
            
            // Write to current cursor position
            self.cell_at_mut(self.cursor_y, self.cursor_x)
                .set_egc(egc, width);
            
            // Handle wide characters: mark secondary cells
            for i in 1..width {
                self.cell_at_mut(self.cursor_y, self.cursor_x + i)
                    .gcluster = 0; // "continuation" of wide char
            }
            
            // Advance cursor
            self.cursor_x += width;
            cols_written += width as i32;
        }
        
        cols_written
    }
}
```

### Wide character rules

Notcurses enforces terminal limitations strictly:

1. **Wide characters cannot be split** across columns—no partial glyphs
2. **Writing to rightmost column**: Wide chars cannot start at the final column
3. **Overwriting rules**: Writing onto the right cell of a wide char destroys the entire glyph
4. **Compositing**: A higher plane bisecting a wide glyph obliterates it entirely

```rust
// Example: destroying a wide character
impl NcPlane {
    fn put_char_at(&mut self, y: u32, x: u32, ch: char) {
        let target = self.cell_at_mut(y, x);
        
        // Check if we're hitting the right side of a wide char
        if x > 0 {
            let left = self.cell_at(y, x - 1);
            if left.width > 1 && left.gcluster != 0 {
                // Destroy the wide char
                self.cell_at_mut(y, x - 1).gcluster = ' ' as u32;
            }
        }
        
        target.gcluster = ch as u32;
        target.width = 1;
    }
}
```

### Text wrapping

The `ncplane_puttext()` function handles multi-line, aligned, wrapped text:

```rust
// notcurses: `ncplane_puttext()`
fn put_text(&mut self, y: i32, align: NcAlign, text: &str) -> usize {
    // Current implementation uses isspace() for word breaking
    // Note: Does NOT use full Unicode Line Breaking (UAX#14)
    // This is a known limitation (GitHub issue #772)
}

enum NcAlign {
    Left,
    Center, 
    Right,
}
```

---

## CLI/Inline mode: Scrolling terminal integration

CLI mode allows notcurses to work as a scrolling shell utility rather than a fullscreen application. Activated via:

```rust
// notcurses: `NCOPTION_CLI_MODE` (convenience macro)
const NCOPTION_CLI_MODE: u64 = 
    NCOPTION_NO_ALTERNATE_SCREEN |    // Stay on main screen
    NCOPTION_NO_CLEAR_BITMAPS |       // Don't clear images
    NCOPTION_PRESERVE_CURSOR |         // Match physical cursor
    NCOPTION_SCROLLING;                // Enable scrolling
```

### How CLI mode differs

| Aspect | Fullscreen mode | CLI mode |
|--------|-----------------|----------|
| Screen buffer | Alternate screen | Main screen |
| Cursor | Hidden, managed internally | Visible, preserved |
| Scrollback | None | Preserved |
| Standard plane | Static | Scrolls with content |
| Exit behavior | Restores original | Cursor at end of output |

### Scroll tracking

In CLI mode, notcurses tracks how many lines have scrolled to maintain cursor positioning:

```rust
struct NcPile {
    /// Lines scrolled since last render (CLI mode only)
    scrolls: i32,
}

// After rasterization in CLI mode:
// 1. Physical cursor placed at "logical end" of output
// 2. Accounts for any scrolling that occurred
// 3. Standard plane's virtual cursor updated to match
```

### Restrictions

Only the **standard pile** can be rasterized in CLI mode. Other piles can be used for off-screen rendering, then content copied or planes reparented.

---

## Putting it together: The notcurses architecture

The complete data flow:

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Code                                │
│  ncplane_putstr(), ncplane_box(), etc.                         │
└──────────────────────────┬──────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     ncplane Framebuffers                        │
│  Each plane: rows × cols of nccell + egcpool                   │
│  Organized into piles with z-ordering                          │
└──────────────────────────┬──────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                   ncpile_render()                               │
│  Composite planes top→bottom into crender[]                    │
│  Handle alpha blending, high-contrast, wide chars              │
└──────────────────────────┬──────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                  ncpile_rasterize()                             │
│  Diff crender[] against lastframe[]                            │
│  Generate minimal escape sequences                              │
│  Track color/style state for elision                           │
└──────────────────────────┬──────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Terminal Output                             │
│  ~10% of cells actually emitted                                │
│  Colors/styles elided when unchanged                           │
└─────────────────────────────────────────────────────────────────┘
```

### Key architectural decisions to apply in Rust

1. **Pack cells tightly**: The 16-byte `nccell` with inline EGC storage minimizes cache misses during rendering. Consider `#[repr(C)]` structs with careful field ordering.

2. **Separate render from rasterize**: This enables concurrent rendering of independent regions and cleaner testing (render output is inspectable before terminal output).

3. **Store lastframe for O(1) diffing**: The memory cost of a full frame buffer pays for itself in output reduction.

4. **Track terminal state during rasterization**: Color/style elision requires knowing what the terminal currently has set. A state machine approach works well.

5. **Use pool allocation for extended strings**: The egcpool pattern avoids per-cell heap allocations while handling arbitrary Unicode.

6. **Make z-ordering explicit**: A total ordering (linked list) enables O(1) plane moves and clear compositing semantics.

7. **Coordinate systems matter**: Distinguish relative (to parent) from absolute (to screen) coordinates. Provide translation functions.

These patterns form the foundation of a high-performance terminal UI library that minimizes both memory usage and terminal I/O.