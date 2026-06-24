use crate::{Arena, Border, Buffer, DrawingContext, DrawingOptions};
use ansi::Style;
use geometry::{Bound, Intersect, Outer, Point, Rect, Size, Translate};
use smallvec::SmallVec;
use std::io;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Snapshot of all context state, pushed/popped via save/restore.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    clip: Rect,
    origin: Point,
    style: Style,
    border: Border,
    glyph: char,
}

#[derive(Debug, Clone, Copy)]
struct Options {
    style: Style,
    glyph: char,
    glyph_width: usize,
    border: Border,
}

/// 2D drawing context for terminal buffers.
///
/// Modeled after HTML Canvas — mutable "current state" with a save/restore
/// stack. All coordinates are relative to `origin`; all draws are clipped
/// to the current clip rect.
#[derive(Debug)]
pub struct BufferPainter<'a> {
    buffer: &'a mut Buffer,
    arena: &'a mut Arena,
    state: State,
    stacks: SmallVec<State, 16>,
}

impl<'buf> BufferPainter<'buf> {
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

    #[inline]
    fn resolve(&self, options: DrawingOptions) -> Options {
        let glyph = options.glyph.unwrap_or(self.state.glyph);

        Options {
            style: options.layout.map_or(self.state.style, Into::into),
            glyph,
            glyph_width: glyph.width().unwrap_or(0),
            border: options.border.map_or(self.state.border, Into::into),
        }
    }

    fn to_local<T: Translate<Point>>(&self, value: T) -> T::Output {
        value.translate(&self.state.origin)
    }

    fn intersect<T: Bound, O: Bound>(&self, rect: T) -> Option<O> where Rect: Intersect<T, Output = O> {
        let result = self.state.clip.intersect(&rect);
        (!result.is_empty()).then_some(result)
    }
}

impl<'a> DrawingContext for BufferPainter<'a> {
    type Error = io::Error;

    fn current_clip(&self) -> Rect {
        self.state.clip
    }

    fn current_style(&self) -> crate::Layout {
        self.state.style.into()
    }

    fn current_glyph(&self) -> char {
        self.state.glyph
    }

    fn current_border_style(&self) -> Border {
        self.state.border
    }

    fn style(&mut self, style: crate::Layout) -> &mut Self {
        self.state.style = style.into();
        self
    }

    fn glyph(&mut self, glyph: char) -> &mut Self {
        self.state.glyph = glyph;
        self
    }

    fn border_style(&mut self, border: Border) -> &mut Self {
        self.state.border = border;
        self
    }

    /// Intersect the current clip region with `rect`.
    ///
    /// The input is in local coordinates and will be transformed before
    /// intersection.
    fn clip(&mut self, rect: Rect) -> &mut Self {
        self.state.clip = self.state.clip.intersect(&self.to_local(rect));
        self
    }

    /// Shift the origin by `offset`. Cumulative within a save/restore frame.
    fn translate(&mut self, offset: Point) -> &mut Self {
        self.state.origin += offset;
        self
    }

    fn rect_with(&mut self, rect: Rect, options: DrawingOptions) -> Result<&mut Self, Self::Error> {
        let rect = self.to_local(rect);
        let options = self.resolve(options);

        if options.glyph_width == 0 {
            return Ok(self);
        }

        if let Some(clipped) = self.intersect(rect) {
            let mut encoded = [0; 4];
            let grapheme = options.glyph.encode_utf8(&mut encoded);
            let width = options.glyph_width as u16;

            for y in clipped.top()..clipped.bottom() {
                let mut x = clipped.left();

                while x.saturating_add(width) <= clipped.right() {
                    self.buffer.set_grapheme_styled(
                        Point::new(x, y),
                        grapheme,
                        options.glyph_width,
                        options.style,
                        self.arena,
                    );
                    x = x.saturating_add(width);
                }
            }
        }

        Ok(self)
    }

