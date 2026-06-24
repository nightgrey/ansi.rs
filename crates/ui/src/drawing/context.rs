use crate::{Border, Layout};
use bon::Builder;
use geometry::{Bound, Outer, Point, Rect, Size};

/// Per-call overrides for drawing operations.
#[derive(Debug, Clone, Default, Builder, Copy)]
pub struct DrawingOptions {
    pub layout: Option<Layout>,
    pub glyph: Option<char>,
    pub border: Option<Border>,
}

/// Primitive operations implemented by a drawing backend.
pub trait DrawingContext {
    type Error;

    fn current_clip(&self) -> Rect;
    fn current_style(&self) -> Layout;
    fn current_glyph(&self) -> char;
    fn current_border_style(&self) -> Border;

    fn style(&mut self, style: Layout) -> &mut Self;
    fn glyph(&mut self, glyph: char) -> &mut Self;
    fn border_style(&mut self, border: Border) -> &mut Self;

    fn clip(&mut self, rect: Rect) -> &mut Self;
    fn translate(&mut self, offset: Point) -> &mut Self;

    fn rect_with(&mut self, rect: Rect, options: DrawingOptions) -> Result<&mut Self, Self::Error>;

    /// Draw one line of text and return the number of display cells written.
    ///
    /// Control and zero-width graphemes are skipped. A wide grapheme is only
    /// written when its complete display width fits inside the current clip.
    fn text_with(
        &mut self,
        position: Point,
        text: &str,
        options: DrawingOptions,
    ) -> Result<usize, Self::Error>;

    /// Draw one scalar and return its display width, or zero when it is a
    /// control/zero-width scalar or does not fit completely inside the clip.
    fn char_with(
        &mut self,
        position: Point,
        ch: char,
        options: DrawingOptions,
    ) -> Result<usize, Self::Error>;

    fn save(&mut self) -> &mut Self;
    fn restore(&mut self) -> &mut Self;

    fn resize(&mut self, size: Size) -> Result<&mut Self, Self::Error>;
    fn finish(&mut self) -> Result<&mut Self, Self::Error>;
}

/// Ergonomic and derived drawing operations shared by all backends.
pub trait DrawingContextExtension: DrawingContext {
    fn rect(&mut self, rect: impl Into<Rect>) -> Result<&mut Self, Self::Error> {
        self.rect_with(rect.into(), DrawingOptions::default())
    }

    fn text(
        &mut self,
        position: impl Into<Point>,
        text: impl AsRef<str>,
    ) -> Result<usize, Self::Error> {
        self.text_with(position.into(), text.as_ref(), DrawingOptions::default())
    }

    fn char(&mut self, position: impl Into<Point>, ch: char) -> Result<usize, Self::Error> {
        self.char_with(position.into(), ch, DrawingOptions::default())
    }

    fn horizontal_line(
        &mut self,
        position: impl Into<Point>,
        length: u16,
    ) -> Result<&mut Self, Self::Error> {
        self.horizontal_line_with(position, length, DrawingOptions::default())
    }

    fn horizontal_line_with(
        &mut self,
        position: impl Into<Point>,
        length: u16,
        options: DrawingOptions,
    ) -> Result<&mut Self, Self::Error> {
        let position = position.into();
        self.rect_with(Rect::new(position.x, position.y, length, 1), options)
    }

    fn vertical_line(
        &mut self,
        position: impl Into<Point>,
        length: u16,
    ) -> Result<&mut Self, Self::Error> {
        self.vertical_line_with(position, length, DrawingOptions::default())
    }

    fn vertical_line_with(
        &mut self,
        position: impl Into<Point>,
        length: u16,
        options: DrawingOptions,
    ) -> Result<&mut Self, Self::Error> {
        let position = position.into();
        self.rect_with(Rect::new(position.x, position.y, 1, length), options)
    }

    fn outline(&mut self, rect: impl Into<Rect>) -> Result<&mut Self, Self::Error> {
        self.outline_with(rect, DrawingOptions::default())
    }

