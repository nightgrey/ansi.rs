use crate::{Arena, Buffer, BufferPainter, Document, DoubleBuffer, DrawingContext, Presenter};
use derive_more::{Deref, DerefMut};
use geometry::Size;
use std::io;

#[derive(Debug, Deref, DerefMut)]
pub struct Engine<'a> {
    space: Size,
    #[deref]
    #[deref_mut]
    document: Document<'a>,
    buffer: DoubleBuffer,
    arena: Arena,
    presenter: Presenter<io::Stdout>,
}

impl<'a> Engine<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            space: Size::new(width as u16, height as u16),
            document: Document::new(),
            buffer: DoubleBuffer::new(width, height),
            arena: Arena::new(),
            presenter: Presenter::inline(io::stdout()),
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

    pub fn paint(&mut self) -> io::Result<()> {
        let buffer = &mut self.buffer.back;
        let arena = &mut self.arena;

        buffer.clear();
        self.document.paint(&mut BufferPainter::new(buffer, arena))
    }

    pub fn paint_with<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut BufferPainter<'_>),
    {
        let buffer = &mut self.buffer.back;
        let arena = &mut self.arena;
        let _document = &self.document;

        buffer.clear();
        let mut ctx = BufferPainter::new(buffer, arena);
        f(&mut ctx);
        ctx.finish()?;
        Ok(())
    }

    pub fn layout_and_paint(&mut self) -> io::Result<()> {
        self.layout();
        self.paint()
    }

    pub fn present(&mut self, _w: &mut impl io::Write) -> io::Result<()> {
        let back = &mut self.buffer.back;
        let front = &mut self.buffer.front;
        let arena = &mut self.arena;

        self.presenter.present(front, back, arena)?;
        self.presenter.flush()?;
        Ok(())
    }

    pub fn render(&mut self, w: &mut impl io::Write) -> io::Result<()> {
        self.layout();
        self.paint()?;
        self.present(w)?;

        self.buffer.swap();

        Ok(())
    }

    pub fn invalidate(&mut self) {
        self.presenter.invalidate();
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if self.space.width != width as u16 || self.space.height != height as u16 {
            self.space.width = width as u16;
            self.space.height = height as u16;

            self.buffer.resize(width, height);
            self.presenter.resize(width, height);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.presenter.clear();
        self.document.clear();
        self.buffer.clear();
    }
}
