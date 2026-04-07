use bon::Builder;
use crate::{BorderStyle, Document, NodeId, NodeKind, Style};
use maybe::Maybe;
use geometry::{Bounded, Point, Rect, Size};
use derive_more::{AsMut, AsRef, Deref, DerefMut};

// ── Options Structs ────────────────────────────────────────────────────────
//
// Lightweight per-call override containers. `None` fields inherit from
// the current context state. All methods are `const` to enable static
// construction without runtime overhead.

/// Per-call style overrides for fill operations.
#[derive(Debug, Clone, Default, Builder, Copy)]
pub struct PainterOptions {
    pub style: Option<Style>,
    pub glyph: Option<char>,
    pub border: Option<BorderStyle>,
}

impl PainterOptions {
    fn new() -> PainterOptions {
        Self::default()
    }
}


pub trait DrawingContext {
    type Error;

    /// Get the current clip.
    fn current_clip(&self) -> Rect;

    /// Get the current style.
    fn current_style(&self) -> Style;

    /// Get the current fill glyph.
    fn current_glyph(&self) -> char;

    /// Get the current border style.
    fn current_border_style(&self) -> BorderStyle;

    /// Set the current style.
    fn style(&mut self, style: Style) -> &mut Self;

    /// Set the current fill glyph.
    fn glyph(&mut self, fill: char) -> &mut Self;

    /// Set the current border style.
    fn border_style(&mut self, border: BorderStyle) -> &mut Self;

    /// Clip to a [`Rect`].
    ///
    /// All subsequent drawing operations up to the next [`restore`]
    /// are clipped by the bounds.
    ///
    /// [`restore`]: DrawingContext::restore
    fn clip(&mut self, rect: Rect) -> &mut Self;

    /// Translate the origin.
    fn translate(&mut self, offset: Point) -> &mut Self;

    /// Fill an area with a style.
    fn rect(&mut self, rect: Rect) -> &mut Self;
    fn rect_with(&mut self, rect: Rect, options: PainterOptions) -> &mut Self;

    /// Draw an outline (edges without corners) using current fill state.
    fn outline(&mut self, rect: Rect) -> &mut Self;
    fn outline_with(&mut self, rect: Rect, options: PainterOptions) -> &mut Self;

    /// Stroke a [`Shape`], using the default [`Style`].
    fn border(&mut self, rect: Rect) -> &mut Self;
    fn border_with(&mut self, rect: Rect, options: PainterOptions) -> &mut Self;

    /// Draw a text.
    ///
    /// The `pos` parameter specifies the upper-left corner of the text
    fn text(&mut self, position: Point, str: impl AsRef<str>) -> usize;
    fn text_with(&mut self, position: Point, str: impl AsRef<str>, options: PainterOptions) -> usize;

    /// Draw a horizontal line.
    fn horizontal_line(&mut self, position: Point, length: usize) -> &mut Self;
    fn horizontal_line_with(&mut self, position: Point, length: usize, options: PainterOptions) -> &mut Self;

    /// Draw a vertical line.
    fn vertical_line(&mut self, position: Point, length: usize) -> &mut Self;
    fn vertical_line_with(&mut self, position: Point, length: usize, options: PainterOptions) -> &mut Self;

    fn clear(&mut self, rect: Rect) -> &mut Self;

    /// Save the context state.
    ///
    /// Push a new context state onto the stack. See [`pop`] for details.
    ///
    /// [`pop`]: DrawingContext::restore
    fn save(&mut self) ->  &mut Self;

    /// Restore the context state.
    ///
    /// Pop a context state that was pushed by [`save`]. See
    /// that method for more details.
    ///
    /// [`save`]: DrawingContext::save
    fn restore(&mut self) -> &mut Self;

    /// Do graphics operations with the context state saved and then restored.
    ///
    /// Equivalent to [`save`], calling `f`, then
    /// [`restore`]. See those methods for more details.
    ///
    /// [`restore`]: DrawingContext::restore
    /// [`save`]: DrawingContext::save
    fn with(
        &mut self,
        f: impl FnOnce(&mut Self),
    ) -> &mut Self;

    fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self;

    /// Resize the canvas, if necessary.
    fn resize(&mut self, size: Size) -> &mut Self;

    /// Finish any pending operations.
    fn finish(&mut self) -> &mut Self;

    fn into_renderer(self) -> Painter<Self> where Self: Sized {
        Painter(self)
    }
}

#[derive(Default, Deref, DerefMut, AsRef, AsMut)]
pub struct Painter<B: DrawingContext>(pub B);

impl<B: DrawingContext> Painter<B> {
    pub fn render(&mut self, document: &Document<'_>) {
        self.resize(document.size(document.root));
        self.render_node(document, document.root, Style::DEFAULT);
        self.finish();
    }

    fn render_node(
        &mut self,
        document: &Document<'_>,
        id: NodeId,
        parent_style: Style,
    ) {
        let node = document.node(id);
        let bounds = document.bounds(id);
        let content_bounds = document.content_bounds(id);
        let style = node.style.inherit(parent_style);
        // Snapshot the current state
        self.save();

        // Apply this node's style and clip
        self
            .translate(bounds.min)
            .clip(bounds.size().into())
            .style(style)
            .border_style(node.border);

        // Normalize content bounds to be relative to the node's border box.
        let content_bounds = content_bounds - bounds.min;

        // Paint
        if style.background.is_some() { self.rect(content_bounds); }
        if style.border.is_some() { self.border(content_bounds); }

        match &node.kind {
            NodeKind::Span(text) => {
                self.text(content_bounds.min, text);
            }
            NodeKind::Div => {
            }
        }

        // Recurse — clip to *content* area so children don't paint over padding/borders.
        // Don't translate — child bounds from taffy are relative to the parent's
        // border box, so translating to the content area would double-count padding.
        self.save();
        self.clip(content_bounds);

        for child in document.children(id) {
            self.render_node(document, child, style);
        }

        self.restore();

        self.restore();
    }
}
