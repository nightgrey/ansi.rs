use crate::{Style};
use etwa::Maybe;
use geometry::{Bounded, Point, Rect, Size};

pub trait Backend {
    type Error;

    /// Stroke a [`Shape`], using the default [`Style`].
    fn stroke(&mut self, bounds: Rect, style: Style);

    fn clear(&mut self, bounds: Rect) {
        self.fill(bounds, Style::DEFAULT, ' ');
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
    fn with(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error> {
        self.save()?;
        f(self).and(self.restore())
    }

    /// Resize the canvas, if necessary.
    fn resize(&mut self, size: Size) -> Result<(), Self::Error>;

    /// Finish any pending operations.
    fn finish(&mut self) -> Result<(), Self::Error>;
}
