use crate::{Border, Document, NodeId, NodeKind, Style};
use etwa::Maybe;
use geometry::{Bounded, Point, Rect, Size};
use derive_more::{AsMut, AsRef, Deref, DerefMut};

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Renderer<B: RendererBackend>(pub B);

impl<B: RendererBackend> Renderer<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }

    pub fn render(&mut self, doc: &Document<'_>) -> Result<(), B::Error> {
        self.resize(doc.size(doc.root))?;

        self.render_node(doc, doc.root, Style::DEFAULT)?;
        self.finish()
    }

    fn render_node(
        &mut self,
        doc: &Document<'_>,
        id: NodeId,
        inherited: Style,
    ) -> Result<(), B::Error> {
        let  node = doc.node(id);
        let bounds = doc.bounds(id);
        let content_bounds = doc.content_bounds(id);
        let style = node.style.inherit(inherited);

        // Snapshot the current state
        self.save()?;

        // Apply this node's style and clip
        self.translate(bounds.min)?;
        self.clip(bounds.size().into())?;
        self.set_style(style);

        // Paint
        if style.has_background() { self.fill(None, None, Some(' ')) }
        if style.has_border() { self.stroke(None, None) }

        match &node.kind {
            NodeKind::Span(text) => {
                self.draw_text(text, Some(content_bounds.min - bounds.min), None);
            }
            NodeKind::Div => {
            }
        }

        // Recurse — clip to *content* area so children don't paint over padding/borders
        self.save()?;
        self.translate(content_bounds.min)?;
        self.clip(content_bounds.size().into())?;

        for child in doc.children(id) {
            self.render_node(doc, child, style)?;
        }

        self.restore()?;

        self.restore()
    }
}

pub trait RendererBackend {
    type Error;

    fn set_style(&mut self, style: Style);
    fn set_border(&mut self, border: Border);
    fn set_fill(&mut self, fill: char);

    /// Clip to a [`Shape`].
    ///
    /// All subsequent drawing operations up to the next [`restore`]
    /// are clipped by the bounds.
    ///
    /// [`restore`]: RendererBackend::restore
    fn clip(&mut self, bounds: Rect) -> Result<(), Self::Error>;

    /// Stroke a [`Shape`], using the default [`Style`].
    fn stroke(&mut self, bounds: Option<Rect>, style: Option<Style>);

    /// Fill an area with a style.
    fn fill(&mut self, bounds: Option<Rect>, style: Option<Style>, char: Option<char>);
    /// Draw a text.
    ///
    /// The `pos` parameter specifies the upper-left corner of the text
    fn draw_text(&mut self,text: &str,  position: Option<Point>,  style: Option<Style>);

    fn clear(&mut self, bounds: Option<Rect>) {
        self.fill(bounds, None, None);
    }

    /// Get the current clip bounds.
    fn current_clip(&self) -> Rect;


    /// Translate the origin.
    fn translate(&mut self, offset: Point) -> Result<(), Self::Error>;

    /// Save the context state.
    ///
    /// Push a new context state onto the stack. See [`pop`] for details.
    ///
    /// [`pop`]: RendererBackend::restore
    fn save(&mut self) -> Result<(), Self::Error>;

    /// Restore the context state.
    ///
    /// Pop a context state that was pushed by [`save`]. See
    /// that method for more details.
    ///
    /// [`save`]: RendererBackend::save
    fn restore(&mut self) -> Result<(), Self::Error>;

    /// Do graphics operations with the context state saved and then restored.
    ///
    /// Equivalent to [`save`], calling `f`, then
    /// [`restore`]. See those methods for more details.
    ///
    /// [`restore`]: RendererBackend::restore
    /// [`save`]: RendererBackend::save
    fn with(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error> {
        self.save()?;
        f(self).and(self.restore())
    }

    fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self) -> Result<(), Self::Error>) -> Result<(), Self::Error> {
        self.save()?;
        self.translate(rect.min)?;
        self.clip(Rect::from(rect.size()))?;
        f(self).and(self.restore())
    }

    /// Resize the canvas, if necessary.
    fn resize(&mut self, size: Size) -> Result<(), Self::Error>;

    /// Finish any pending operations.
    fn finish(&mut self) -> Result<(), Self::Error>;
}
