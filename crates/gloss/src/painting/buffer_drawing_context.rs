use std::io;
use std::ops::Sub;
use bon::{Builder};
use derive_more::{Deref, DerefMut};
use unicode_segmentation::UnicodeSegmentation;
use geometry::{Bounded, Contains, Intersect, Outer, Point, Ranges, Rect, Edges, Sides, Size, Translate, Resolve};
use crate::{Buffer, Arena, Cell, PainterOptions};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::{BorderStyle, DrawingContext, Painter};
use crate::symbols::Symbol;
use ansi::{Attribute, Color, Style};

// ── Options Structs ────────────────────────────────────────────────────────
//
// Lightweight per-call override containers. `None` fields inherit from
// the current context state. All methods are `const` to enable static
// construction without runtime overhead.

/// Per-call style overrides for fill operations.
#[derive(Debug, Clone, Default, Builder, Copy)]
pub struct DrawingOptions {
    pub style: Option<Style>,
    pub glyph: Option<char>,
    pub border: Option<BorderStyle>,
}

impl DrawingOptions {
    fn new() -> DrawingOptions {
       Self::default()
    }
}

impl From<PainterOptions> for DrawingOptions {
    fn from(value: PainterOptions) -> Self {
        Self {
            style: value.style.map(Into::into),
            glyph: value.glyph,
            border: value.border,
        }
    }
}

/// Snapshot of all context state, pushed/popped via save/restore.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Context {
    clip: Rect,
    origin: Point,
    pub style: Style,
    pub border_style: BorderStyle,
    pub glyph: char,
}

/// 2D drawing context for terminal buffers.
///
/// Modeled after HTML Canvas — mutable "current state" with a save/restore
/// stack. All coordinates are relative to `origin`; all draws are clipped
/// to the current clip rect.
#[derive(Debug, Deref, DerefMut)]
pub struct BufferDrawingContext<'a> {
    buffer: &'a mut Buffer,
    arena: &'a mut Arena,
    #[deref]
    #[deref_mut]
    context: Context,
    stacks: Vec<Context>,
}

impl<'buf> BufferDrawingContext<'buf> {
    /// Create a new context spanning the full buffer.
    pub fn new(buffer: &'buf mut Buffer, arena: &'buf mut Arena) -> Self {
        let clip = buffer.bounds(); // full buffer rect
        Self {
            buffer,
            arena,
            context: Context {
                clip,
                origin: Point::ZERO,
                style: Style::None,
                glyph: ' ',
                border_style: BorderStyle::None,
            },
            stacks: Vec::new(),
        }
    }
    // ── State Mutations ────────────────────────────────────────────────

    /// Set the fill style for subsequent draw operations.
    pub fn style(&mut self, style: Style) -> &mut Self {
        self.context.style = style;
        self
    }

    pub fn foreground(&mut self, color: Color) -> &mut Self {
        self.context.style.foreground = color;
        self
    }

    pub fn background(&mut self, color: Color) -> &mut Self {
        self.context.style.background = color;
        self
    }

    pub fn attributes(&mut self, attributes: Attribute) -> &mut Self {
        self.context.style.attributes = attributes;
        self
    }

    /// Set the fill glyph for subsequent draw operations.
    pub fn glyph(&mut self, glyph: char) -> &mut Self {
        self.context.glyph = glyph;
        self
    }

    /// Set the border style for subsequent stroke operations.
    pub fn border_style(&mut self, border: BorderStyle) -> &mut Self {
        self.context.border_style = border;
        self
    }

    /// Intersect the current clip region with `rect`.
    ///
    /// The input is in local coordinates and will be transformed before
    /// intersection. Use `clip_intersect()` to pass buffer-space coordinates
    /// directly.
    pub fn clip(&mut self, rect: Rect) -> &mut Self {
        self.context.clip = self.context.clip.intersect(&self.to_local(rect));
        self
    }

