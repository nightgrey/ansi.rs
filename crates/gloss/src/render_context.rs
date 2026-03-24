use unicode_width::UnicodeWidthChar;
use ansi::{Attribute, Color, Style};
use geometry::{Bounded, Position, Point, Rect, Transform, Intersect, Contains, Sides};
use sigil::{Buffer, Cell, Grapheme};
use crate::{BorderStyle};

/// Snapshot of all context state, pushed/popped via save/restore.
#[derive(Debug, Clone)]
struct ContextState {
    clip: Rect,
    origin: Point,
    style: Style,
    fill_char: char,
    border: BorderStyle,
}

/// 2D drawing context for terminal buffers.
///
/// Modeled after HTML Canvas — mutable "current state" with a save/restore
/// stack. All coordinates are relative to `origin`; all draws are clipped
/// to the current clip rect.
pub struct RenderContext<'buf> {
    buffer: &'buf mut Buffer,
    state: ContextState,
    stack: Vec<ContextState>,
}

impl<'buf> RenderContext<'buf> {
    /// Create a new context spanning the full buffer.
    pub fn new(buffer: &'buf mut Buffer) -> Self {
        let clip = buffer.bounds(); // full buffer rect
        Self {
            buffer,
            state: ContextState {
                clip,
                origin: Point::ZERO,
                style: Style::default(),
                fill_char: ' ',
                border: BorderStyle::Single,
            },
            stack: Vec::new(),
        }
    }

    // ── State stack ──────────────────────────────────────────────

    /// Push the full current state (clip, origin, style, fill_char, border).
    pub fn save(&mut self) -> &mut Self {
        self.stack.push(self.state.clone());
        self
    }

    /// Pop and restore the most recently saved state.
    /// No-op if the stack is empty (defensive, avoids panics in widget code).
    pub fn restore(&mut self) -> &mut Self {
        if let Some(prev) = self.stack.pop() {
            self.state = prev;
        }
        self
    }

    /// Stack depth (number of saved states).
    pub fn save_depth(&self) -> usize {
        self.stack.len()
    }

    // ── Coordinate transform ─────────────────────────────────────

    /// Shift the origin by `offset`. Cumulative within a save/restore frame.
    pub fn translate(&mut self, offset: Point) -> &mut Self {
        self.state.origin = self.state.origin + offset;
        self
    }

    /// Current origin (local → buffer coords).
    pub fn origin(&self) -> Point {
        self.state.origin
    }

    // ── Clipping ─────────────────────────────────────────────────

    /// Intersect the current clip with `rect` (in local coords).
    /// Only narrows — you can never *expand* clip without restore.
    pub fn clip(&mut self, rect: Rect) -> &mut Self {
        let abs = rect.translate(self.state.origin);
        self.state.clip = self.state.clip.intersect(&abs);
        self
    }

    /// Current clip rect (in buffer coords).
    pub fn clip_rect(&self) -> Rect {
        self.state.clip
    }

