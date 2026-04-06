use std::io;
use unicode_segmentation::UnicodeSegmentation;
use geometry::{Bounded, Contains, Intersect, Outer, Point, Ranges, Rect, Edges, Sides, Size, Translate, Resolve};
use crate::{Buffer, Arena};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::{Border, Backend, Renderer};
use crate::symbols::Symbol;
use ansi::Style;

/// Snapshot of all context state, pushed/popped via save/restore.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Context {
    rect: Rect,
    origin: Point,
    fill_style: Style,
    fill_char: char,
    stroke_type: Border,
}

/// 2D drawing context for terminal buffers.
///
/// Modeled after HTML Canvas — mutable "current state" with a save/restore
/// stack. All coordinates are relative to `origin`; all draws are clipped
/// to the current clip rect.
pub struct BufferBackend<'a> {
    buffer: &'a mut Buffer,
    arena: &'a mut Arena,
    context: Context,
    stacks: Vec<Context>,
}

impl<'buf> BufferBackend<'buf> {
    /// Create a new context spanning the full buffer.
    pub fn new(buffer: &'buf mut Buffer, arena: &'buf mut Arena) -> Self {
        let clip = buffer.bounds(); // full buffer rect
        Self {
            buffer,
            arena,
            context: Context {
                rect: clip,
                origin: Point::ZERO,
                fill_style: Style::None,
                fill_char: ' ',
                stroke_type: Border::None,
            },
            stacks: Vec::new(),
        }
    }

    pub fn fill_style(&mut self, style: Style) -> &mut Self {
        self.context.fill_style = style;
        self
    }

    pub fn fill_char(&mut self, char: char) -> &mut Self {
        self.context.fill_char = char;
        self
    }

    pub fn stroke_type(&mut self, border: Border) -> &mut Self {
        self.context.stroke_type = border;
        self
    }

    pub fn local<T: Translate<Point>>(&self, rect: T) -> T::Output {
        rect.translate(&self.context.origin)
    }

    pub fn clip(&mut self, rect: Rect) -> &mut Self {
        self.context.rect = self.context.rect.intersect(&self.local(rect));
        self
    }

    /// Shift the origin by `offset`. Cumulative within a save/restore frame.
    pub fn translate(&mut self, offset: Point) -> &mut Self {
        self.context.origin = self.context.origin + offset;
        self
    }

    fn intersect(&self, rect: Rect) -> Option<Rect> {
        let result = self.context.rect
            .intersect(&rect);

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn resolve_rect(&self, rect: impl Into<Option<Rect>>) -> Rect {
        rect.into().map(|r| self.local(r)).unwrap_or(self.context.rect)
    }

    fn resolve_position(&self, pos: impl Into<Option<Point>>) -> Point {
        pos.into().map(|p| self.local(p)).unwrap_or(Point::ZERO)
    }

    fn resolve_fill_style(&self, style: impl Into<Option<Style>>) -> Style {
        style.into().unwrap_or(self.context.fill_style)
    }

    fn resolve_stroke_type(&self, border: impl Into<Option<Border>>) -> Border {
        border.into().unwrap_or(self.context.stroke_type)
    }

    fn resolve_fill_char(&self, char: impl Into<Option<char>>) -> char {
        char.into().unwrap_or(self.context.fill_char)
    }

    pub fn fill(&mut self, bounds: impl Into<Option<Rect>>, fill_style: impl Into<Option<Style>>, fill_char: impl Into<Option<char>>) -> &mut Self {
        let rect = self.resolve_rect(bounds);
        let fill_style = self.resolve_fill_style(fill_style);
        let fill_char = self.resolve_fill_char(fill_char);

        if let Some(r) = self.intersect(rect) {
            for pos in &r {
                let index: usize = self.buffer.bounds().resolve(pos);
                self.buffer[index].set_char(fill_char, self.arena).set_style(fill_style);
            }

        }
        self
    }

    pub fn stroke(&mut self, rect: impl Into<Option<Rect>>, stroke_type: impl Into<Option<Border>>) -> &mut Self {
        let mut rect = self.resolve_rect(rect);
        let stroke_type = self.resolve_stroke_type(stroke_type);
        let border = stroke_type.into_border();

        rect.max.x -= border.right.width();
        rect.max.y -= border.bottom.width();

        if rect.is_empty() {
            return self;
        }

        // We clip each cell individually so partial borders work.
        let mut set = |x: usize, y: usize, border: Symbol| {
            if self.context.rect.contains(&(x, y)) {
                self.buffer[(x, y)].set_char_measured(border.symbol(), border.width(), self.arena);
            }
        };

        // corners
        set(rect.left(), rect.top(), border.top_left);
        set(rect.right(), rect.top(), border.top_right);
        set(rect.left(), rect.bottom(), border.bottom_left);
        set(rect.right(), rect.bottom(), border.bottom_right);

        // horizontal edges
        for x in (rect.left() + border.left.width())..rect.right() {
            set(x, rect.top(), border.top);
            set(x, rect.bottom(), border.bottom);
        }

        // vertical edges
        for y in (rect.top() + border.top.width())..rect.bottom() {
            set(rect.left(), y, border.left);
            set(rect.right(), y, border.right);
        }

        self
    }

    pub fn text(&mut self, position: impl Into<Option<Point>>, fill_style: impl Into<Option<Style>>, str: impl AsRef<str>) -> usize {
        let position = self.resolve_position(position);
        let style = self.resolve_fill_style(fill_style);

        let y = position.y;
        let mut i = 0;

        for (grapheme, width) in str.as_ref().graphemes(true)
            .map(|g| (g, g.width())) {
            let x = position.x + i; // or your coord type

            // Stop if we've gone past clip right edge
            if x + width > self.context.rect.right() {
                break;
            }

            if self.context.rect.contains(&(x, y))  {
                self.buffer[(x, y)].set_str_measured(grapheme, width, self.arena);
                // For wide chars, mark continuation cell(s)
                for i in 1..width {
                    let cont = (x + i, y);
                    if self.context.rect.contains(&cont) {
                        self.buffer[cont].set_continuation(self.arena).set_style(style);
                    }
                }
            }

            i += width;
        }

        i
    }

    pub fn clear(&mut self, bounds: impl Into<Option<Rect>>) -> &mut Self {
        self.fill(bounds, self.context.fill_style, self.context.fill_char)
    }

    pub fn save(&mut self) -> &mut Self {
        self.stacks.push(self.context.clone());
        self
    }

    pub fn restore(&mut self) -> &mut Self {
        if let Some(prev) = self.stacks.pop() {
            self.context = prev;
        }
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.context = Context::default();
        self
    }

    pub fn with(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        f(self);
        self.restore();
        self
    }

    pub fn within(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        self.translate(rect.min);
        self.clip(Rect::from(rect.size()));
        f(self);
        self.restore();
        self
    }

}

impl<'a> Renderer<BufferBackend<'a>> {
    pub fn new(buffer: &'a mut Buffer, arena: &'a mut Arena) -> Self {
        Self(BufferBackend::new(buffer, arena))
    }
}

impl<'a> Backend for BufferBackend<'a> {
    type Error = io::Error;

