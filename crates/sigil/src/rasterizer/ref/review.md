## Notcurses vs UV TerminalRenderer: Architectural Comparison

### Fundamental model difference

The biggest divergence is **where compositing happens**:

**Notcurses** owns compositing. It maintains N planes with z-ordering, composites them top→bottom into `crender[]`, then diffs against `lastframe[]`. The library *is* the layout engine + renderer.

**UV** is *just* the rasterizer. It receives a pre-composited `RenderBuffer` from the caller (Bubble Tea / Lip Gloss handle layout). There are no planes, no z-index — UV's job starts where Notcurses' Phase 2 starts. This is a deliberate architectural boundary: compositing is the framework's problem, not the renderer's.

### Diffing strategy

**Notcurses**: Cell-level diffing. Every cell in `crender[]` is compared against `lastframe[]`. Damaged cells are emitted; identical cells are skipped. Simple, brute-force, cache-friendly (linear scan over contiguous memory).

**UV**: Two-tier diffing — coarse then fine.
1. **Line-level dirty tracking** via `Touched[]` with `FirstCell`/`LastCell` markers. Untouched lines are skipped entirely — no per-cell comparison needed.
2. **`transformLine()`** then does character-level diffing within dirty lines, walking from both ends inward to find the changed region, minimizing the range that actually gets emitted.

This is classic ncurses-style diffing. The line-level skip is potentially cheaper than Notcurses' full-buffer scan when only a few lines change (common in TUIs with a status bar or single input line updating). The tradeoff: the caller must maintain `Touched` metadata correctly.

UV also has **hash-based scroll detection** (`oldhash`/`newhash`, `scrollOptimize`) — if a block of lines shifted vertically, it can emit `DECSTBM` + `SU`/`SD` instead of rewriting everything. Notcurses' document doesn't describe scroll optimization; it relies on cell-level elision to handle scrolled content (which still works but emits more bytes for large scrolls).

### Elision

Both do **stateful elision** — tracking what the terminal "currently has" to skip redundant escape sequences:

**Notcurses** tracks `last_fg_rgb`, `last_bg_rgb`, `last_attrs` in `RasterState` with explicit elidable flags.

**UV** tracks this in `cursor.Cell` (the pen). `updatePen()` calls `Style.Diff()` which presumably returns only the delta sequences. Same concept, slightly different mechanics — UV delegates diff computation to the `Style` type itself, which is cleaner from a separation-of-concerns perspective.

### Cursor movement

This is where UV really shines with classic curses sophistication. `moveCursor()` tries **four methods** and picks the shortest:

0. Absolute `CUP` (fallback, always works)
1. Relative movement (`CUU`/`CUD`/`CUF`/`CUB`, optionally with VPA/HPA)
2. `CR` + relative movement
3. `Home` + relative movement

Each method is tried with combinatorial variants of hard tabs (`HT`/`CHT`/`CBT`) and backspace, and the method also considers **overwriting existing cell content** as cursor movement (writing the characters that are already there is sometimes shorter than an escape sequence). The `notLocal()` heuristic short-circuits to CUP for large distances.

Notcurses presumably does something similar but the document doesn't detail it. UV's implementation is exhaustive and well-structured.

### Capabilities/terminal detection

**UV** has explicit capability bitflags (`capVPA`, `capHPA`, `capREP`, `capECH`, etc.) with per-terminal profiles in `xtermCaps()`. This is a lightweight terminfo replacement — hardcoded knowledge of what ghostty/kitty/alacritty/etc. support.

**Notcurses** uses terminfo databases (the traditional approach). More complete but heavier dependency.

UV's approach is pragmatic: the set of terminals people actually use is small, and the relevant capability differences between modern terminals are minimal. Hardcoding avoids terminfo parsing entirely.

### Inline/CLI mode

Both support inline (non-fullscreen) rendering. UV's approach is clean — `tRelativeCursor` and `tFullscreen` flags control behavior, with `scrollHeight` tracking how far down the output has gone. Notcurses has similar tracking via `NcPile.scrolls`.

One notable UV detail: `PrependString` physically scrolls the screen and uses `InsertLine` to inject content above the rendered area. This enables "progressive output above a TUI widget" patterns (think: streaming command output above a status bar).

### Verdict

These aren't really competing designs — they solve different problems at different layers:

**Notcurses** = compositor + rasterizer. It gives you planes and z-ordering, which is powerful but means the library owns your entire rendering model. The compositing pass (`paint()`) is an unavoidable O(planes × visible_cells) cost per frame.

**UV** = pure rasterizer. No compositing overhead, no plane abstraction. The dirty-line tracking + line transformation approach is more surgical — it does less work per frame when changes are localized (which they usually are). The cursor movement optimization is best-in-class. The tradeoff is the caller must handle all layout/compositing.

**For your Rust TUI work**, the interesting synthesis would be:

- **UV-style rasterizer** as the output layer (line-level dirty tracking, ncurses-style `transformLine`, exhaustive cursor movement optimization)
- **Notcurses-style planes** as an *optional* higher-level abstraction that composites down to a flat buffer before handing it to the rasterizer
- Keep Notcurses' cell packing insights (inline EGC, channel encoding) for the buffer representation

The key lesson from UV: you don't *need* planes to get excellent rendering performance. Line-level dirty tracking + good elision + smart cursor movement gets you most of the way there with simpler architecture. Planes are about API ergonomics for the application developer, not rendering efficiency.