    /// Shift the origin by `offset`. Cumulative within a save/restore frame.
    pub fn translate(&mut self, offset: Point) -> &mut Self {
        self.context.origin = self.context.origin + offset;
        self
    }

    /// Push current state onto the stack.
    pub fn save(&mut self) -> &mut Self {
        self.stacks.push(self.context.clone());
        self
    }

    /// Pop state from the stack, restoring previous values.
    ///
    /// No-op if the stack is empty.
    pub fn restore(&mut self) -> &mut Self {
        if let Some(previous) = self.stacks.pop() {
            self.context = previous;
        }
        self
    }

    /// Reset state to defaults without affecting the stack.
    pub fn reset(&mut self) -> &mut Self {
        self.context = Context::default();
        self
    }

    /// Execute `f` with a temporary state modification.
    ///
    /// State is saved before `f` runs and restored afterward, regardless
    /// of whether `f` returns or panics (on panic, the restore is skipped
    /// — use a `Drop` guard if panic safety is required).
    pub fn with(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        f(self);
        self.restore()
    }

    /// Execute `f` within a sub-region.
    ///
    /// Equivalent to:
    /// ```ignore
    /// self.save();
    /// self.translate(rect.min);
    /// self.clip(Rect::from(rect.size()));
    /// f(self);
    /// self.restore();
    /// ```
    pub fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        self.translate(rect.min);
        self.clip(Rect::from(rect.size()));
        f(self);
        self.restore()
    }


    /// Fill a rectangle using current fill state.
    pub fn rect(&mut self, rect: Rect) -> &mut Self {
        self.rect_with(rect, DrawingOptions::default())
    }

    /// Draw an outline (edges without corners) using current fill state.
    pub fn outline(&mut self, rect: Rect) -> &mut Self {
        self.outline_with(rect, DrawingOptions::default())
    }

    /// Draw a border with corners using current stroke state.
    pub fn border(&mut self, rect: Rect) -> &mut Self {
        self.border_with(rect, DrawingOptions::default())
    }

    /// Draw text at `pos` using current fill state.
    ///
    /// Returns the number of cells written (accounts for wide characters).
    pub fn text(&mut self, pos: Point, content: impl AsRef<str>) -> usize {
        self.text_with(pos, content, DrawingOptions::default())
    }

    /// Draw a horizontal line from `origin` using current fill state.
    pub fn horizontal_line(&mut self, origin: Point, length: usize) -> &mut Self {
        self.horizontal_line_with(origin, length, DrawingOptions::default())
    }

    /// Draw a vertical line from `origin` using current fill state.
    pub fn vertical_line(&mut self, origin: Point, length: usize) -> &mut Self {
        self.vertical_line_with(origin, length, DrawingOptions::default())
    }

    /// Fill the current clip region using current fill state.
    ///
    /// Equivalent to `self.rect(self.current_clip())`.
    pub fn clear(&mut self, rect: Rect) -> &mut Self {
        self.rect(rect)
    }

    // ── Draw Operations with Overrides ────────────────────────────────

    /// Fill a rectangle with per-call style/glyph overrides.
    pub fn rect_with(&mut self, rect: Rect, options: DrawingOptions) -> &mut Self {
        let local_rect = self.to_local(rect);
        let style = options.style.unwrap_or(self.context.style);
        let glyph = options.glyph.unwrap_or(self.context.glyph);

        if let Some(clipped) = self.intersect(local_rect) {
            for pos in &clipped {
                let index: usize = self.buffer.bounds().resolve(pos);
                self.buffer[index].set_char(glyph, self.arena).set_style(style);
            }
        }

        self
    }

    /// Draw an outline with per-call style/glyph overrides.
    pub fn outline_with(&mut self, rect: Rect, options: DrawingOptions) -> &mut Self {
        let local_rect = self.to_local(rect);

        // Top edge
        self.horizontal_line_with(local_rect.min, local_rect.width(), options);

        // Bottom edge
        if local_rect.height() > 1 {
            self.horizontal_line_with(
                local_rect.bottom_left().saturating_sub(Point::new(0, 1)),
                local_rect.width(),
                options,
            );
        }

        // Left edge (excluding corners)
        if local_rect.height() > 2 {
            self.vertical_line_with(
                local_rect.top_left().saturating_add(Point::new(0, 1)),
                local_rect.height() - 2,
                options,
            );
        }

        // Right edge (excluding corners)
        if local_rect.width() > 1 && local_rect.height() > 2 {
            self.vertical_line_with(
                Point::new(
                    local_rect.right().saturating_sub(1),
                    local_rect.top().saturating_add(1),
                ),
                local_rect.height() - 2,
                options,
            );
        }

        self
    }

    /// Draw a border with corners, with per-call stroke overrides.
    pub fn border_with(&mut self, rect: Rect, options: DrawingOptions) -> &mut Self {
        let mut local_rect = self.to_local(rect);
        let border_style = options.border.unwrap_or(self.context.border_style);
        let border = border_style.into_border();

        // Shrink rect to account for border thickness
        local_rect.max.x = local_rect.max.x.saturating_sub(border.right.width());
        local_rect.max.y = local_rect.max.y.saturating_sub(border.bottom.width());

        if local_rect.is_empty() {
            return self;
        }

        // Helper: set cell if within clip
        let mut set_cell = |x: usize, y: usize, symbol: Symbol| {
            if self.context.clip.contains(&(x, y)) {
                self.buffer[(x, y)]
                    .set_char_measured(symbol.symbol(), symbol.width(), self.arena);
            }
        };

        // Corners
        set_cell(local_rect.left(), local_rect.top(), border.top_left);
        set_cell(local_rect.right(), local_rect.top(), border.top_right);
        set_cell(local_rect.left(), local_rect.bottom(), border.bottom_left);
        set_cell(local_rect.right(), local_rect.bottom(), border.bottom_right);

        // Horizontal edges
        let left_offset = border.left.width();
        let right_bound = local_rect.right();
        for x in (local_rect.left() + left_offset)..right_bound {
            set_cell(x, local_rect.top(), border.top);
            set_cell(x, local_rect.bottom(), border.bottom);
        }

        // Vertical edges
        let top_offset = border.top.width();
        let bottom_bound = local_rect.bottom();
        for y in (local_rect.top() + top_offset)..bottom_bound {
            set_cell(local_rect.left(), y, border.left);
            set_cell(local_rect.right(), y, border.right);
        }

        self
    }

    /// Draw text at `pos` with per-call style overrides.
    ///
    /// Returns the number of cells written (accounts for wide characters).
    pub fn text_with(
        &mut self,
        pos: Point,
        content: impl AsRef<str>,
        options: DrawingOptions,
    ) -> usize {
        let local_pos = self.to_local(pos);
        let style = options.style.unwrap_or(self.context.style);
        let y = local_pos.y;
        let mut cells_written = 0;

        for (grapheme, width) in content
            .as_ref()
            .graphemes(true)
            .map(|grapheme| (grapheme, grapheme.width()))
        {
            let x = local_pos.x + cells_written;

            // Stop if past clip right edge
            if x + width > self.context.clip.right() {
                break;
            }

            if self.context.clip.contains(&(x, y)) {
                self.buffer[(x, y)]
                    .set_str_measured(grapheme, width, self.arena);

                // Mark continuation cells for wide characters
                for offset in 1..width {
                    let continuation_pos = (x + offset, y);
                    if self.context.clip.contains(&continuation_pos) {
                        self.buffer[continuation_pos]
                            .set_continuation(self.arena)
                            .set_style(style);
                    }
                }
            }

            cells_written += width;
        }

        cells_written
    }

    /// Draw a horizontal line with per-call style/glyph overrides.
    pub fn horizontal_line_with(
        &mut self,
        origin: Point,
        length: usize,
        options: DrawingOptions,
    ) -> &mut Self {
        let local_origin = self.to_local(origin);
        let style = options.style.unwrap_or(self.context.style);
        let glyph = options.glyph.unwrap_or(self.context.glyph);

        let end = (
            local_origin.x.saturating_add(length),
            local_origin.y,
        );

        if self.context.clip.contains(&local_origin) && self.context.clip.contains(&end) {
            for offset in 0..length {
                self.buffer[(local_origin.x.saturating_add(offset), local_origin.y)]
                    .set_style(style)
                    .set_char(glyph, self.arena);
            }
        }

        self
    }

    /// Draw a vertical line with per-call style/glyph overrides.
    pub fn vertical_line_with(
        &mut self,
        origin: Point,
        length: usize,
        opts: DrawingOptions,
    ) -> &mut Self {
        let local_origin = self.to_local(origin);
        let style = opts.style.unwrap_or(self.context.style);
        let glyph = opts.glyph.unwrap_or(self.context.glyph);

        let end = (
            local_origin.x,
            local_origin.y.saturating_add(length),
        );

        if self.context.clip.contains(&local_origin) && self.context.clip.contains(&end) {
            for offset in 0..length {
                self.buffer[(local_origin.x, local_origin.y.saturating_add(offset))]
                    .set_style(style)
                    .set_char(glyph, self.arena);
            }
        }

        self
    }

    // Utils
    fn to_local<T: Translate<Point>>(&self, rect: T) -> T::Output {
        rect.translate(&self.context.origin)
    }

    fn intersect(&self, rect: Rect) -> Option<Rect> {
        let result = self.context.clip
            .intersect(&rect);

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }


}

