use crate::{
    Arena, Buffer, BufferDrawingContext, Document, DoubleBuffer, DrawingContext, ElementId,
    ElementKind, Layout, Rasterer, document,
};
use bilge::prelude::u1;
use derive_more::{Deref, DerefMut};
use geometry::{Bound, Row};
use geometry::{Rect, Size};
use maybe::Maybe;
use std::io;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Engine<'a> {
    space: Size,
    #[deref]
    #[deref_mut]
    pub document: Document<'a>,
    pub buffer: DoubleBuffer,
    pub arena: Arena,
    rasterer: Rasterer,
}

impl<'a> Engine<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            space: Size::new(width as u16, height as u16),
            document: Document::new(),
            buffer: DoubleBuffer::new(width, height),
            arena: Arena::new(),
            rasterer: Rasterer::inline(width, height),
        }
    }

    pub fn back_buffer(&self) -> &Buffer {
        self.buffer.back()
    }

    pub fn front_buffer(&self) -> &Buffer {
        self.buffer.front()
    }

    pub fn size(&self) -> Size {
        self.space
    }

    pub fn layout(&mut self) {
        self.document.compute_layout(self.space);
    }

    pub fn paint(&mut self) {
        let buffer = &mut self.buffer.back;
        let arena = &mut self.arena;
        let document = &self.document;

        buffer.clear();
        let mut ctx = BufferDrawingContext::new(buffer, arena);
        paint_node(&mut ctx, document, document.root_id, Layout::DEFAULT);
        ctx.finish();
    }

    pub fn paint_with<F>(&mut self, f: F)
    where
        F: FnOnce(&mut BufferDrawingContext<'_>),
    {
        let buffer = &mut self.buffer.back;
        let arena = &mut self.arena;
        let document = &self.document;

        buffer.clear();
        let mut ctx = BufferDrawingContext::new(buffer, arena);

        f(&mut ctx);
        ctx.finish();
    }

    pub fn layout_and_paint(&mut self) {
        self.layout();
        self.paint();
    }

    pub fn present(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        let back = &mut self.buffer.back;
        let front = &mut self.buffer.front;
        let arena = &mut self.arena;

        self.rasterer.present(front, back, arena)?;
        self.rasterer.flush(w)
    }

    pub fn render(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        self.layout();
        self.paint();
        self.present(w)?;

        self.buffer.swap();

        Ok(())
    }

    pub fn invalidate(&mut self) {
        self.rasterer.invalidate();
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if self.space.width != width as u16 || self.space.height != height as u16 {
            self.space.width = width as u16;
            self.space.height = height as u16;

            self.buffer.resize(width, height);
            self.rasterer.resize(width, height);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.rasterer.clear();
        self.document.clear();
        self.buffer.clear();
    }
}

fn paint_node<B: DrawingContext + ?Sized>(
    ctx: &mut B,
    document: &Document<'_>,
    id: ElementId,
    parent_layout: Layout,
) {
    let node = document.element(id);
    let border_bounds = document.border_bounds(id);
    let content_bounds = document.content_bounds(id);
    let style = node.layout.inherit(parent_layout);

    ctx.save();

    ctx.translate(border_bounds.min)
        .clip(border_bounds.size())
        .style(style)
        .border_style(node.border);

    // Everything below is in node-local coordinates (relative to border-box origin).
    let local_bounds = Rect::from(border_bounds.size());

    // Children's taffy locations are border-box relative, so clip/bg use
    // content bounds normalized into the node's own origin.
    let normalized_bounds = content_bounds - border_bounds.min;

    // Background fills the border-box (CSS `background-clip: border-box`)
    // so padding participates in the backdrop. Only a real color paints —
    // `Color::None` means "no fill", leaving the parent's backdrop visible.
    if let Some(bg) = style.background
        && bg != ansi::Color::None
    {
        ctx.rect(local_bounds);
    }

    // Border is drawn over the background so the corners / edges overwrite it.
    if style.border.is_some() {
        ctx.border(local_bounds);
    }

    match &node.kind {
        ElementKind::Span(text) => {
            ctx.text(normalized_bounds.min, text);
        }
        ElementKind::Div => {}
    }

    ctx.save();
    ctx.clip(normalized_bounds);

    for child in document.children(id) {
        paint_node(ctx, document, child, style);
    }

    ctx.restore();
    ctx.restore();
}
