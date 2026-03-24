use derive_more::{AsMut, AsRef, Deref, DerefMut};
use ansi::{Attribute, Color};
use geometry::{Bounded, Point, Rect, Size};
use crate::{AlignSelf, Style, Border, Document, FontStyle, FontWeight, NodeId, NodeKind};
pub trait RenderContext {
    type Error;

    /// Stroke a [`Shape`], using the default [`Style`].
    fn stroke(&mut self, bounds: Rect, style: Style);

    fn clear(&mut self, bounds: Rect) {
        self.fill(bounds, ' ', Style::Default);
    }

    /// Fill an area with a style.
    fn fill(&mut self, bounds: Rect, char: char, style: Style);

    /// Fill an area with a single character.
    fn fill_char(&mut self, bounds: Rect, char: char);

    /// Fill an area with a single character and style.
    fn fill_style(&mut self, bounds: Rect, style: Style);

    /// Clip to a [`Shape`].
    ///
    /// All subsequent drawing operations up to the next [`restore`]
    /// are clipped by the bounds.
    ///
    /// [`restore`]: RenderContext::restore
    fn push(&mut self, bounds: Rect);

    /// Restore the context state.
    ///
    /// Pop a context state that was pushed by [`save`]. See
    /// that method for details.
    ///
    /// [`save`]: RenderContext::save
    fn pop(&mut self) -> Result<(), Self::Error>;

    /// Save the context state.
    ///
    /// Push a new context state onto the stack. See [`pop`] for details.
    ///
    /// [`pop`]: RenderContext::pop
    fn save(&mut self) -> Result<(), Self::Error>;

    /// Restore the context state.
    ///
    /// Pop a context state that was pushed by [`save`]. See
    /// that method for more details.
    ///
    /// [`save`]: RenderContext::save
    fn restore(&mut self) -> Result<(), Self::Error>;

    /// Draw a text.
    ///
    /// The `pos` parameter specifies the upper-left corner of the text
    fn draw_text(&mut self, position: Point, text: &str, style: Style);

    /// Do graphics operations with the context state saved and then restored.
    ///
    /// Equivalent to [`save`], calling `f`, then
    /// [`restore`]. See those methods for more details.
    ///
    /// [`restore`]: RenderContext::restore
    /// [`save`]: RenderContext::save
    fn with_state(&mut self, f: impl FnOnce(&mut Self) -> Result<(), Self::Error>) -> Result<(), Self::Error> {
        self.save()?;
        f(self).and(self.restore())
    }


    /// Resize the canvas, if necessary.
    fn maybe_resize(&mut self, size: Size) -> Result<(), Self::Error>;

    /// Finish any pending operations.
    fn finish(&mut self) -> Result<(), Self::Error>;
}

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Renderer<B: RenderContext>(pub B);

impl<B: RenderContext> Renderer<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }

    pub fn render(
        &mut self,
        doc: &Document<'_>,
    ) -> Result<(), B::Error> {
        self.maybe_resize(doc.size(doc.root))?;
        self.render_node(doc, doc.root, crate::Style::Default)?;
        self.finish()
    }

    fn render_node(
        &mut self,
        doc: &Document<'_>,
        id: NodeId,
        inherited: Style,
    ) -> Result<(), B::Error> {
        let node = doc.node(id);
        let rect = doc.bounds(id);
        let content_rect = doc.content_bounds(id);
        // Background fill.
        let resolved = node.style;
        if resolved.background_color != Color::None {
            dbg!("background", rect, resolved.background_color);
        }
        if resolved.background_color != Color::None {
            self.fill_style(rect, resolved);
        }

        // Border.
/*        if matches!(node.style.border, Border::Solid) {
            let border_paint = resolved;
            paint_border(rect, node.style.border, border_paint)?;
        }*/

        // Leaf text.
        if let NodeKind::Span(text) = &node.kind {
            let pos = content_rect.min;

            // MVP: left-aligned only in actual paint pass.
            // Center/right can be added once line layout is shared with measurement.
            self.draw_text(pos, text, resolved);
        }

        for child in doc.children(id) {
            self.render_node(doc, child, resolved)?;
        }

        Ok(())
    }
}

fn paint_border<B: RenderContext>(
    rect: Rect<Point<usize>>,
    border: Border,
    style: Style,
    backend: &mut B,
) -> Result<(), B::Error> {
    /*let w = rect.max.x.saturating_sub(rect.min.x);
    let h = rect.max.y.saturating_sub(rect.min.y);

    if w == 0 || h == 0 {
        return Ok(());
    }

    let g = border.glyphs;

    // Top / bottom
    if border.widths.top > 0 && h >= 1 {
        for x in rect.min.x..rect.max.x {
            let ch = if x == rect.min.x {
                g.top_left
            } else if x + 1 == rect.max.x {
                g.top_right
            } else {
                g.top
            };

            backend.write_text(Point { x, y: rect.min.y }, &ch.to_string(), style)?;
        }
    }

    if border.widths.bottom > 0 && h >= 1 {
        let y = rect.max.y - 1;
        for x in rect.min.x..rect.max.x {
            let ch = if x == rect.min.x {
                g.bottom_left
            } else if x + 1 == rect.max.x {
                g.bottom_right
            } else {
                g.bottom
            };

            backend.write_text(Point { x, y }, &ch.to_string(), style)?;
        }
    }

    // Left / right
    if border.widths.left > 0 && w >= 1 {
        for y in (rect.min.y + 1)..rect.max.y.saturating_sub(1) {
            backend.write_text(Point { x: rect.min.x, y }, &g.left.to_string(), style)?;
        }
    }

    if border.widths.right > 0 && w >= 1 {
        let x = rect.max.x - 1;
        for y in (rect.min.y + 1)..rect.max.y.saturating_sub(1) {
            backend.write_text(Point { x, y }, &g.right.to_string(), style)?;
        }
    }*/

    Ok(())
}