    fn text_with(
        &mut self,
        position: Point,
        text: &str,
        options: DrawingOptions,
    ) -> Result<usize, Self::Error> {
        let position = self.to_local(position);
        let options = self.resolve(options);
        let clip = self.state.clip;

        if position.y < clip.min.y || position.y >= clip.max.y {
            return Ok(0);
        }

        let mut col = position.x;
        let mut written = 0;

        for grapheme in UnicodeSegmentation::graphemes(text, true) {
            if grapheme.contains(char::is_control) {
                continue;
            }

            let width = grapheme.width() as u16;

            if width == 0 {
                continue;
            }

            let next = col.saturating_add(width);

            if next > clip.right() {
                break;
            }

            if col >= clip.left() {
                self.buffer.set_grapheme_styled(
                    Point::new(col, position.y),
                    grapheme,
                    width as usize,
                    options.style,
                    self.arena,
                );
                written += width as usize;
            }

            col = next;
        }

        Ok(written)
    }

    fn char_with(
        &mut self,
        position: Point,
        ch: char,
        options: DrawingOptions,
    ) -> Result<usize, Self::Error> {
        if ch.is_control() {
            return Ok(0);
        }

        let position = self.to_local(position);
        let options = self.resolve(options);
        let width = ch.width().unwrap_or(0);

        if width == 0 {
            return Ok(0);
        }

        let bounds = Rect::new(position.x, position.y, width as u16, 1);

        if self.state.clip.intersect(&bounds) != bounds {
            return Ok(0);
        }

        let mut encoded = [0; 4];
        self.buffer.set_grapheme_styled(
            position,
            ch.encode_utf8(&mut encoded),
            width,
            options.style,
            self.arena,
        );

        Ok(width)
    }

    fn save(&mut self) -> &mut Self {
        self.stacks.push(self.state.clone());
        self
    }

    fn restore(&mut self) -> &mut Self {
        if let Some(previous) = self.stacks.pop() {
            self.state = previous;
        }
        self
    }

    fn resize(&mut self, size: Size) -> Result<&mut Self, Self::Error> {
        self.buffer
            .resize(size.width as usize, size.height as usize);
        self.state.clip = self.buffer.bounds();
        Ok(self)
    }

