use std::io;
use unicode_segmentation::UnicodeSegmentation;
use geometry::{Bounded, Contains, ContextualResolve, Intersect, Outer, Point, Ranges, Rect, Sides, Size, Translate};
use sigil::{Buffer, Cell, GraphemeArena};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::{Style, Border, Backend, Renderer};
use crate::symbols::Symbol;

/// Snapshot of all context state, pushed/popped via save/restore.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextState {
    clip: Rect,
    origin: Point,
    style: ansi::Style,
    border: Border,
    fill: char,
}

/// 2D drawing context for terminal buffers.
///
/// Modeled after HTML Canvas — mutable "current state" with a save/restore
/// stack. All coordinates are relative to `origin`; all draws are clipped
/// to the current clip rect.
pub struct BufferRenderer<'a> {
    pub(crate) buffer: &'a mut Buffer,
    arena: &'a mut GraphemeArena,
    state: ContextState,
    stacks: Vec<ContextState>,
}

impl<'buf> BufferRenderer<'buf> {
    /// Create a new context spanning the full buffer.
    pub fn new(buffer: &'buf mut Buffer, arena: &'buf mut GraphemeArena) -> Renderer<Self> {
        let clip = buffer.bounds(); // full buffer rect
        Renderer(Self {
            buffer,
            arena,
            state: ContextState {
                clip,
                origin: Point::ZERO,
                style: ansi::Style::None,
                border: Border::None,
                fill: ' ',
            },
            stacks: Vec::new(),
        })
    }

    /// Current clip
    pub fn state(&self) -> &ContextState {
        &self.state
    }

    /// Intersect the current clip with `rect` (in local coords).
    /// Only narrows — you can never *expand* clip without restore.
    pub fn clip(&mut self, rect: Rect) -> &mut Self {
        let abs = rect.translate(&self.state.origin);
        self.state.clip = self.state.clip.intersect(&abs);
        self
    }

    /// Push the full current state (clip, origin, style, fill_char, border).
    pub fn save(&mut self) -> &mut Self {
        self.stacks.push(self.state.clone());
        self
    }

    /// Pop and restore the most recently saved state.
    /// No-op if the stack is empty (defensive, avoids panics in widget code).
    pub fn restore(&mut self) -> &mut Self {
        if let Some(prev) = self.stacks.pop() {
            self.state = prev;
        }
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.state = ContextState::default();
        self
    }


    /// Shift the origin by `offset`. Cumulative within a save/restore frame.
    pub fn translate(&mut self, offset: Point) -> &mut Self {
        self.state.origin = self.state.origin + offset;
        self
    }

    pub fn fill(&mut self, rect: Rect, style: ansi::Style, fill: char) -> &mut Self {
        self.fill_impl(rect, style, fill)
    }

    pub fn fill_char(&mut self, rect: Rect, fill: char) -> &mut Self {
        self.fill_impl(rect, self.state.style, fill)
    }

    pub fn stroke(&mut self, rect: Rect, style: ansi::Style, border: Border) -> &mut Self {
        self.stroke_impl(rect, style, border)
    }

    pub fn draw_text(&mut self, text: &str, position: Point, style: ansi::Style) -> usize {
        self.draw_text_impl(text, position, style)
    }

    /// Reset a region to default cells.
    pub fn clear(&mut self, rect: Rect) -> &mut Self {
        self.fill_impl(rect, ansi::Style::default(), ' ')
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
    pub fn with(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        f(self);
        self.restore();
        self
    }

    /// save → translate + clip to `rect` → run closure → restore.
    /// The closure sees (0,0) as `rect.top_left` and is clipped to it.
    pub fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        self.translate(rect.min);
        self.clip(Rect::from(rect.size()));
        f(self);
        self.restore();
        self
    }

    fn clipped(&self, rect: Rect) -> Option<Rect> {
        let clipped = rect
            .translate(&self.state.origin)
            .intersect(&self.state.clip);

        if clipped.is_empty() {
            None
        } else {
            Some(clipped)
        }
    }

