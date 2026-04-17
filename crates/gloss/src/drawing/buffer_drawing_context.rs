use std::io;
use std::ops::Sub;
use bon::{Builder};
use derive_more::{Deref, DerefMut};
use smallvec::SmallVec;
use unicode_segmentation::UnicodeSegmentation;
use geometry::{Bound, Contains, Intersect, Outer, Point, Rect, Edges, Size, Translate, Resolve, pos};
use number::{SaturatingSub, SaturatingAdd};
use crate::{Buffer, Arena,  DrawingOptions};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::{Border, DrawingContext};
use crate::symbols::Symbol;
use ansi::{Attribute, Color, Style};

/// Snapshot of all context state, pushed/popped via save/restore.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    clip: Rect,
    origin: Point,
    pub style: Style,
    pub border: Border,
    pub glyph: char,
}

// Lightweight per-call override containers. `None` fields inherit from
// the current context state. All methods are `const` to enable static
// construction without runtime overhead.
#[derive(Debug, Clone, Default, Builder, Copy)]
pub struct BufferDrawingOptions {
    pub style: Option<Style>,
    pub glyph: Option<char>,
    pub border: Option<Border>,
}

impl BufferDrawingOptions {
    fn new() -> BufferDrawingOptions {
       Self::default()
    }
}

impl From<DrawingOptions> for BufferDrawingOptions {
    fn from(value: DrawingOptions) -> Self {
        Self {
            style: value.layout.map(Into::into),
            glyph: value.glyph,
            border: value.border,
        }
    }
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
    state: State,
    stacks: SmallVec<State, 16>,
}

impl<'buf> BufferDrawingContext<'buf> {
    /// Create a new context spanning the full buffer.
    pub fn new(buffer: &'buf mut Buffer, arena: &'buf mut Arena) -> Self {
        let clip = buffer.bounds(); // full buffer rect
        Self {
            buffer,
            arena,
            state: State {
                clip,
                origin: Point::ZERO,
                style: Style::None,
                glyph: ' ',
                border: Border::None,
            },
            stacks: SmallVec::new(),
        }
    }

    /// Set the foreground color for subsequent draw operations.
    pub fn foreground(&mut self, color: Color) -> &mut Self {
        self.style.foreground = color;
        self
    }

    /// Set the background color for subsequent draw operations.
    pub fn background(&mut self, color: Color) -> &mut Self {
        self.style.background = color;
        self
    }

    /// Set the text attributes for subsequent draw operations.
    pub fn attributes(&mut self, attributes: Attribute) -> &mut Self {
        self.style.attributes = attributes;
        self
    }

    /// Reset state to defaults without affecting the stack.
    pub fn reset(&mut self) -> &mut Self {
        self.state = State::default();
        self
    }

    fn to_local<T: Translate<Point>>(&self, rect: T) -> T::Output {
        rect.translate(&self.origin)
    }

    fn intersect(&self, rect: Rect) -> Option<Rect> {
        let result = self.clip.intersect(&rect);
        (!result.is_empty()).then_some(result)
    }
}

impl<'a> DrawingContext for BufferDrawingContext<'a> {
    type Error = io::Error;
    type Options = BufferDrawingOptions;

    fn current_clip(&self) -> Rect {
        self.clip
    }

    fn current_style(&self) -> crate::Layout {
        self.style.into()
    }

    fn current_glyph(&self) -> char {
        self.glyph
    }

    fn current_border_style(&self) -> Border {
        self.border
    }

    fn style(&mut self, style: crate::Layout) -> &mut Self {
        self.style = style.into();
        self
    }

    fn glyph(&mut self, glyph: char) -> &mut Self {
        self.glyph = glyph;
        self
    }

    fn border_style(&mut self, border: Border) -> &mut Self {
        self.border = border;
        self
    }

    /// Intersect the current clip region with `rect`.
    ///
    /// The input is in local coordinates and will be transformed before
    /// intersection.
    fn clip(&mut self, rect: Rect) -> &mut Self {
        self.clip = self.clip.intersect(&self.to_local(rect));
        self
    }

