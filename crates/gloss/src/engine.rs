use std::io;
use derive_more::{Deref, DerefMut};
use crate::{Arena, BufferDrawingContext, Document, DoubleBuffer, DrawingContext, Painter, Rasterer, Space};

#[derive(Debug, Deref, DerefMut)]
pub struct Engine<'a> {
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
            document: Document::new(),
            buffer: DoubleBuffer::new(width, height),
            arena: Arena::new(),
            rasterer: Rasterer::inline(width, height),
        }
    }

    pub fn render(&mut self, mut w: impl io::Write) -> io::Result<()> {
        self.document.compute_layout(self.buffer.size());

        self.buffer.back_mut().clear();
        BufferDrawingContext::new(self.buffer.back_mut(), &mut self.arena)
            .painter()
            .paint(&self.document);

        self.rasterer.present(self.buffer.front(), self.buffer.back(), &self.arena)?;
        self.rasterer.flush(&mut w)?;
        self.buffer.swap();

        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.buffer.resize(width, height);
        self.rasterer.resize(width, height);
    }
}