    fn fill_style(&mut self, style: crate::Style) {
        self.fill_style(style.into());
    }

    fn fill_char(&mut self, fill: char) {
        self.fill_char(fill);
    }

    fn stroke_type(&mut self, border: Border) {
        self.stroke_type(border);
    }

    fn clip(&mut self, bounds: Rect) -> Result<(), Self::Error> {
        self.clip(bounds);
        Ok(())
    }

    fn translate(&mut self, offset: Point) -> Result<(), Self::Error> {
        self.translate(offset);
        Ok(())
    }

    fn fill(&mut self, bounds: impl Into<Option<Rect>>, fill_style: impl Into<Option<crate::Style>>, fill_char: impl Into<Option<char>>) {
        self.fill(bounds, fill_style.into().map(|s| s.into()), fill_char);
    }

    fn stroke(&mut self, bounds: impl Into<Option<Rect>>, stroke_type: impl Into<Option<Border>>) {
        self.stroke(bounds, stroke_type);
    }

    fn text(&mut self, position: impl Into<Option<Point>>, fill_style: impl Into<Option<crate::Style>>, str: impl AsRef<str>) {
        self.text(position, fill_style.into().map(|s| s.into()), str);
    }

    fn save(&mut self) -> Result<(), Self::Error> {
        self.save();
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        self.restore();
        Ok(())
    }

    fn resize(&mut self, size: Size) -> Result<(), Self::Error> {
        self.buffer.resize(size.width, size.height);
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::ops::{Add, Sub};
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

        root.set_border(Border::Solid);
        root.set_margin((2, 2));
        root.set_padding((1, 1));

        let heading = document.insert_with(
            Node::Span(Cow::Borrowed("Title")),
            |node| {
                node.set_color(Color::Red);
                node.set_text_decoration(TextDecoration::Underline);
                node.set_font_weight(FontWeight::Bold);
            },
        );

        let footer = document.insert_with(Node::Div(), |node| {
            node.set_background(Color::BrightBlack);
            node.set_flex_direction(FlexDirection::Row);
        });

        let footer_left = document.insert_at_with(Node::Div(), At::Child(footer), |node| {
            node.set_padding((1, 1));
        });
        let footer_left_content = document.insert_at(Node::Span("Gloss Rendering"), At::Child(footer_left));

        let footer_right = document.insert_at_with(Node::Div(), At::Child(footer), |node| {
            node.set_padding((1, 1));
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

    fn renderer<'a>(context: &'a mut Context) -> Renderer<BufferBackend<'a>> {
        BufferBackend::new(&mut context.buffer, &mut context.arena).into_renderer()
    }

    #[test]
    fn test_basic_fill() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.fill(None, Some(Style::default().foreground(Color::White)), Some('x'));

        assert_eq!(context.buffer.iter().all(|c| c.style.foreground == Color::White && c.grapheme() == Grapheme::inline('x')), true);
    }


