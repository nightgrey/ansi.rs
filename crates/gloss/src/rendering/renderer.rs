use crate::{Border, Document, NodeId, NodeKind, Style};
use maybe::Maybe;
use geometry::{Bounded, Point, Rect, Size};
use derive_more::{AsMut, AsRef, Deref, DerefMut};

pub trait Backend {
    type Error;

    fn fill_style(&mut self, style: Style);
    fn fill_char(&mut self, fill: char);

    fn stroke_type(&mut self, border: Border);

    /// Clip to a [`Rect`].
    ///
    /// All subsequent drawing operations up to the next [`restore`]
    /// are clipped by the bounds.
    ///
    /// [`restore`]: Backend::restore
    fn clip(&mut self, bounds: Rect) -> Result<(), Self::Error>;

    /// Translate the origin.
    fn translate(&mut self, offset: Point) -> Result<(), Self::Error>;

    /// Fill an area with a style.
    fn fill(&mut self, bounds: impl Into<Option<Rect>>, fill_style: impl Into<Option<Style>>, fill_char: impl Into<Option<char>>);

    /// Stroke a [`Shape`], using the default [`Style`].
    fn stroke(&mut self, bounds: impl Into<Option<Rect>>, stroke_type: impl Into<Option<Border>>);

    /// Draw a text.
    ///
    /// The `pos` parameter specifies the upper-left corner of the text
    fn text(&mut self, position: impl Into<Option<Point>>, fill_style: impl Into<Option<Style>>, str: impl AsRef<str>);

    fn clear(&mut self, bounds: Option<Rect>) {
        self.fill(bounds, None, None);
    }

    /// Save the context state.
    ///
    /// Push a new context state onto the stack. See [`pop`] for details.
    ///
    /// [`pop`]: Backend::restore
    fn save(&mut self) -> Result<(), Self::Error>;

    /// Restore the context state.
    ///
    /// Pop a context state that was pushed by [`save`]. See
    /// that method for more details.
    ///
    /// [`save`]: Backend::save
    fn restore(&mut self) -> Result<(), Self::Error>;

    /// Do graphics operations with the context state saved and then restored.
    ///
    /// Equivalent to [`save`], calling `f`, then
    /// [`restore`]. See those methods for more details.
    ///
    /// [`restore`]: Backend::restore
    /// [`save`]: Backend::save
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

    fn into_renderer(self) -> Renderer<Self> where Self: Sized {
        Renderer(self)
    }
}

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Renderer<B: Backend>(pub B);

impl<B: Backend> Renderer<B> {
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
        let style = node.style.apply(inherited);

        // Snapshot the current state
        self.save()?;

        // Apply this node's style and clip
        self.translate(bounds.min)?;
        self.clip(bounds.size().into())?;
        self.fill_style(style);

        // Normalize content bounds to be relative to the node's border box.
        let content_bounds = content_bounds - bounds.min;

        // Paint
        if style.has_background() { self.fill(content_bounds, None, None) }
        if style.has_border() { self.stroke(None, node.get_border()) }

        match &node.kind {
            NodeKind::Span(text) => {
                self.text(Some(content_bounds.min), None, text);
            }
            NodeKind::Div => {
            }
        }

        // Recurse — clip to *content* area so children don't paint over padding/borders.
        // Don't translate — child bounds from taffy are relative to the parent's
        // border box, so translating to the content area would double-count padding.
        self.save()?;
        self.clip(content_bounds)?;

        for child in doc.children(id) {
            self.render_node(doc, child, style)?;
        }

        self.restore()?;

        self.restore()
    }
}
