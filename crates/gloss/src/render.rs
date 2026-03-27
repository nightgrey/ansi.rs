use crate::{Document, FontStyle, FontWeight, NodeId, NodeKind, Style};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use etwa::Maybe;
use geometry::{Bounded, Intersect, Point, Rect, Size};

pub trait Backend {
    type Error;

    /// Stroke a [`Shape`], using the default [`Style`].
    fn stroke(&mut self, bounds: Rect, style: Style);

    fn clear(&mut self, bounds: Rect) {
        self.fill(bounds, Style::Default, ' ');
    }

    /// Fill an area with a style.
    fn fill(&mut self, bounds: Rect, style: Style, char: char);

    /// Fill an area with a single character.
    fn fill_char(&mut self, bounds: Rect, char: char);

    /// Fill an area with a single character and style.
    fn fill_style(&mut self, bounds: Rect, style: Style);

    /// Draw a text.
    ///
    /// The `pos` parameter specifies the upper-left corner of the text
    fn draw_text(&mut self, position: Point, text: &str, style: Style);

    /// Clip to a [`Shape`].
    ///
    /// All subsequent drawing operations up to the next [`restore`]
    /// are clipped by the bounds.
    ///
    /// [`restore`]: Backend::restore
    fn clip(&mut self, bounds: Rect) -> Result<(), Self::Error>;

    /// Translate the origin.
    fn translate(&mut self, offset: Point) -> Result<(), Self::Error>;

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
    fn with_state(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error> {
        self.save()?;
        f(self).and(self.restore())
    }

    /// Do graphics operations within a clipped region.
    ///
    /// The closure sees (0,0) as `rect.top_left` and is clipped to it.
    fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self) -> Result<(), Self::Error>) -> Result<(), Self::Error> {
        self.save()?;
        self.translate(rect.min)?;
        self.clip(Rect::from(rect.size()))?;
        let result = f(self);
        self.restore()?;
        result
    }

    /// Resize the canvas, if necessary.
    fn resize(&mut self, size: Size) -> Result<(), Self::Error>;

    /// Finish any pending operations.
    fn finish(&mut self) -> Result<(), Self::Error>;
}

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Renderer<B: Backend>(pub B);

impl<B: Backend> Renderer<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }

    pub fn render(&mut self, doc: &Document<'_>) -> Result<(), B::Error> {
        self.resize(doc.size(doc.root))?;
        self.render_node(doc, doc.root, crate::Style::Default)?;
        self.finish()
    }

    fn render_node(
        &mut self,
        doc: &Document<'_>,
        id: NodeId,
        inherited: Style,
    ) -> Result<(), B::Error> {
        let mut node = doc.node(id);
        let bounds = doc.bounds(id);
        let content_bounds = doc.content_bounds(id);
        let style = node.style.inherit_from(inherited);


        if style.background_color().is_some() { self.fill_style(content_bounds.clone(), style) }
        if style.border().is_some() { self.stroke(bounds, style) }

            // Leaf text.
            match &node.kind {
                NodeKind::Span(text) => {
                        // MVP: left-aligned only in actual paint pass.
                        // Center/right can be added once line layout is shared with measurement.
                        self.draw_text(content_bounds.min, text, style);
                }
                _ => {}
            }

        for child in doc.children(id) {
            self.render_node(doc, child, style)?;
        }
        
        self.finish()?;

        Ok(())
    }
}
