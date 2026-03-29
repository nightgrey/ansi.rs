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

    /// Current state
    pub fn state(&self) -> &ContextState {
        &self.state
    }

    pub fn set_style(&mut self, style: ansi::Style) -> &mut Self {
        self.state.style = style;
        self
    }

    pub fn set_border(&mut self, border: Border) -> &mut Self {
        self.state.border = border;
        self
    }

    pub fn set_fill(&mut self, fill: char) -> &mut Self {
        self.state.fill = fill;
        self
    }

    /// Intersect the current clip with `rect` (in local coords).
    /// Only narrows — you can never *expand* clip without restore.
    pub fn clip(&mut self, rect: Rect) -> &mut Self {
        self.state.clip = self.state.clip.intersect(&self.local(rect));
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

    pub fn fill(&mut self, rect: Option<Rect>, style: Option<ansi::Style>, fill: Option<char>) -> &mut Self {
        self.fill_impl(rect.map(|r| self.local(r)).unwrap_or(self.state.clip), style.map(|s| s.into()).unwrap_or(self.state.style), fill.unwrap_or(self.state.fill))
    }

    pub fn stroke(&mut self, rect: Option<Rect>, style: Option<ansi::Style>, border: Option<Border>) -> &mut Self {
        self.stroke_impl(rect.map(|r| self.local(r)).unwrap_or(self.state.clip), style.map(|s| s.into()).unwrap_or(self.state.style), border.map(|b| b.into()).unwrap_or(self.state.border))
    }

    pub fn draw_text(&mut self, text: &str, position: Option<Point>, style: Option<ansi::Style>) -> usize {
        self.draw_text_impl(text, position.map(|p| self.local(p)).unwrap_or(Point::ZERO), style.map(|s| s.into()).unwrap_or(self.state.style))
    }

    pub fn clear(&mut self, rect: Option<Rect>) -> &mut Self {
        self.fill_impl(rect.map(|r| self.local(r)).unwrap_or(self.state.clip), self.state.style, self.state.fill)
    }

    /// Set a single cell at `pos` (local coords). Respects clip.
    pub fn set(&mut self, pos: Point, cell: Cell) -> &mut Self {
        let abs = pos + self.state.origin;
        if self.state.clip.contains(&abs) {
            self.buffer[abs] = cell;
        }
        self
    }

    /// Read a cell at `pos` (local coords). Returns None if outside clip.
    pub fn get(&self, pos: Point) -> Option<&Cell> {
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

    fn fill_impl(&mut self, rect: Rect, style: ansi::Style, ch: char) -> &mut Self {
        if let Some(r) = self.intersect(rect) {
            for pos in &r {
                let index: usize = self.buffer.bounds().resolve(pos);
                self.buffer[index].set_char(ch, self.arena).set_style(style);
            }

        }
        self
    }

    fn stroke_impl(&mut self, rect: Rect, style: ansi::Style, border: Border) -> &mut Self {
        let mut rect = rect;
        let border = border.into_symbols();

        rect.max.x -= border.right.width();
        rect.max.y -= border.bottom.width();

        if rect.is_empty() {
            return self;
        }

        // We clip each cell individually so partial borders work.
        let mut set = |x: usize, y: usize, border: Symbol| {
            if self.state.clip.contains(&(x, y)) {
                self.buffer[(x, y)].set_char_and_width('x', border.width() as u8, self.arena);
            }
        };

        // corners
        set(rect.left(), rect.top(), border.top_left);
        set(rect.right(), rect.top(), border.top_right);
        set(rect.left(), rect.bottom(), border.bottom_left);
        set(rect.right(), rect.bottom(), border.bottom_right);

        // horizontal edges
        for x in (rect.left() + border.left.width())..rect.right() {
            set(x, rect.top(), border.top);
            set(x, rect.bottom(), border.bottom);
        }

        // vertical edges
        for y in (rect.top() + border.top.width())..rect.bottom() {
            set(rect.left(), y, border.left);
            set(rect.right(), y, border.right);
        }

        self
    }

    fn draw_text_impl(&mut self, text: &str, pos: Point, style: ansi::Style) -> usize {
        let pos = pos;

        let y = pos.y;
        let mut i = 0;

        for (grapheme, width) in text.graphemes(true)
            .map(|g| (g, g.width())) {
            let x = pos.x + i; // or your coord type

            // Stop if we've gone past clip right edge
            if x + width > self.state.clip.right() {
                break;
            }

            if self.state.clip.contains(&(x, y)) {
                self.buffer[(x, y)].set_str_and_width(grapheme, width as u8, self.arena);
                // For wide chars, mark continuation cell(s)
                for i in 1..width {
                    let cont = (x + i, y);
                    if self.state.clip.contains(&cont) {
                        self.buffer[cont].set_continuation(self.arena).set_style(style);
                    }
                }
            }

            i += width;
        }

        i
    }

    fn intersect(&self, rect: Rect) -> Option<Rect> {
        let result = self.state.clip
            .intersect(&rect);

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn local<T: Translate<Point>>(&self, rect: T) -> T::Output {
        rect.translate(&self.state.origin)
    }

}

impl<'a> Backend for BufferRenderer<'a> {
    type Error = io::Error;

    fn set_style(&mut self, style: Style) {
        self.state.style = style.into();
    }

    fn set_border(&mut self, border: Border) {
        self.state.border = border;
    }

    fn set_fill(&mut self, fill: char) {
        self.state.fill = fill;
    }

    fn clip(&mut self, bounds: Rect) -> Result<(), Self::Error> {
        BufferRenderer::clip(self, bounds);
        Ok(())
    }

    fn stroke(&mut self, bounds: Option<Rect>, style: Option<Style>) {
        BufferRenderer::stroke(self, bounds, style.map(|s| s.into()), style.map_or(Some(Border::Solid), |s| Some(s.get_border())));
    }

    fn fill(&mut self, bounds: Option<Rect>, style: Option<Style>, char: Option<char>) {
        BufferRenderer::fill(self, bounds, style.map(|s| s.into()), char);
    }

    fn draw_text(&mut self, text: &str, position: Option<Point>, style: Option<Style>) {
        BufferRenderer::draw_text(self, text, position, style.map(|s| s.into()));
    }

    fn current_clip(&self) -> Rect {
        self.state.clip.translate(&self.state.origin)
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