impl<'a> Painter<BufferDrawingContext<'a>> {
    pub fn new(buffer: &'a mut Buffer, arena: &'a mut Arena) -> Self {
        Self(BufferDrawingContext::new(buffer, arena))
    }
}

impl<'a> DrawingContext for BufferDrawingContext<'a> {
    type Error = io::Error;

    fn current_clip(&self) -> Rect {
        self.context.clip
    }

    fn current_style(&self) -> crate::Style {
       self.context.style.into()
    }

    fn current_glyph(&self) -> char {
        self.context.glyph
    }

    fn current_border_style(&self) -> BorderStyle {
        self.context.border_style
    }

    fn style(&mut self, style: crate::Style) -> &mut Self {
        self.style(style.into())
    }

    fn glyph(&mut self, fill: char) -> &mut Self {
        self.glyph(fill)
    }

    fn border_style(&mut self, border: BorderStyle) -> &mut Self {
        self.border_style(border)
    }

    fn clip(&mut self, rect: Rect) -> &mut Self {
        self.clip(rect)
    }

    fn translate(&mut self, offset: Point) -> &mut Self {
        self.translate(offset)
    }

    fn rect(&mut self, rect: Rect) -> &mut Self {
        self.rect(rect)
    }

    fn rect_with(&mut self, rect: Rect, options: PainterOptions) -> &mut Self {
        self.rect_with(rect, options.into())
    }