    fn finish(&mut self) -> Result<&mut Self, Self::Error> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Document, DrawingContextExtension, Element, FlexDirection, FontWeight, Layout,
    };
    use crate::{Grapheme, Layouted};
    use ansi::Color;
    use geometry::{Edges, pos};
    use std::borrow::Cow;
    use std::ops::Sub;
    use tree::At;

    struct Context<'a> {
        buffer: Buffer,
        arena: Arena,
        document: Document<'a>,
    }

    fn add_content(context: &mut Context) {
        let document = &mut context.document;
        let root = document.root_element_mut();

        root.border = Border::Solid;
        root.margin = (2, 2).into();
        root.padding = (1, 1).into();

        let heading = document.insert(
            Element::Span(Cow::Borrowed("Title"))
                .color(Color::Red)
                .underline()
                .bold(),
        );

        let footer = document.insert(Element::Div().background(Color::BrightBlack).flex_row());

        let footer_left = document.insert_at(Element::Div().padding(1), At::Child(footer));
        let footer_left_content =
            document.insert_at(Element::Span("ui Rendering"), At::Child(footer_left));

        let footer_right = document.insert_at(Element::Div().padding(1), At::Child(footer));
        let footer_right_content =
            document.insert_at(Element::Span("Test Consortium"), At::Child(footer_right));
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

    fn renderer<'a>(context: &'a mut Context) -> BufferPainter<'a> {
        BufferPainter::new(&mut context.buffer, &mut context.arena)
    }

    #[test]
    fn test_basic_fill() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.state.style = Style::default().foreground(Color::White);
        renderer.state.glyph = 'x';
        renderer.rect(Rect::new(0, 0, 10, 10)).unwrap();

        assert_eq!(
            context.buffer.iter().all(
                |c| c.style.foreground == Color::White && c.grapheme() == Grapheme::inline('x')
            ),
            true
        );
    }

    #[test]
    fn test_basic_stroke() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.state.border = Border::Solid;
        renderer.border(Rect::new(0, 0, 10, 10)).unwrap();

        assert_eq!(
            context
                .buffer
                .iter_row(0)
                .all(|c| c.grapheme() != Grapheme::EMPTY),
            true
        );
        assert_eq!(
            context
                .buffer
                .iter_col(0)
                .all(|c| c.grapheme() != Grapheme::EMPTY),
            true
        );
        assert_eq!(
            context
                .buffer
                .iter_col(9)
                .all(|c| c.grapheme() != Grapheme::EMPTY),
            true
        );
        assert_eq!(
            context
                .buffer
                .iter_row(9)
                .all(|c| c.grapheme() != Grapheme::EMPTY),
            true
        );
        context
            .buffer
            .iter_rect(&context.buffer.bounds().sub(Edges::all(1)))
            .for_each(|c| assert_eq!(c.grapheme(), Grapheme::EMPTY));
    }

    #[test]
    fn test_save_restore_origin() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.save();
        renderer.translate(Point::new(3, 3));
        renderer.text(Point::ZERO, "A").unwrap();
        renderer.restore();

        // After restore, origin is back to (0,0)
        renderer.text(Point::ZERO, "B").unwrap();

        assert_eq!(context.buffer[pos!(3, 3)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[pos!(0, 0)].grapheme(), Grapheme::inline('B'));
    }

    #[test]
    fn test_save_restore_clip() {
        let mut context = context(10, 10);

        {
            let mut renderer = renderer(&mut context);
            renderer.state.glyph = 'X';
            renderer.save();
            renderer.clip(Rect::from(Size::new(5, 5)));
            renderer.rect(Rect::from(Size::new(15, 5))).unwrap();
            renderer.restore();
        }

        // Inside the old clip — should be filled
        assert_eq!(context.buffer[(2, 2)].grapheme(), Grapheme::inline('X'));
        // Outside the old clip — should be empty
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::EMPTY);

        {
            // After restore, full clip is back — can write outside
            let mut renderer = renderer(&mut context);
            renderer.text(Point::new(7, 7), "Y").unwrap();
        }
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::inline('Y'));
    }

    #[test]
    fn test_within_scoped_translate_and_clip() {
        let mut context = context(10, 10);
        let mut renderer = renderer(&mut context);

        renderer.state.glyph = 'W';
        renderer.within(Rect::bounds(Point::new(2, 2), Point::new(6, 6)), |r| {
            r.rect(Rect::new(0, 0, 5, 5)).unwrap();
        });

        // Inside the within rect — filled
        assert_eq!(context.buffer[(3, 3)].grapheme(), Grapheme::inline('W'));
        // Outside — empty
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::EMPTY);
        assert_eq!(context.buffer[(7, 7)].grapheme(), Grapheme::EMPTY);
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
        renderer.text(Point::ZERO, "N").unwrap();
        renderer.restore();
        renderer.restore();

        // (3+2, 3+2) = (5, 5)
        assert_eq!(context.buffer[(5, 5)].grapheme(), Grapheme::inline('N'));
    }

    #[test]
    fn test_draw_text_position() {
        let mut context = context(20, 5);
        let mut renderer = renderer(&mut context);

        renderer.text(Point::new(4, 1), "Hi").unwrap();

        assert_eq!(context.buffer[(1, 4)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(1, 5)].grapheme(), Grapheme::inline('i'));
        // Adjacent cell untouched
        assert_eq!(context.buffer[(1, 6)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn test_draw_text_clipped() {
        let mut context = context(10, 5);
        let mut renderer = renderer(&mut context);

        renderer.clip(Rect::from(Size::new(4, 5)));
        renderer.text(Point::new(0, 0), "Hello").unwrap();

        // Only first 4 chars fit in clip
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('H'));
        assert_eq!(context.buffer[(0, 3)].grapheme(), Grapheme::inline('l'));
        // 5th char ('o') is outside clip — cell stays empty
        assert_eq!(context.buffer[(4, 0)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn text_returns_display_cells_and_stores_wide_continuations() {
        let mut context = context(6, 1);

        let written = renderer(&mut context).text(Point::ZERO, "A中B").unwrap();

        assert_eq!(written, 4);
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(0, 1)].grapheme(), Grapheme::inline('中'));
        assert!(context.buffer[(0, 2)].is_continuation());
        assert_eq!(context.buffer[(0, 3)].grapheme(), Grapheme::inline('B'));
    }

    #[test]
    fn text_only_writes_wide_graphemes_that_fully_fit() {
        let mut context = context(4, 1);
        let mut renderer = renderer(&mut context);
        renderer.clip(Rect::new(0, 0, 2, 1));

        let written = renderer.text(Point::ZERO, "A中").unwrap();

        assert_eq!(written, 1);
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(0, 1)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn char_respects_display_width_and_clip() {
        let mut context = context(4, 1);
        let mut renderer = renderer(&mut context);

        assert_eq!(renderer.char(Point::ZERO, '中').unwrap(), 2);
        renderer.clip(Rect::new(0, 0, 3, 1));
        assert_eq!(renderer.char(Point::new(2, 0), '中').unwrap(), 0);

        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('中'));
        assert!(context.buffer[(0, 1)].is_continuation());
        assert_eq!(context.buffer[(0, 2)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn text_ignores_controls_and_char_ignores_zero_width_scalars() {
        let mut context = context(4, 1);
        let mut renderer = renderer(&mut context);

        assert_eq!(renderer.text(Point::ZERO, "A\nB").unwrap(), 2);
        assert_eq!(renderer.char(Point::new(2, 0), '\u{0301}').unwrap(), 0);

        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('A'));
        assert_eq!(context.buffer[(0, 1)].grapheme(), Grapheme::inline('B'));
        assert_eq!(context.buffer[(0, 2)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn clear_does_not_depend_on_current_glyph_or_style() {
        let mut context = context(3, 1);
        let mut renderer = renderer(&mut context);

        renderer
            .style(Layout {
                color: Some(Color::Red),
                ..Layout::DEFAULT
            })
            .glyph('x');
        renderer.rect(Rect::new(0, 0, 3, 1)).unwrap();
        renderer.clear(Rect::new(1, 0, 1, 1)).unwrap();

        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::inline('x'));
        assert_eq!(context.buffer[(0, 1)].grapheme(), Grapheme::inline(' '));
        assert!(context.buffer[(0, 1)].style.is_empty());
        assert_eq!(context.buffer[(0, 2)].grapheme(), Grapheme::inline('x'));
    }

    #[test]
    fn test_render_document_with_padding() {
        use crate::Space;

        let mut context = context(20, 10);
        let document = &mut context.document;

        // Root with padding — children should render inside the content area
        let root = document.root_element_mut();
        root.padding = (2, 2).into();
        root.flex_direction = FlexDirection::Column;

        let child = document.insert_with(Element::Span(Cow::Borrowed("AB")), |node| {
            node.color = Some(Color::Blue);
        });

        document.compute_layout(Space::new(20,10));

        document.paint(&mut BufferPainter::new(&mut context.buffer, &mut context.arena)).unwrap();

        // Text should appear at content area offset (padding=2 on each side)
        let child_content = document.content_bounds(child);
        let text_x = child_content.min.x as usize;
        let text_y = child_content.min.y as usize;
        assert_eq!(
            context.buffer[(text_y, text_x)].grapheme(),
            Grapheme::inline('A')
        );
        assert_eq!(
            context.buffer[(text_y, text_x + 1)].grapheme(),
            Grapheme::inline('B')
        );
        // Origin cell should be empty (it's in the padding)
        assert_eq!(context.buffer[(0, 0)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn test_render_nested_nodes() {
        use crate::Space;

        let mut context = context(30, 15);
        let document = &mut context.document;

        // Root with padding, column layout
        let root = document.root_element_mut();
        root.padding = (1, 1).into();
        root.flex_direction = FlexDirection::Column;

        // Child div with its own padding
        let child_div = document.insert_with(Element::Div(), |node| {
            node.padding = (1, 1).into();
            node.flex_direction = FlexDirection::Column;
        });

        // Grandchild text inside the child div
        let text_id = document.insert_at_with(Element::Span("OK"), At::Child(child_div), |node| {
            node.color = Some(Color::Blue);
        });

        document.compute_layout(Space::new(30, 15));

        let text_content = document.content_bounds(text_id);

        document.paint(&mut BufferPainter::new(&mut context.buffer, &mut context.arena)).unwrap();

        let div_bounds = document.border_bounds(child_div);
        let text_bounds = document.border_bounds(text_id);

        // Absolute position = parent bounds + child bounds (taffy locations are parent-relative)
        let tx = (div_bounds.min.x + text_bounds.min.x) as usize;
        let ty = (div_bounds.min.y + text_bounds.min.y) as usize;
        assert_eq!(context.buffer[(ty, tx)].grapheme(), Grapheme::inline('O'));
        assert_eq!(
            context.buffer[(ty, tx + 1)].grapheme(),
            Grapheme::inline('K')
        );
        // Padding area should be empty
        assert_eq!(context.buffer[pos!(0, 0)].grapheme(), Grapheme::EMPTY);
    }

    #[test]
    fn test_render_stacked_children() {
        use crate::Space;

        let mut context = context(30, 15);
        let document = &mut context.document;

        let root = document.root_element_mut();
        root.flex_direction = FlexDirection::Column;

        // Two stacked children in column layout
        let child_a = document.insert_with(Element::Span(Cow::Borrowed("AA")), |node| {
            node.color = Some(Color::Blue);
        });
        let child_b = document.insert_with(Element::Span(Cow::Borrowed("BB")), |node| {
            node.color = Some(Color::Green);
        });

        document.compute_layout(Space::new(30,15));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        document.paint(
            &mut BufferPainter::new(&mut context.buffer, &mut context.arena),
        )
        .unwrap();

        // First child
        assert_eq!(
            context.buffer[(a_bounds.min.y as usize, a_bounds.min.x as usize)].grapheme(),
            Grapheme::inline('A')
        );
        // Second child should be below the first
        assert!(
            b_bounds.min.y > a_bounds.min.y,
            "B should be below A: A.y={}, B.y={}",
            a_bounds.min.y,
            b_bounds.min.y
        );
        assert_eq!(
            context.buffer[(b_bounds.min.y as usize, b_bounds.min.x as usize)].grapheme(),
            Grapheme::inline('B')
        );
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

        let root = document.root_element_mut();
        root.background = Some(Color::Red);
        root.padding = (1, 1).into();

        document.compute_layout(Space::new(10,5));
        document.paint(
            &mut BufferPainter::new(&mut context.buffer, &mut context.arena),
        )
        .unwrap();

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
            node.size = crate::Size::new(10,4);
        });

        document.compute_layout(Space::new(10,4));
        document.paint(
            &mut BufferPainter::new(&mut context.buffer, &mut context.arena),
        )
        .unwrap();

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

        let root = document.root_element_mut();
        root.background = Some(Color::Red);

        document.insert_with(Element::Span(Cow::Borrowed("X")), |node| {
            node.background = Some(Color::None);
        });

        document.compute_layout(Space::new(10,3));
        document.paint(
            &mut BufferPainter::new(&mut context.buffer, &mut context.arena),
        )
        .unwrap();

        // Every cell that isn't the glyph itself should still carry the
        // root's red backdrop — the span must not overwrite it.
        let h = context.buffer.bounds().height() as usize;
        let w = context.buffer.bounds().width() as usize;
        for y in 0..h {
            for x in 0..w {
                let cell = &context.buffer[(y, x)];
                if cell.grapheme() == Grapheme::inline('X') {
                    continue;
                }
                assert_eq!(
                    cell.style.background,
                    Color::Red,
                    "cell ({y},{x}) lost root's red backdrop",
                );
            }
        }
    }

    /// End-to-end coverage of the `crates/ui/src/bin/main.rs` scene:
    /// root with padding + red background, transparent text element, a
    /// bordered container with three colored children. Asserts the
    /// characteristics the crate binary renders.
    #[test]
    fn test_render_main_scene() {
        use crate::Space;

        let mut context = context(20, 20);
        let document = &mut context.document;

        let root = document.root_element_mut();
        root.background = Some(Color::Red);
        root.color = Some(Color::White);
        root.padding = (1, 1).into();

        document.insert(
            Element::Span(Cow::Borrowed("Hello"))
                .background(Color::None)
                .font_weight(FontWeight::Bold),
        );

        let abc = document.insert(Element::Div().border(Border::Bold));
        let a = document.insert_at(Element::Div().background(Color::Green), At::Child(abc));
        let b = document.insert_at(Element::Div().background(Color::Yellow), At::Child(abc));
        let c = document.insert_at(Element::Div().background(Color::Blue), At::Child(abc));

        document.insert_at(Element::Span("A"), At::Child(a));
        document.insert_at(Element::Span("B"), At::Child(b));
        document.insert_at(Element::Span("C"), At::Child(c));

        document.compute_layout(Space::new(20,20));
        document.paint(
            &mut BufferPainter::new(&mut context.buffer, &mut context.arena),
        )
        .unwrap();

        let w = context.buffer.bounds().width() as usize;
        let h = context.buffer.bounds().height() as usize;

        // Root's padding row (top/bottom) is filled with red.
        for x in 0..w {
            assert_eq!(
                context.buffer[(0, x)].style.background,
                Color::Red,
                "top padding row should be red at col {x}",
            );
            assert_eq!(
                context.buffer[(h - 1, x)].style.background,
                Color::Red,
                "bottom padding row should be red at col {x}",
            );
        }

        // Root's padding column (left/right) is filled with red.
        for y in 0..h {
            assert_eq!(
                context.buffer[(y, 0)].style.background,
                Color::Red,
                "left padding col should be red at row {y}",
            );
            assert_eq!(
                context.buffer[(y, w - 1)].style.background,
                Color::Red,
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
        assert_eq!(
            context.buffer[(top, left)].grapheme(),
            Grapheme::inline('┏')
        );
        assert_eq!(
            context.buffer[(top, right)].grapheme(),
            Grapheme::inline('┓')
        );
        assert_eq!(
            context.buffer[(bottom, left)].grapheme(),
            Grapheme::inline('┗')
        );
        assert_eq!(
            context.buffer[(bottom, right)].grapheme(),
            Grapheme::inline('┛')
        );
    }

    #[test]
    fn test_render_row_children() {
        use crate::Space;

        let mut context = context(30, 5);
        let document = &mut context.document;

        let root = document.root_element_mut();
        root.display = crate::Display::Flex;
        root.flex_direction = FlexDirection::Row;

        let child_a = document.insert_with(Element::Span(Cow::Borrowed("L")), |node| {
            node.color = Some(Color::Blue);
        });
        let child_b = document.insert_with(Element::Span(Cow::Borrowed("R")), |node| {
            node.color = Some(Color::Green);
        });

        document.compute_layout(Space::new(30,5));

        let a_bounds = document.content_bounds(child_a);
        let b_bounds = document.content_bounds(child_b);

        document.paint(
            &mut BufferPainter::new(&mut context.buffer, &mut context.arena),
        )
        .unwrap();

        // Side by side in row layout
        assert_eq!(
            context.buffer[(a_bounds.min.y as usize, a_bounds.min.x as usize)].grapheme(),
            Grapheme::inline('L')
        );
        assert!(b_bounds.min.x > a_bounds.min.x, "R should be right of L");
        assert_eq!(
            context.buffer[(b_bounds.min.y as usize, b_bounds.min.x as usize)].grapheme(),
            Grapheme::inline('R')
        );
    }
}
