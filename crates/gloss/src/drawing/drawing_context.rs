use std::io;
use bon::Builder;
use geometry::{Bounds, Point, Rect, Size};
use maybe::Maybe;
use crate::{Border, Document, ElementId, ElementKind, Layout};

/// Per-call style overrides for fill operations.
#[derive(Debug, Clone, Default, Builder, Copy)]
pub struct DrawingOptions {
    pub layout: Option<Layout>,
    pub glyph: Option<char>,
    pub border: Option<Border>,
}

pub trait DrawingContext {
    type Error;
    type Options: From<DrawingOptions>;

    /// Get the current clip.
    fn current_clip(&self) -> Rect;

    /// Get the current style.
    fn current_style(&self) -> Layout;

    /// Get the current fill glyph.
    fn current_glyph(&self) -> char;

    /// Get the current border style.
    fn current_border_style(&self) -> Border;

    /// Set the current style.
    fn style(&mut self, style: Layout) -> &mut Self;

    /// Set the current fill glyph.
    fn glyph(&mut self, fill: char) -> &mut Self;

    /// Set the current border style.
    fn border_style(&mut self, border: Border) -> &mut Self;

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
    fn rect_with(&mut self, rect: Rect, options: Self::Options) -> &mut Self;

    /// Draw an outline.
    fn outline(&mut self, rect: Rect) -> &mut Self;
    fn outline_with(&mut self, rect: Rect, options: Self::Options) -> &mut Self;

    /// Draw a border.
    fn border(&mut self, rect: Rect) -> &mut Self;
    fn border_with(&mut self, rect: Rect, options: Self::Options) -> &mut Self;

    /// Draw text.
    ///
    /// The `pos` parameter specifies the upper-left corner of the text
    fn text(&mut self, position: Point, str: impl AsRef<str>) -> usize;
    fn text_with(&mut self, position: Point, str: impl AsRef<str>, options: Self::Options) -> usize;

    /// Draw a horizontal line.
    fn horizontal_line(&mut self, position: Point, length: u16) -> &mut Self;
    fn horizontal_line_with(&mut self, position: Point, length: u16, options: Self::Options) -> &mut Self;

    /// Draw a vertical line.
    fn vertical_line(&mut self, position: Point, length: u16) -> &mut Self;
    fn vertical_line_with(&mut self, position: Point, length: u16, options: Self::Options) -> &mut Self;

    /// Clear an area.
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

    /// Do graphics operations within a sub-region.
    /// 
    /// Equivalent to:
    /// ```ignore
    /// self.save();
    /// self.translate(rect.min);
    /// self.clip(Rect::from(rect.size()));
    /// f(self);
    /// self.restore();
    /// ```
    fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self;

    /// Resize the canvas, if necessary.
    fn resize(&mut self, size: impl Into<Size>) -> &mut Self;

    /// Finish any pending operations.
    fn finish(&mut self) -> &mut Self;

    /// Paint a document into this context.
    ///
    /// Resizes to fit the document's root, traverses the element tree
    /// applying styles/borders/content, then flushes any pending work.
    fn paint(&mut self, document: &Document<'_>) {
        self.resize(document.border_bounds(document.root_id));
        paint_node(self, document, document.root_id, Layout::DEFAULT);
        self.finish();
    }
}

fn paint_node<B: DrawingContext + ?Sized>(
    ctx: &mut B,
    document: &Document<'_>,
    id: ElementId,
    parent_style: Layout,
) {
    let node = document.element(id);
    let border_bounds = document.border_bounds(id);
    let content_bounds = document.content_bounds(id);
    let style = node.style.inherit(parent_style);

    ctx.save();

    if style.border.is_some() {
        ctx.border(border_bounds);
    }

    ctx.translate(border_bounds.min)
        .clip(border_bounds.size().into())
        .style(style)
        .border_style(node.border);

    // Children's taffy locations are border-box relative, so clip/bg use
    // content bounds normalized into the node's own origin.
    let normalized_bounds = content_bounds - border_bounds.min;

    if style.background.is_some() {
        ctx.rect(normalized_bounds);
    }

    match &node.kind {
        ElementKind::Span(text) => {
            ctx.text(normalized_bounds.min, text);
        }
        ElementKind::Div => {}
    }

    ctx.save();
    ctx.clip(normalized_bounds);

    for child in document.children(id) {
        paint_node(ctx, document, child, style);
    }

    ctx.restore();
    ctx.restore();
}