    fn outline_with(
        &mut self,
        rect: impl Into<Rect>,
        options: DrawingOptions,
    ) -> Result<&mut Self, Self::Error> {
        let rect = rect.into();

        self.horizontal_line_with(rect.min, rect.width(), options)?;

        if rect.height() > 1 {
            self.horizontal_line_with(
                Point::new(rect.left(), rect.bottom() - 1),
                rect.width(),
                options,
            )?;
        }

        if rect.height() > 2 {
            self.vertical_line_with(
                Point::new(rect.left(), rect.top() + 1),
                rect.height() - 2,
                options,
            )?;
        }

        if rect.width() > 1 && rect.height() > 2 {
            self.vertical_line_with(
                Point::new(rect.right() - 1, rect.top() + 1),
                rect.height() - 2,
                options,
            )?;
        }

        Ok(self)
    }

    fn border(&mut self, rect: impl Into<Rect>) -> Result<&mut Self, Self::Error> {
        self.border_with(rect, DrawingOptions::default())
    }

    fn border_with(
        &mut self,
        rect: impl Into<Rect>,
        options: DrawingOptions,
    ) -> Result<&mut Self, Self::Error> {
        let mut rect = rect.into();
        let border = options
            .border
            .unwrap_or_else(|| self.current_border_style())
            .into_symbols();

        rect.max.x = rect.max.x.saturating_sub(border.right.width() as u16);
        rect.max.y = rect.max.y.saturating_sub(border.bottom.width() as u16);

        if rect.is_empty() {
            return Ok(self);
        }

        let draw = |ctx: &mut Self, position: Point, symbol: crate::symbols::Symbol| {
            ctx.char_with(
                position,
                symbol.symbol(),
                DrawingOptions {
                    glyph: Some(symbol.symbol()),
                    ..options
                },
            )
        };

        draw(self, Point::new(rect.left(), rect.top()), border.top_left)?;
        draw(self, Point::new(rect.right(), rect.top()), border.top_right)?;
        draw(
            self,
            Point::new(rect.left(), rect.bottom()),
            border.bottom_left,
        )?;
        draw(
            self,
            Point::new(rect.right(), rect.bottom()),
            border.bottom_right,
        )?;

        let left_offset = border.left.width() as u16;
        for x in (rect.left() + left_offset)..rect.right() {
            draw(self, Point::new(x, rect.top()), border.top)?;
            draw(self, Point::new(x, rect.bottom()), border.bottom)?;
        }

        let top_offset = border.top.width() as u16;
        for y in (rect.top() + top_offset)..rect.bottom() {
            draw(self, Point::new(rect.left(), y), border.left)?;
            draw(self, Point::new(rect.right(), y), border.right)?;
        }

        Ok(self)
    }

    /// Fill an area using the current glyph and style.
    fn fill(&mut self, rect: impl Into<Rect>) -> Result<&mut Self, Self::Error> {
        self.rect(rect)
    }

    /// Reset an area to unstyled spaces, independently of the current state.
    fn clear(&mut self, rect: impl Into<Rect>) -> Result<&mut Self, Self::Error> {
        self.rect_with(
            rect.into(),
            DrawingOptions {
                layout: Some(Layout::DEFAULT),
                glyph: Some(' '),
                border: None,
            },
        )
    }

    fn with(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.save();
        f(self);
        self.restore()
    }

    fn try_with<R>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<R, Self::Error>,
    ) -> Result<R, Self::Error> {
        self.save();
        let result = f(self);
        self.restore();
        result
    }

    fn within(&mut self, rect: impl Into<Rect>, f: impl FnOnce(&mut Self)) -> &mut Self {
        let rect = rect.into();

        self.with(|ctx| {
            ctx.translate(rect.min);
            ctx.clip(Rect::from(rect.size()));
            f(ctx);
        })
    }
}

impl<T: DrawingContext + ?Sized> DrawingContextExtension for T {}
