use std::borrow::Cow;
use std::io;
use std::io::{BufRead, Cursor, Read, Write};
use std::ops::Index;
use derive_more::{Deref, DerefMut, Index, IndexMut};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use ansi::{escape, Attribute, Color, Style};
use geometry::{Axis, Contains, Intersect, Point, Rect, Size};
use sigil::{Buffer, Capabilities, Grapheme, GraphemeArena, Rasterizer};
use gloss::*;
use tree::At;

fn main() {
    let mut document = Document::new();

    let root = document.node_mut(document.root);
    root.align_items.insert(AlignItems::Start);
    root.justify_items.insert(JustifyItems::Start);

    let n = document.insert_with(Node::Span(Cow::Borrowed("Hello Worldwwwwwwwwwwwwwwwww!")), |node| {
        node.color = Color::Red;
    });

    let row = document.insert_with(Node::Div(), |node| {
        node.flex_direction = FlexDirection::Row;
        node.flex_grow = 1.0;
        node.gap = Axis { horizontal: Dimension::Length(1), vertical: Dimension::Length(1) };
    });

    let a = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background_color = Color::Green;
    });


    let b = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background_color = Color::Yellow;
    });

    let c = document.insert_at_with(Node::Div(), At::Child(row), |node| {
        node.background_color = Color::Blue;
    });

    let mut arena = GraphemeArena::new();
    let mut buffer = Buffer::new(10, 11);
    let mut out = io::stdout().lock();
    let mut rasterizer = Rasterizer::inline(buffer.width, buffer.height);

    document.compute_layout(Space { width: Available::Definite(80), height: Available::Definite(24) });
    let mut renderer = Renderer::new(BufferContext::new(&mut buffer, &mut arena));


    renderer.render(&document).unwrap();
    rasterizer.raster(&buffer, &arena);
    rasterizer.write(&mut out).unwrap();

    dbg!(document.bounds(n));
}


#[derive(Debug, Index, IndexMut, Deref, DerefMut)]
struct BufferContext<'a> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    buffer: &'a mut Buffer,
    arena: &'a mut GraphemeArena,
    stack: Vec<Rect>,
}
impl<'a> BufferContext<'a> {
    pub fn new(buffer: &'a mut Buffer, arena: &'a mut GraphemeArena) -> Self {
        let bounds = Rect::bounds(0, 0, buffer.width, buffer.height);
        Self {
            buffer,
            arena,
            stack: vec![bounds],
        }
    }

    #[inline]
    pub fn current(&self) -> Rect {
        // SAFETY: clips is never empty — `new` pushes one, `pop` refuses to
        // remove the last.
        *self.stack.last().unwrap()
    }

    pub fn push(&mut self, rect: Rect) {
        let next = self.bounds().intersect(&rect).intersect(&self.current());
        self.stack.push(next);
    }

    pub fn pop(&mut self) -> io::Result<()> {
        assert!(self.stack.len() > 1, "cannot pop the root push");
        self.stack.pop();
        Ok(())
    }

    fn save(&mut self) -> io::Result<()> {
        self.stack.push(self.current());
        Ok(())
    }
    fn restore(&mut self) -> io::Result<()> {
        self.pop();
        Ok(())
    }

    fn stroke(&mut self, bounds: Rect, style: ansi::Style, border: Border) {
        todo!()
    }

    fn stroke_styled(&mut self, bounds: Rect, style: ansi::Style, border: Border, border_style: ansi::Style) {
        todo!()
    }

    fn fill(&mut self, bounds: Rect, char: char, style: Style) {
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                self[(y, x)].set(Grapheme::from_char(char), char.width().unwrap_or(0) as u8, style);
            }
        }
    }

    fn fill_char(&mut self, bounds: Rect, char: char) {
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                self[(y, x)].set(Grapheme::from_char(char), char.width().unwrap_or(0) as u8, Style::None);
            }
        }
    }

    fn fill_style(&mut self, bounds: Rect, style: Style) {
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                self[(y, x)].set_style(style);
            }
        }
    }

    fn draw_text(&mut self, row: usize, col: usize, text: &str, style: Style) {
        let mut i = 0;

        let mut remaining = self.width.saturating_sub(col);
        for (grapheme, width) in text
            .graphemes(true)
            .filter(|symbol| !symbol.contains(char::is_control))
            .map(|symbol| (symbol, symbol.width()))
            .filter(|&(symbol, width)| width > 0)
            .map_while(|(symbol, width)| {
                remaining = remaining.checked_sub(width)?;
                Some((symbol, width))
            })
        {
            let grapheme = Grapheme::encode(grapheme, &mut self.arena);
            // Set the starting cell
            self[(row, col + i)].set(grapheme, width as u8, style);
            let next_symbol = i + width;
            i += 1;

            // Reset subsequent cells for multi-width graphemes
            while i < next_symbol {
                self[(row, col + i)].set_space(style);
                i += 1;
            }
        }
    }

    fn maybe_resize(&mut self, size: Size) -> Result<(), io::Error> {
        if size.width != self.buffer.width || size.height != self.buffer.height {
            self.buffer.resize(size.width, size.height);
        }

        Ok(())
    }

    fn finish(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

fn apply<'a>(to: &'a mut Style, from: &gloss::Style) -> &'a mut Style {
    to.background = from.background_color;
    to.foreground = from.color;

    match from.text_decoration {
        TextDecorationLine::None => to.remove(Attribute::Underline | Attribute::Strikethrough),
        TextDecorationLine::Underline => to.insert(Attribute::Underline),
        TextDecorationLine::LineThrough => to.insert(Attribute::Strikethrough),
    }

    match from.font_weight {
        FontWeight::Normal => to.remove(Attribute::Bold),
        FontWeight::Bold => to.insert(Attribute::Bold),
    }
    match from.font_style {
        FontStyle::Normal => to.remove(Attribute::Italic),
        FontStyle::Italic => to.insert(Attribute::Italic),
    }

    to
}

fn ansi(style: &gloss::Style) -> Style {
    let mut ansi = ansi::Style::default();
    apply(&mut ansi, style);
    ansi
}

impl RenderContext for BufferContext<'_> {
    type Error = io::Error;

    fn stroke(&mut self, bounds: Rect, style: gloss::Style) {
        todo!()
    }

    fn fill(&mut self, bounds: Rect, char: char, style: gloss::Style) {
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                self[(y, x)].grapheme = Grapheme::from_char(char);
                apply(&mut self[(y, x)].style, &style);
            }
        }
    }

    fn fill_char(&mut self, bounds: Rect, char: char) {
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                self[(y, x)].grapheme = Grapheme::from_char(char);
            }
        }
    }

    fn fill_style(&mut self, bounds: Rect, style: gloss::Style) {
        let style = ansi(&style);
        dbg!("fill_style", bounds, style);
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
               self[(y, x)].set_style(style);
            }
        }
    }

    fn push(&mut self, bounds: Rect) {
        BufferContext::push(self, bounds);
    }

    fn pop(&mut self) -> Result<(), Self::Error> {
        BufferContext::pop(self)
    }

    fn save(&mut self) -> Result<(), Self::Error> {
        BufferContext::save(self)
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        BufferContext::restore(self)
    }

    fn draw_text(&mut self, position: Point, text: &str, style: gloss::Style) {
        BufferContext::draw_text(self, position.y, position.x, text, ansi(&style));
    }

    fn maybe_resize(&mut self, size: Size) -> Result<(), Self::Error> {
        BufferContext::maybe_resize(self, size)
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        BufferContext::finish(self)
    }
}