use crate::{
    Arena, Buffer, BufferDrawingContext, Document, DoubleBuffer, DrawingContext, Rasterer, document,
};
use bilge::prelude::u1;
use derive_more::{Deref, DerefMut};
use geometry::Size;
use geometry::{Bound, Row};
use std::io;

#[derive(Debug, Deref, DerefMut)]
pub struct Engine<'a> {
    size: Size,
    #[deref]
    #[deref_mut]
    document: Document<'a>,
    buffer: DoubleBuffer,
    arena: Arena,
    rasterer: Rasterer,
}

impl<'a> Engine<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: Size::new(width as u16, height as u16),
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
        self.size
    }

    pub fn layout(&mut self) {
        self.document.compute_layout(self.size);
    }

    pub fn paint(&mut self) {
        let buffer = &mut self.buffer.back;
        let arena = &mut self.arena;
        let document = &self.document;

        buffer.clear();
        BufferDrawingContext::new(buffer, arena).paint(document);
    }

    pub fn present(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        let back = &mut self.buffer.back;
        let front = &mut self.buffer.front;
        let arena = &mut self.arena;

        dbg!(back.size());
        dbg!(front.size());
        self.rasterer.present(back, front, arena)?;
        self.rasterer.flush(w)
    }

    pub fn render(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        self.layout();
        self.paint();
        self.present(w);

        self.buffer.swap();

        Ok(())
    }

    pub fn invalidate(&mut self) {
        self.rasterer.invalidate();
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if self.size.width != width as u16 || self.size.height != height as u16 {
            self.size.width = width as u16;
            self.size.height = height as u16;

            self.buffer.resize(width, height);
            self.rasterer.resize(width, height);
        }
    }
}
