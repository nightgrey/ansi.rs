use std::io;
use derive_more::{Deref, DerefMut};
use geometry::Bounded;
use crate::{Arena, Buffer, BufferDrawingContext, Document, DrawingContext, Painter, Rasterer, Space};

#[derive(Debug, Deref, DerefMut)]
pub struct Engine<'a> {
    #[deref]
    #[deref_mut]
    document: Document<'a>,
    buffer: Buffer,
    arena: Arena,
    rasterer: Rasterer,
}

impl<'a> Engine<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            document: Document::new(),
            buffer: Buffer::new(width, height),
            arena: Arena::new(),
            rasterer: Rasterer::inline(width, height),
        }
    }

    pub fn render(&mut self, mut w: impl io::Write) -> io::Result<()> {
        self.document.compute_layout(self.buffer.size());

        // TODO: Save painter, or re-make it for each render
        BufferDrawingContext::new(&mut self.buffer, &mut self.arena).painter().paint(&self.document);
        self.rasterer.raster(&self.buffer, &self.arena)?;
        self.rasterer.flush(&mut w)?;
        
        Ok(())
    }
    
    pub fn resize(&mut self, width: usize, height: usize) {
        self.buffer.resize(width, height);
        self.rasterer.resize(width, height);
    }
}