    #[test]
    fn test_basic_stroke() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.stroke(None, Some(Border::Solid));

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
        renderer.text(Some(Point::ZERO), None, "A");
        renderer.restore();

        // After restore, origin is back to (0,0)
        renderer.text(Some(Point::ZERO), None, "B");

        assert_eq!(context.buffer[(3, 3)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('B'));
    }

    #[test]
    fn test_save_restore_clip() {
        let mut context = context(10, 10);

        {
            let mut renderer = renderer(&mut context);
            renderer.save();
            renderer.clip(Rect::from(Size::new(5, 5)));
            renderer.fill(None, None, Some('X'));
            renderer.restore();
        }

        // Inside the old clip — should be filled
        assert_eq!(context.buffer[(2, 2)].grapheme(), Grapheme::inline('X'));
        // Outside the old clip — should be empty
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::SPACE);

        {
            // After restore, full clip is back — can write outside
            let mut renderer = renderer(&mut context);
            renderer.text(Some(Point::new(7, 7)), None, "Y");
        }
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::inline('Y'));
    }

    #[test]
    fn test_within_scoped_translate_and_clip() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.within(Rect::bounds(Point::new(2, 2), Point::new(6, 6)), |r| {
            r.fill(None, None, Some('W'));
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
        renderer.text(Some(Point::ZERO), None, "N");
        renderer.restore();
        renderer.restore();

        // (3+2, 3+2) = (5, 5)
        assert_eq!(context.buffer[(5, 5)].grapheme(), Grapheme::inline('N'));
    }

    #[test]
    fn test_draw_text_position() {
        let mut context = context(20, 5);
        let mut renderer = renderer(&mut context);

        renderer.text(Some(Point::new(4, 1)), None, "Hi");

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
        renderer.text(Some(Point::new(0, 0)), None, "Hello");

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
        root.set_padding((2, 2));
        root.set_flex_direction(FlexDirection::Column);

        let child = document.insert_with(
            Node::Span(Cow::Borrowed("AB")),
            |node| { node.set_color(Color::Blue); },
        );

        document.compute_layout(Space::new(20u32, 10u32));

        let mut renderer = BufferBackend::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document).unwrap();

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
        root.set_padding((1, 1));
        root.set_flex_direction(FlexDirection::Column);

        // Child div with its own padding
        let child_div = document.insert_with(Node::Div(), |node| {
            node.set_padding((1, 1));
            node.set_flex_direction(FlexDirection::Column);
        });

        // Grandchild text inside the child div
        let text_id = document.insert_at_with(
            Node::Span("OK"),
            At::Child(child_div),
            |node| { node.set_color(Color::Blue); },
        );

        document.compute_layout(Space::new(30u32, 15u32));

        let text_content = document.content_bounds(text_id);

        let mut renderer = BufferBackend::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document).unwrap();

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
        root.set_flex_direction(FlexDirection::Column);

        // Two stacked children in column layout
        let child_a = document.insert_with(
            Node::Span(Cow::Borrowed("AA")),
            |node| { node.set_color(Color::Blue); },
        );
        let child_b = document.insert_with(
            Node::Span(Cow::Borrowed("BB")),
            |node| { node.set_color(Color::Green); },
        );

        document.compute_layout(Space::new(30u32, 15u32));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        let mut renderer = BufferBackend::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document).unwrap();

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
        root.set_flex_direction(FlexDirection::Row);

        let child_a = document.insert_with(
            Node::Span(Cow::Borrowed("L")),
            |node| { node.set_color(Color::Blue); },
        );
        let child_b = document.insert_with(
            Node::Span(Cow::Borrowed("R")),
            |node| { node.set_color(Color::Green); },
        );

        document.compute_layout(Space::new(30u32, 5u32));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        let mut renderer = BufferBackend::new(&mut context.buffer, &mut context.arena).into_renderer();
        renderer.render(&document).unwrap();

        // Side by side in row layout
        assert_eq!(context.buffer[(a_bounds.min.x, a_bounds.min.y)].grapheme(), Grapheme::inline('L'));
        assert!(b_bounds.min.x > a_bounds.min.x, "R should be right of L");
        assert_eq!(context.buffer[(b_bounds.min.x, b_bounds.min.y)].grapheme(), Grapheme::inline('R'));
    }
}