    fn outline(&mut self, rect: Rect) -> &mut Self {
        self.outline(rect)
    }

    fn outline_with(&mut self, rect: Rect, options: PainterOptions) -> &mut Self {
        self.outline_with(rect, options.into())
    }

    fn border(&mut self, rect: Rect) -> &mut Self {
        self.border(rect)
    }

    fn border_with(&mut self, rect: Rect, options: PainterOptions) -> &mut Self {
        self.border_with(rect, options.into())
    }

    fn text(&mut self, position: Point, str: impl AsRef<str>) -> usize {
        self.text(position, str)
    }

    fn text_with(&mut self, position: Point, str: impl AsRef<str>, options: PainterOptions) -> usize {
        self.text_with(position, str, options.into())
    }

    fn horizontal_line(&mut self, position: Point, length: usize) -> &mut Self {
        self.horizontal_line(position, length)
    }

    fn horizontal_line_with(&mut self, position: Point, length: usize, options: PainterOptions) -> &mut Self {
        self.horizontal_line_with(position, length, options.into())
    }

    fn vertical_line(&mut self, position: Point, length: usize) -> &mut Self {
        self.vertical_line(position, length)
    }

    fn vertical_line_with(&mut self, position: Point, length: usize, options: PainterOptions) -> &mut Self {
        self.vertical_line_with(position, length, options.into())
    }