    // ── Style setters ────────────────────────────────────────────

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.state.style = style;
        self
    }

    pub fn set_fg(&mut self, color: Color) -> &mut Self {
        self.state.style.foreground = color;
        self
    }

    pub fn set_bg(&mut self, color: Color) -> &mut Self {
        self.state.style.background = color;
        self
    }

    pub fn set_modifier(&mut self, modifier: Attribute) -> &mut Self {
        self.state.style.attributes.insert(modifier);
        self
    }

    /// Merge `style` on top of the current style (non-None fields overwrite).
    pub fn merge_style(&mut self, style: Style) -> &mut Self {
        // self.state.style = self.state.style.merge(style);
        self.state.style = style;
        self
    }

    pub fn set_fill_char(&mut self, ch: char) -> &mut Self {
        self.state.fill_char = ch;
        self
    }

    pub fn set_border(&mut self, border: BorderStyle) -> &mut Self {
        self.state.border = border;
        self
    }

    // ── Drawing primitives ───────────────────────────────────────

    /// Fill `rect` (local coords) with current style + fill_char.
    pub fn fill(&mut self, rect: Rect) -> &mut Self {
        self.fill_inner(rect, self.state.style, self.state.fill_char)
    }

    /// Fill with an explicit style, keeping current fill_char.
    pub fn fill_styled(&mut self, rect: Rect, style: Style) -> &mut Self {
        self.fill_inner(rect, style, self.state.fill_char)
    }

    /// Fill with a specific char, keeping current style.
    pub fn fill_char(&mut self, rect: Rect, ch: char) -> &mut Self {
        self.fill_inner(rect, self.state.style, ch)
    }

    /// Reset a region to default cells.
    pub fn clear(&mut self, rect: Rect) -> &mut Self {
        self.fill_inner(rect, Style::default(), ' ')
    }

    /// Stroke (draw border around) `rect` with current style + border set.
    pub fn stroke(&mut self, rect: Rect) -> &mut Self {
        self.stroke_inner(rect, self.state.style, self.state.border)
    }

    /// Stroke with explicit style + border set.
    pub fn stroke_styled(
        &mut self,
        rect: Rect,
        style: Style,
        border: BorderStyle,
    ) -> &mut Self {
        self.stroke_inner(rect, style, border)
    }

    /// Draw a single line of text at `pos` (local coords), current style.
    /// Truncates at clip boundary. Returns number of columns consumed.
    pub fn draw_text(&mut self, pos: Point, text: &str) -> usize {
        self.draw_text_inner(pos, text, self.state.style)
    }

    /// Draw text with an explicit style.
    pub fn draw_text_styled(
        &mut self,
        pos: Point,
        text: &str,
        style: Style,
    ) -> usize {
        self.draw_text_inner(pos, text, style)
    }

    /// Set a single cell at `pos` (local coords). Respects clip.
    pub fn set_cell(&mut self, pos: Point, cell: Cell) -> &mut Self {
        let abs = pos + self.state.origin;
        if self.state.clip.contains(&abs) {
            self.buffer[abs] = cell;
        }
        self
    }

    /// Read a cell at `pos` (local coords). Returns None if outside clip.
    pub fn get_cell(&self, pos: Point) -> Option<&Cell> {
        let abs = pos + self.state.origin;
        self.buffer.get(abs)
    }

    // ── Scoped helpers ───────────────────────────────────────────

    /// save → run closure → restore. Guarantees balanced stack.
    pub fn scoped(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        f(self);
        self.restore();
        self
    }

    /// save → translate + clip to `rect` → run closure → restore.
    /// The closure sees (0,0) as `rect.top_left` and is clipped to it.
    pub fn with_region(
        &mut self,
        rect: Rect,
        f: impl FnOnce(&mut Self),
    ) -> &mut Self {
        self.save();
        self.translate(rect.min);
        self.clip(Rect::from(rect.size()));
        f(self);
        self.restore();
        self
    }

    // ── Internals ────────────────────────────────────────────────

    fn resolve(&self, rect: Rect) -> Option<Rect> {
        let clipped = rect.translate(self.state.origin).intersect(&self.state.clip);
        if clipped.is_empty() { None } else { Some(clipped) }
    }

    fn fill_inner(&mut self, rect: Rect, style: Style, ch: char) -> &mut Self {
        if let Some(r) = self.resolve(rect) {
            for pos in &r {
                self.buffer[pos].set_char(ch);
                self.buffer[pos].style = style;
            }
        }
        self
    }

    fn stroke_inner(
        &mut self,
        rect: Rect,
        style: Style,
        border: BorderStyle,
    ) -> &mut Self {
        let abs = rect.translate(self.state.origin);
        // We clip each cell individually so partial borders work.
        let clip = self.state.clip;

        let x0 = abs.left();
        let x1 = abs.right() - 1;
        let y0 = abs.top();
        let y1 = abs.bottom() - 1;

        if abs.width() == 0 || abs.height() == 0 {
            return self;
        }

        let borders = border.to_border();

        let mut put = |x, y, str: &'static str| {
            let p = Point::new(y, x);
            if clip.contains(&p) {
                self.buffer[p].set_char('x');
                self.buffer[p].style = style;
            }
        };

        // corners
        put(x0, y0, borders.top_left);
        put(x1, y0, borders.top_right);
        put(x0, y1, borders.bottom_left);
        put(x1, y1, borders.bottom_right);

        // horizontal edges
        for x in (x0 + 1)..x1 {
            put(x, y0, borders.top);
            put(x, y1, borders.bottom);
        }

        // vertical edges
        for y in (y0 + 1)..y1 {
            put(x0, y, borders.left);
            put(x1, y, borders.right);
        }

        self
    }

    fn draw_text_inner(
        &mut self,
        pos: Point,
        text: &str,
        style: Style,
    ) -> usize {
        let mut col = 0usize;
        let abs_y = pos.y + self.state.origin.y;
        let abs_x_start = pos.x + self.state.origin.x;

        for ch in text.chars() {
            let w = ch.width().unwrap_or(1); // your grapheme/char width fn
            let abs_x = abs_x_start + col; // or your coord type

            // Stop if we've gone past clip right edge
            if abs_x + w > self.state.clip.right() {
                break;
            }

            let p = Point::new(abs_x, abs_y);
            if self.state.clip.contains(&p) {
                self.buffer[p].set_char(ch);
                self.buffer[p].style = style;
                // For wide chars, mark continuation cell(s)
                for i in 1..w {
                    let cont = Point::new(abs_x + i, abs_y);
                    if self.state.clip.contains(&cont) {
                        self.buffer[cont].set_continuation(style);
                    }
                }
            }

            col += w;
        }

        col
    }
}