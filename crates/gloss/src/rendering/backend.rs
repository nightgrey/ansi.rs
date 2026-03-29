use crate::{Border, Style};
use etwa::Maybe;
use geometry::{Bounded, Point, Rect, Size};

pub trait Backend {
    type Error;

    fn set_style(&mut self, style: Style);
    fn set_border(&mut self, border: Border);
    fn set_fill(&mut self, fill: char);

    /// Clip to a [`Shape`].
    ///
    /// All subsequent drawing operations up to the next [`restore`]
    /// are clipped by the bounds.
    ///
    /// [`restore`]: Backend::restore
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
}