    /// Shift the origin by `offset`. Cumulative within a save/restore frame.
    fn translate(&mut self, offset: Point) -> &mut Self {
        self.origin = self.origin + offset;
        self
    }

    fn rect(&mut self, rect: Rect) -> &mut Self {
        self.rect_with(rect, BufferDrawingOptions::default())
    }

    fn rect_with(&mut self, rect: Rect, options: Self::Options) -> &mut Self {
        let local_rect = self.to_local(rect);
        let style = options.style.unwrap_or(self.style);
        let glyph = options.glyph.unwrap_or(self.glyph);

        if let Some(clipped) = self.intersect(local_rect) {
            for pos in clipped.steps() {
                let index: usize = self.buffer.bounds().resolve(pos);
                self.buffer[index].set_char(glyph, self.arena).set_style(style);
            }
        }

        self
    }

    fn outline(&mut self, rect: Rect) -> &mut Self {
        self.outline_with(rect, BufferDrawingOptions::default())
    }

    fn outline_with(&mut self, rect: Rect, options: Self::Options) -> &mut Self {
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

    fn border(&mut self, rect: Rect) -> &mut Self {
        self.border_with(rect, BufferDrawingOptions::default())
    }

    fn border_with(&mut self, rect: Rect, options: Self::Options) -> &mut Self {
        let mut local_rect = self.to_local(rect);
        let border_style = options.border.unwrap_or(self.border);
        let border = border_style.into_symbols();

        // Shrink rect to account for border thickness
        local_rect.max.x = local_rect.max.x.saturating_sub(border.right.width() as u16);
        local_rect.max.y = local_rect.max.y.saturating_sub(border.bottom.width() as u16);

        if local_rect.is_empty() {
            return self;
        }

        let mut set_cell = |x: u16, y: u16, symbol: Symbol| {
            if self.clip.contains(&(y as usize, x as usize)) {
                self.buffer[(y as usize, x as usize)]
                    .set_char_measured(symbol.symbol(), symbol.width(), self.arena);
            }
        };

        // Corners
        set_cell(local_rect.left(), local_rect.top(), border.top_left);
        set_cell(local_rect.right(), local_rect.top(), border.top_right);
        set_cell(local_rect.left(), local_rect.bottom(), border.bottom_left);
        set_cell(local_rect.right(), local_rect.bottom(), border.bottom_right);

        // Horizontal edges
        let left_offset = border.left.width() as u16;
        let right_bound = local_rect.right();
        for x in (local_rect.left() + left_offset)..right_bound {
            set_cell(x, local_rect.top(), border.top);
            set_cell(x, local_rect.bottom(), border.bottom);
        }

        // Vertical edges
        let top_offset = border.top.width() as u16;
        let bottom_bound = local_rect.bottom();
        for y in (local_rect.top() + top_offset)..bottom_bound {
            set_cell(local_rect.left(), y, border.left);
            set_cell(local_rect.right(), y, border.right);
        }

        self
    }

    fn text(&mut self, position: Point, str: impl AsRef<str>) -> usize {
        self.text_with(position, str, BufferDrawingOptions::default())
    }

    fn text_with(&mut self, position: Point, str: impl AsRef<str>, options: Self::Options) -> usize {
        let position = self.to_local(position);
        let style = options.style.unwrap_or(self.style);
        let clip = self.clip;

        // Drop early when the row is outside the clip.
        if position.y < clip.min.y || position.y >= clip.max.y {
            return 0;
        }

        let (left, right) = (clip.left().max(position.x), clip.right());

        if left >= right {
            return 0;
        }

        let mut col = position.x;
        let mut n = 0;

        for grapheme in UnicodeSegmentation::graphemes(str.as_ref(), true) {
            if grapheme.contains(char::is_control) {
                continue;
            }
            let width = grapheme.width() as u16;
            if width == 0 {
                continue;
            }

            if col + width > right {
                break;
            }

            if col >= left {
                self.buffer[pos!(position.y, col)]
                    .set_str_measured(grapheme, width as usize, self.arena)
                    .set_style(style);

                // Clear continuation cells for wide characters.
                for dx in 1..width {
                    self.buffer[pos!(position.y, col + dx)].set_continuation(self.arena);
                }
                n += width as usize;
            }
            col += width;
        }

        n
    }

    fn horizontal_line(&mut self, position: Point, length: u16) -> &mut Self {
        self.horizontal_line_with(position, length, BufferDrawingOptions::default())
    }

    fn horizontal_line_with(&mut self, position: Point, length: u16, options: Self::Options) -> &mut Self {
        let local_origin = self.to_local(position);
        let style = options.style.unwrap_or(self.style);
        let glyph = options.glyph.unwrap_or(self.glyph);

        let end = (
            local_origin.x.saturating_add(length),
            local_origin.y,
        );

        if self.clip.contains(&local_origin) && self.clip.contains(&end) {
            for offset in 0..length {
                self.buffer[pos!(local_origin.y, local_origin.x.saturating_add(offset))]
                    .set_style(style)
                    .set_char(glyph, self.arena);
            }
        }

        self
    }

    fn vertical_line(&mut self, position: Point, length: u16) -> &mut Self {
        self.vertical_line_with(position, length, BufferDrawingOptions::default())
    }

    fn vertical_line_with(&mut self, position: Point, length: u16, options: Self::Options) -> &mut Self {
        let local_origin = self.to_local(position);
        let style = options.style.unwrap_or(self.style);
        let glyph = options.glyph.unwrap_or(self.glyph);

        let end = (
            local_origin.x,
            local_origin.y.saturating_add(length),
        );

        if self.clip.contains(&local_origin) && self.clip.contains(&end) {
            for offset in 0..length {
                self.buffer[pos!(local_origin.y.saturating_add(offset), local_origin.x)]
                    .set_style(style)
                    .set_char(glyph, self.arena);
            }
        }

        self
    }

    fn clear(&mut self, rect: Rect) -> &mut Self {
        self.rect(rect)
    }

    fn save(&mut self) -> &mut Self {
        self.stacks.push(self.clone());
        self
    }

    fn restore(&mut self) -> &mut Self {
        if let Some(previous) = self.stacks.pop() {
            self.state = previous;
        }
        self
    }

    fn with(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        f(self);
        self.restore()
    }

    fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        self.translate(rect.min);
        self.clip(Rect::from(rect.size()));
        f(self);
        self.restore()
    }