    fn fill_impl(&mut self, rect: Rect, style: ansi::Style, ch: char) -> &mut Self {
        if let Some(r) = self.clipped(rect) {
            for pos in &r {
                let index: usize = self.buffer.bounds().resolve(pos);
                self.buffer[index].set_char(ch, self.arena).set_style(style);
            }

        }
        self
    }

    fn stroke_impl(&mut self, rect: Rect, style: ansi::Style, border: Border) -> &mut Self {
        let  mut bounds = rect.clone().translate(&self.state.origin);
        let border = border.into_symbols();

        bounds.max.x -= border.right.width();
        bounds.max.y -= border.bottom.width();

        if bounds.is_empty() {
            return self;
        }
        // We clip each cell individually so partial borders work.
        let clip = self.state.clip;

        let mut set = |point: Point, border: Symbol| {
            if clip.contains(&point) {
                self.buffer[point].set_char_and_width('x', border.width() as u8, self.arena);
            }
        };

        // corners
        set(bounds.top_left(), border.top_left);
        set(bounds.top_right(), border.top_right);
        set(bounds.bottom_left(), border.bottom_left);
        set(bounds.bottom_right(), border.bottom_right);


        // horizontal edges
        for x in (bounds.left() + 1)..bounds.right() {
            set(Point { x, y: bounds.top() }, border.top);
            set(Point { x, y: bounds.bottom() }, border.bottom);
        }

        // vertical edges
        for y in (bounds.top() + 1)..bounds.bottom() {
            set(Point { x: bounds.left(), y }, border.left);
            set(Point { x: bounds.right(), y }, border.right);
        }

        self
    }

    fn draw_text_impl(&mut self, text: &str, pos: Point, style: ansi::Style) -> usize {
        let mut col = 0usize;
        let abs_y = pos.y + self.state.origin.y;
        let abs_x_start = pos.x + self.state.origin.x;

        for (grapheme, width) in text.graphemes(true)
            .map(|g| (g, g.width())) {
            let abs_x = abs_x_start + col; // or your coord type

            // Stop if we've gone past clip right edge
            if abs_x + width > self.state.clip.right() {
                break;
            }

            let p = Point::new(abs_x, abs_y);
            if self.state.clip.contains(&p) {
                self.buffer[p].set_str_and_width(grapheme, width as u8, self.arena);
                // For wide chars, mark continuation cell(s)
                for i in 1..width {
                    let cont = Point::new(abs_x + i, abs_y);
                    if self.state.clip.contains(&cont) {
                        self.buffer[cont].set_continuation(self.arena).set_style(style);
                    }
                }
            }

            col += width;
        }

        col
    }
}

impl<'a> Backend for BufferRenderer<'a> {
    type Error = io::Error;

    fn stroke(&mut self, bounds: Rect, style: Style) {
        BufferRenderer::stroke(self, bounds, style.into(), style.get_border());
    }

    fn fill(&mut self, bounds: Rect, style: Style, char: char) {
        BufferRenderer::fill(self, bounds, style.into(), char);
    }

    fn fill_char(&mut self, bounds: Rect, char: char) {
        self.fill(bounds, self.state.style, char);
    }

    fn fill_style(&mut self, bounds: Rect, style: Style) {
        if let Some(r) = self.clipped(bounds) {
            for pos in &r {
                let index: usize = self.buffer.bounds().resolve(pos);
                self.buffer[index].set_char(self.state.fill, self.arena).set_style(style.into());
            }
        }
    }

    fn draw_text(&mut self, position: Point, text: &str, style: Style) {
        BufferRenderer::draw_text(self, text, position, style.into());
    }

    fn clip(&mut self, bounds: Rect) -> Result<(), Self::Error> {
        BufferRenderer::clip(self, bounds);
        Ok(())
    }

    fn translate(&mut self, offset: Point) -> Result<(), Self::Error> {
        BufferRenderer::translate(self, offset);
        Ok(())
    }

    fn save(&mut self) -> Result<(), Self::Error> {
        BufferRenderer::save(self);
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        BufferRenderer::restore(self);
        Ok(())
    }

    fn resize(&mut self, size: Size) -> Result<(), Self::Error> {
        self.buffer.resize(size.width, size.height);
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