    fn clear(&mut self, rect: Rect) -> &mut Self {
        self.clear(rect)
    }

    fn save(&mut self) -> &mut Self {
        self.save()
    }

    fn restore(&mut self) -> &mut Self {
        self.restore()
    }

    fn with(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.with(f)
    }

    fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.within(rect, f)
    }

    fn resize(&mut self, size: Size) -> &mut Self {
        self.buffer.resize(size.width, size.height);
        self.context.clip = self.buffer.bounds();
        self
    }

    fn finish(&mut self) -> &mut Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::ops::{Sub};
    use ansi::Color;
    use crate::Grapheme;
    use tree::At;
    use crate::{Document, FlexDirection, FontWeight, Node, TextDecoration};
    use super::*;

    struct Context<'a> {
        buffer: Buffer,
        arena: Arena,
        document: Document<'a>,
    }

    fn add_content(context: &mut Context) {
        let document = &mut context.document;
        let root = document.node_mut(document.root);

        root.border = BorderStyle::Solid;
        root.margin = (2, 2).into();
        root.padding = (1, 1).into();

        let heading = document.insert_with(
            Node::Span(Cow::Borrowed("Title")),
            |node| {
                node.color = Some(Color::Red);
                node.text_decoration = Some(TextDecoration::Underline);
                node.font_weight = Some(FontWeight::Bold);
            },
        );

        let footer = document.insert_with(Node::Div(), |node| {
            node.background = Some(Color::BrightBlack);
            node.flex_direction = FlexDirection::Row;
        });

        let footer_left = document.insert_at_with(Node::Div(), At::Child(footer), |node| {
            node.padding = (1, 1).into();
        });
        let footer_left_content = document.insert_at(Node::Span("Gloss Rendering"), At::Child(footer_left));

        let footer_right = document.insert_at_with(Node::Div(), At::Child(footer), |node| {
            node.padding = (1, 1).into();
        });
        let footer_right_content = document.insert_at(Node::Span("Test Consortium"), At::Child(footer_right));

    }

    fn context<'a>(width: usize, height: usize) -> Context<'a> {
        let mut arena = Arena::new();
        let mut buffer = Buffer::new(width, height);
        let mut document = Document::new();

        Context {
            buffer,
            arena,
            document,
        }
    }

    fn renderer<'a>(context: &'a mut Context) -> Painter<BufferDrawingContext<'a>> {
        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).into_renderer()
    }

    #[test]
    fn test_basic_fill() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.style = Style::default().foreground(Color::White);
        renderer.glyph = 'x';
        renderer.rect(Rect::new(0, 0, 10, 10));

        assert_eq!(context.buffer.iter().all(|c| c.style.foreground == Color::White && c.grapheme() == Grapheme::inline('x')), true);
    }


    #[test]
    fn test_basic_stroke() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.border_style = BorderStyle::Solid;
        renderer.border(Rect::new(0, 0, 10, 10));

        assert_eq!(context.buffer.iter_row(0).all(|c| c.grapheme() != Grapheme::SPACE), true);
        assert_eq!(context.buffer.iter_col(0).all(|c| c.grapheme() != Grapheme::SPACE), true);
        assert_eq!(context.buffer.iter_col(9).all(|c| c.grapheme() != Grapheme::SPACE), true);
        assert_eq!(context.buffer.iter_row(9).all(|c| c.grapheme() != Grapheme::SPACE), true);
        context.buffer.iter_rect(&context.buffer.bounds().sub(Edges::all(1))).for_each(|c| assert_eq!(c.grapheme(), Grapheme::SPACE));
    }

    #[test]
    fn test_save_restore_origin() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.save();
        renderer.translate(Point::new(3, 3));
        renderer.text(Point::ZERO, "A");
        renderer.restore();

        // After restore, origin is back to (0,0)
        renderer.text(Point::ZERO, "B");

        assert_eq!(context.buffer[(3, 3)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('B'));
    }

    #[test]
    fn test_save_restore_clip() {
        let mut context = context(10, 10);

        {
            let mut renderer = renderer(&mut context);
            renderer.glyph = 'X';
            renderer.save();
            renderer.clip(Rect::from(Size::new(5, 5)));
            renderer.rect(Rect::from(Size::new(15, 5)));
            renderer.restore();
        }

        // Inside the old clip — should be filled
        assert_eq!(context.buffer[(2, 2)].grapheme(), Grapheme::inline('X'));
        // Outside the old clip — should be empty
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::SPACE);

        {
            // After restore, full clip is back — can write outside
            let mut renderer = renderer(&mut context);
            renderer.text(Point::new(7, 7), "Y");
        }
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::inline('Y'));
    }

    #[test]
    fn test_within_scoped_translate_and_clip() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.glyph = 'W';
        renderer.within(Rect::bounds(Point::new(2, 2), Point::new(6, 6)), |r| {
            r.rect(Rect::new(0, 0, 5, 5));
        });

        // Inside the within rect — filled
        assert_eq!(context.buffer[(3, 3)].grapheme(), Grapheme::inline('W'));
        // Outside — empty
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::SPACE);
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_nested_translate() {
        let mut context = context(20, 20);
        let mut renderer = renderer(&mut context);

        // Translate twice — offsets accumulate within the same save frame
        renderer.save();
        renderer.translate(Point::new(3, 3));
        renderer.save();
        renderer.translate(Point::new(2, 2));
        renderer.text(Point::ZERO, "N");
        renderer.restore();
        renderer.restore();

        // (3+2, 3+2) = (5, 5)
        assert_eq!(context.buffer[(5, 5)].grapheme(), Grapheme::inline('N'));
    }

    #[test]
    fn test_draw_text_position() {
        let mut context = context(20, 5);
        let mut renderer = renderer(&mut context);

        renderer.text(Point::new(4, 1), "Hi");

        assert_eq!(context.buffer[(4, 1)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(5, 1)].grapheme(), Grapheme::inline('i'));
        // Adjacent cell untouched
        assert_eq!(context.buffer[(6, 1)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_draw_text_clipped() {
        let mut context = context(10, 5);
        let mut renderer = renderer(&mut context);

        renderer.clip(Rect::from(Size::new(4, 5)));
        renderer.text(Point::new(0, 0), "Hello");

        // Only first 4 chars fit in clip
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(3, 0)].grapheme(), Grapheme::inline('l'));
        // 5th char ('o') is outside clip — cell stays empty
        assert_eq!(context.buffer[(4, 0)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_render_document_with_padding() {
        use crate::Space;

        let mut context = context(20, 10);
        let document = &mut context.document;

        // Root with padding — children should render inside the content area
        let root = document.node_mut(document.root);
        root.padding = (2, 2).into();
        root.flex_direction = FlexDirection::Column;

        let child = document.insert_with(
            Node::Span(Cow::Borrowed("AB")),
            |node| { node.color = Some(Color::Blue); },
        );

        document.compute_layout(Space::new(20u32, 10u32));

        let mut renderer = BufferDrawingContext::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document);

        // Text should appear at content area offset (padding=2 on each side)
        let child_content = document.content_bounds(child);
        let text_x = child_content.min.x;
        let text_y = child_content.min.y;
        assert_eq!(context.buffer[(text_x, text_y)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(text_x + 1, text_y)].grapheme(), Grapheme::inline('B'));
        // Origin cell should be empty (it's in the padding)
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_render_nested_nodes() {
        use crate::Space;

        let mut context = context(30, 15);
        let document = &mut context.document;

        // Root with padding, column layout
        let root = document.node_mut(document.root);
        root.padding = (1, 1).into();
        root.flex_direction = FlexDirection::Column;

        // Child div with its own padding
        let child_div = document.insert_with(Node::Div(), |node| {
            node.padding = (1, 1).into();
            node.flex_direction = FlexDirection::Column;
        });

        // Grandchild text inside the child div
        let text_id = document.insert_at_with(
            Node::Span("OK"),
            At::Child(child_div),
            |node| { node.color = Some(Color::Blue); },
        );

        document.compute_layout(Space::new(30u32, 15u32));

        let text_content = document.content_bounds(text_id);

        let mut renderer = BufferDrawingContext::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document);

        let div_bounds = document.bounds(child_div);
        let text_bounds = document.bounds(text_id);

        // Absolute position = parent bounds + child bounds (taffy locations are parent-relative)
        let tx = div_bounds.min.x + text_bounds.min.x;
        let ty = div_bounds.min.y + text_bounds.min.y;
        assert_eq!(context.buffer[(tx, ty)].grapheme(), Grapheme::inline('O'));
        assert_eq!(context.buffer[(tx + 1, ty)].grapheme(), Grapheme::inline('K'));
        // Padding area should be empty
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::SPACE);


    }

    #[test]
    fn test_render_stacked_children() {
        use crate::Space;

        let mut context = context(30, 15);
        let document = &mut context.document;

        let root = document.node_mut(document.root);
        root.flex_direction = FlexDirection::Column;

        // Two stacked children in column layout
        let child_a = document.insert_with(
            Node::Span(Cow::Borrowed("AA")),
            |node| { node.color = Some(Color::Blue); },
        );
        let child_b = document.insert_with(
            Node::Span(Cow::Borrowed("BB")),
            |node| { node.color = Some(Color::Green); },
        );

        document.compute_layout(Space::new(30u32, 15u32));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        let mut renderer = BufferDrawingContext::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document);

        // First child
        assert_eq!(context.buffer[(a_bounds.min.x, a_bounds.min.y)].grapheme(), Grapheme::inline('A'));
        // Second child should be below the first
        assert!(b_bounds.min.y > a_bounds.min.y, "B should be below A: A.y={}, B.y={}", a_bounds.min.y, b_bounds.min.y);
        assert_eq!(context.buffer[(b_bounds.min.x, b_bounds.min.y)].grapheme(), Grapheme::inline('B'));
    }

    #[test]
    fn test_render_row_children() {
        use crate::Space;

        let mut context = context(30, 5);
        let document = &mut context.document;

        let root = document.node_mut(document.root);
        root.flex_direction = FlexDirection::Row;

        let child_a = document.insert_with(
            Node::Span(Cow::Borrowed("L")),
            |node| { node.color = Some(Color::Blue); },
        );
        let child_b = document.insert_with(
            Node::Span(Cow::Borrowed("R")),
            |node| { node.color = Some(Color::Green); },
        );

        document.compute_layout(Space::new(30u32, 5u32));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        let mut renderer = BufferDrawingContext::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document);

        // Side by side in row layout
        assert_eq!(context.buffer[(a_bounds.min.x, a_bounds.min.y)].grapheme(), Grapheme::inline('L'));
        assert!(b_bounds.min.x > a_bounds.min.x, "R should be right of L");
        assert_eq!(context.buffer[(b_bounds.min.x, b_bounds.min.y)].grapheme(), Grapheme::inline('R'));
    }
}