    fn resize(&mut self, size: impl Into<Size>) -> &mut Self {
        let size = size.into();
        self.buffer.resize(size.width as usize, size.height as usize);
        self.clip = self.buffer.bounds();
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
    use geometry::pos;
    use crate::Grapheme;
    use tree::At;
    use crate::{Document, FlexDirection, FontWeight, Element, TextDecoration};
    use super::*;

    struct Context<'a> {
        buffer: Buffer,
        arena: Arena,
        document: Document<'a>,
    }

    fn add_content(context: &mut Context) {
        let document = &mut context.document;
        let root = document.root_mut();

        root.border = Border::Solid;
        root.margin = (2, 2).into();
        root.padding = (1, 1).into();

        let heading = document.insert_with(
            Element::Span(Cow::Borrowed("Title")),
            |node| {
                node.color = Some(Color::Red);
                node.text_decoration = Some(TextDecoration::Underline);
                node.font_weight = Some(FontWeight::Bold);
            },
        );

        let footer = document.insert_with(Element::Div(), |node| {
            node.background = Some(Color::BrightBlack);
            node.flex_direction = FlexDirection::Row;
        });

        let footer_left = document.insert_at_with(Element::Div(), At::Child(footer), |node| {
            node.padding = (1, 1).into();
        });
        let footer_left_content = document.insert_at(Element::Span("Gloss Rendering"), At::Child(footer_left));

        let footer_right = document.insert_at_with(Element::Div(), At::Child(footer), |node| {
            node.padding = (1, 1).into();
        });
        let footer_right_content = document.insert_at(Element::Span("Test Consortium"), At::Child(footer_right));

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

    fn renderer<'a>(context: &'a mut Context) -> BufferDrawingContext<'a> {
        BufferDrawingContext::new(&mut context.buffer, &mut context.arena)
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

        renderer.border = Border::Solid;
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

        assert_eq!(context.buffer[pos!(3, 3)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[pos!(0, 0)].grapheme(), Grapheme::inline('B'));
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

        assert_eq!(context.buffer[(1, 4)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(1, 5)].grapheme(), Grapheme::inline('i'));
        // Adjacent cell untouched
        assert_eq!(context.buffer[(1, 6)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_draw_text_clipped() {
        let mut context = context(10, 5);
        let mut renderer = renderer(&mut context);

        renderer.clip(Rect::from(Size::new(4, 5)));
        renderer.text(Point::new(0, 0), "Hello");

        // Only first 4 chars fit in clip
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(0, 3)].grapheme(), Grapheme::inline('l'));
        // 5th char ('o') is outside clip — cell stays empty
        assert_eq!(context.buffer[(4, 0)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_render_document_with_padding() {
        use crate::Space;

        let mut context = context(20, 10);
        let document = &mut context.document;

        // Root with padding — children should render inside the content area
        let root = document.root_mut();
        root.padding = (2, 2).into();
        root.flex_direction = FlexDirection::Column;

        let child = document.insert_with(
            Element::Span(Cow::Borrowed("AB")),
            |node| { node.color = Some(Color::Blue); },
        );

        document.compute_layout(Space::new(20u32, 10u32));

        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        // Text should appear at content area offset (padding=2 on each side)
        let child_content = document.content_bounds(child);
        let text_x = child_content.min.x as usize;
        let text_y = child_content.min.y as usize;
        assert_eq!(context.buffer[(text_y, text_x)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(text_y, text_x + 1)].grapheme(), Grapheme::inline('B'));
        // Origin cell should be empty (it's in the padding)
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::SPACE);
    }

    #[test]
    fn test_render_nested_nodes() {
        use crate::Space;

        let mut context = context(30, 15);
        let document = &mut context.document;

        // Root with padding, column layout
        let root = document.root_mut();
        root.padding = (1, 1).into();
        root.flex_direction = FlexDirection::Column;

        // Child div with its own padding
        let child_div = document.insert_with(Element::Div(), |node| {
            node.padding = (1, 1).into();
            node.flex_direction = FlexDirection::Column;
        });

        // Grandchild text inside the child div
        let text_id = document.insert_at_with(
            Element::Span("OK"),
            At::Child(child_div),
            |node| { node.color = Some(Color::Blue); },
        );

        document.compute_layout(Space::new(30u32, 15u32));

        let text_content = document.content_bounds(text_id);

        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        let div_bounds = document.border_bounds(child_div);
        let text_bounds = document.border_bounds(text_id);

        // Absolute position = parent bounds + child bounds (taffy locations are parent-relative)
        let tx = (div_bounds.min.x + text_bounds.min.x) as usize;
        let ty = (div_bounds.min.y + text_bounds.min.y) as usize;
        assert_eq!(context.buffer[(ty, tx)].grapheme(), Grapheme::inline('O'));
        assert_eq!(context.buffer[(ty, tx + 1)].grapheme(), Grapheme::inline('K'));
        // Padding area should be empty
        assert_eq!(context.buffer[pos!(0, 0)].grapheme(), Grapheme::SPACE);


    }

    #[test]
    fn test_render_stacked_children() {
        use crate::Space;

        let mut context = context(30, 15);
        let document = &mut context.document;

        let root = document.root_mut();
        root.flex_direction = FlexDirection::Column;

        // Two stacked children in column layout
        let child_a = document.insert_with(
            Element::Span(Cow::Borrowed("AA")),
            |node| { node.color = Some(Color::Blue); },
        );
        let child_b = document.insert_with(
            Element::Span(Cow::Borrowed("BB")),
            |node| { node.color = Some(Color::Green); },
        );

        document.compute_layout(Space::new(30u32, 15u32));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        // First child
        assert_eq!(context.buffer[(a_bounds.min.y as usize, a_bounds.min.x as usize)].grapheme(), Grapheme::inline('A'));
        // Second child should be below the first
        assert!(b_bounds.min.y > a_bounds.min.y, "B should be below A: A.y={}, B.y={}", a_bounds.min.y, b_bounds.min.y);
        assert_eq!(context.buffer[(b_bounds.min.y as usize, b_bounds.min.x as usize)].grapheme(), Grapheme::inline('B'));
    }

    /// Padding should be part of the element's background (CSS
    /// `background-clip: border-box`). Regression for a bug where the
    /// background only filled the content-box and left the padding area
    /// transparent.
    #[test]
    fn test_render_background_fills_padding() {
        use crate::Space;

        let mut context = context(10, 5);
        let document = &mut context.document;

        let root = document.root_mut();
        root.background = Some(Color::Red);
        root.padding = (1, 1).into();

        document.compute_layout(Space::new(10u32, 5u32));
        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        let h = context.buffer.bounds().height() as usize;
        let w = context.buffer.bounds().width() as usize;
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    context.buffer[(y, x)].style.background,
                    Color::Red,
                    "cell ({y},{x}) should be red, padding included",
                );
            }
        }
    }

    /// The bordered div's border characters should be drawn. Regression for
    /// a bug where `paint_node` called `ctx.border(..)` before setting the
    /// node's border style on the context, so the draw used the parent's
    /// style (`Border::None`) and produced no output.
    #[test]
    fn test_render_node_border_is_drawn() {
        use crate::Space;

        let mut context = context(10, 4);
        let document = &mut context.document;

        document.insert_with(Element::Div(), |node| {
            node.border = Border::Bold;
            node.size = crate::Size::new(10u32, 4u32);
        });

        document.compute_layout(Space::new(10u32, 4u32));
        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        // Bold border: top-left ┏, top ━, top-right ┓, etc.
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('┏'));
        assert_eq!(context.buffer[(0, 9)].grapheme(), Grapheme::inline('┓'));
        assert_eq!(context.buffer[(3, 0)].grapheme(), Grapheme::inline('┗'));
        assert_eq!(context.buffer[(3, 9)].grapheme(), Grapheme::inline('┛'));
        // Top and bottom horizontal edges
        for x in 1..9 {
            assert_eq!(context.buffer[(0, x)].grapheme(), Grapheme::inline('━'));
            assert_eq!(context.buffer[(3, x)].grapheme(), Grapheme::inline('━'));
        }
    }

    /// `background: Some(Color::None)` means "no fill" and must not clear
    /// the parent's backdrop. Regression for a bug where any `Some(_)`
    /// background triggered a `rect` fill, including the no-color sentinel.
    #[test]
    fn test_child_none_background_preserves_parent() {
        use crate::Space;

        let mut context = context(10, 3);
        let document = &mut context.document;

        let root = document.root_mut();
        root.background = Some(Color::Red);

        document.insert_with(Element::Span(Cow::Borrowed("X")), |node| {
            node.background = Some(Color::None);
        });

        document.compute_layout(Space::new(10u32, 3u32));
        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        // Every cell that isn't the glyph itself should still carry the
        // root's red backdrop — the span must not overwrite it.
        let h = context.buffer.bounds().height() as usize;
        let w = context.buffer.bounds().width() as usize;
        for y in 0..h {
            for x in 0..w {
                let cell = &context.buffer[(y, x)];
                if cell.grapheme() == Grapheme::inline('X') { continue; }
                assert_eq!(
                    cell.style.background,
                    Color::Red,
                    "cell ({y},{x}) lost root's red backdrop",
                );
            }
        }
    }

    /// End-to-end coverage of the `crates/gloss/src/bin/main.rs` scene:
    /// root with padding + red background, transparent text element, a
    /// bordered container with three colored children. Asserts the
    /// characteristics the crate binary renders.
    #[test]
    fn test_render_main_scene() {
        use crate::Space;

        let mut context = context(20, 20);
        let document = &mut context.document;

        let root = document.root_mut();
        root.background = Some(Color::Red);
        root.color = Some(Color::White);
        root.padding = (1, 1).into();

        document.insert_with(
            Element::Span(Cow::Borrowed("Hello")),
            |node| {
                node.background = Some(Color::None);
                node.font_weight = Some(FontWeight::Bold);
            },
        );

        let abc = document.insert_with(Element::Div(), |node| {
            node.border = Border::Bold;
        });

        let a = document.insert_at_with(Element::Div(), At::Child(abc), |node| {
            node.background = Some(Color::Green);
        });
        let b = document.insert_at_with(Element::Div(), At::Child(abc), |node| {
            node.background = Some(Color::Yellow);
        });
        let c = document.insert_at_with(Element::Div(), At::Child(abc), |node| {
            node.background = Some(Color::Blue);
        });
        document.insert_at(Element::Span("A"), At::Child(a));
        document.insert_at(Element::Span("B"), At::Child(b));
        document.insert_at(Element::Span("C"), At::Child(c));

        document.compute_layout(Space::new(20u32, 20u32));
        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        let w = context.buffer.bounds().width() as usize;
        let h = context.buffer.bounds().height() as usize;

        // Root's padding row (top/bottom) is filled with red.
        for x in 0..w {
            assert_eq!(
                context.buffer[(0, x)].style.background, Color::Red,
                "top padding row should be red at col {x}",
            );
            assert_eq!(
                context.buffer[(h - 1, x)].style.background, Color::Red,
                "bottom padding row should be red at col {x}",
            );
        }

        // Root's padding column (left/right) is filled with red.
        for y in 0..h {
            assert_eq!(
                context.buffer[(y, 0)].style.background, Color::Red,
                "left padding col should be red at row {y}",
            );
            assert_eq!(
                context.buffer[(y, w - 1)].style.background, Color::Red,
                "right padding col should be red at row {y}",
            );
        }

        // "Hello" text lands at col 1 (after padding) on row 1.
        assert_eq!(context.buffer[(1, 1)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(1, 5)].grapheme(), Grapheme::inline('o'));

        // abc's border is drawn (bold box) — top/bottom corners.
        let abc_bounds = document.border_bounds(abc);
        let top = abc_bounds.min.y as usize;
        let bottom = abc_bounds.max.y as usize - 1;
        let left = abc_bounds.min.x as usize;
        let right = abc_bounds.max.x as usize - 1;
        assert_eq!(context.buffer[(top, left)].grapheme(), Grapheme::inline('┏'));
        assert_eq!(context.buffer[(top, right)].grapheme(), Grapheme::inline('┓'));
        assert_eq!(context.buffer[(bottom, left)].grapheme(), Grapheme::inline('┗'));
        assert_eq!(context.buffer[(bottom, right)].grapheme(), Grapheme::inline('┛'));
    }

    #[test]
    fn test_render_row_children() {
        use crate::Space;

        let mut context = context(30, 5);
        let document = &mut context.document;

        let root = document.root_mut();
        root.display = crate::Display::Flex;
        root.flex_direction = FlexDirection::Row;

        let child_a = document.insert_with(
            Element::Span(Cow::Borrowed("L")),
            |node| { node.color = Some(Color::Blue); },
        );
        let child_b = document.insert_with(
            Element::Span(Cow::Borrowed("R")),
            |node| { node.color = Some(Color::Green); },
        );

        document.compute_layout(Space::new(30u32, 5u32));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        BufferDrawingContext::new(&mut context.buffer, &mut context.arena).paint(&document);

        // Side by side in row layout
        assert_eq!(context.buffer[(a_bounds.min.y as usize, a_bounds.min.x as usize)].grapheme(), Grapheme::inline('L'));
        assert!(b_bounds.min.x > a_bounds.min.x, "R should be right of L");
        assert_eq!(context.buffer[(b_bounds.min.y as usize, b_bounds.min.x as usize)].grapheme(), Grapheme::inline('R'));
    }
}