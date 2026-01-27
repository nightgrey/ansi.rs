

/// Layout result: node reference with resolved rect
#[derive(Debug)]
pub struct LayoutNode<'a> {
    pub node: &'a Modifier,
    pub rect: Rect,
    pub children: Vec<LayoutNode<'a>>,
}

impl<'a> LayoutNode<'a> {
    fn leaf(node: &'a Modifier, rect: Rect) -> Self {
        Self { node, rect, children: vec![] }
    }

    fn with_children(node: &'a Modifier, rect: Rect, children: Vec<LayoutNode<'a>>) -> Self {
        Self { node, rect, children }
    }
}

/// Layout a tree to fit within a screen size
pub fn layout(root: &Modifier, screen: Size) -> LayoutNode<'_> {
    let ct = Constraints::Fixed(screen.width, screen.height);
    let rect = Rect::from(screen);
    place(root, rect, ct)
}

/// Measure a node given constraints, return its desired size
pub fn measure(node: &Modifier, constraints: Constraints) -> Size {
    match node {
        Modifier::Base(Primitive::Empty) => {
            constraints.clamp(0, 0)
        }

        Modifier::Base(Primitive::Text(s)) => {
            let w = s.width();
            constraints.clamp(w, 1)
        }

        // Modifier::Base(Primitive::TextWrap(tw)) => {
        //     let lines = wrap_text(&tw.content, ct.max_w);
        //     let h = lines.len() as u16;
        //     let w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
        //     let (w, h) = ct.clamp(w, h);
        //     Size::new(w, h)
        // }

        Modifier::Base(Primitive::Fill(_)) => constraints.max(),

        Modifier::Style(_, child) | Modifier::Align(_, child) => measure(child, constraints),

        Modifier::Pad(edges, child) => {
            let inner_ct = constraints.shrink(edges);
            let inner = measure(child, inner_ct);
            Size::new(
                inner.width + edges.horizontal(),
                inner.height + edges.vertical(),
            )
        }

        Modifier::Size(ext_w, ext_h, child) => {
            let new_ct = constraints.with_extent(*ext_w, *ext_h);
            measure(child, new_ct)
        }

        Modifier::Stack(children) => {
            let mut total_h = 0u16;
            let mut max_w = 0u16;

            for child in children {
                let child_ct = Constraints {
                    max_h: constraints.max_h.saturating_sub(total_h),
                    ..constraints
                };
                let size = measure(child, child_ct);
                total_h = total_h.saturating_add(size.height);
                max_w = max_w.max(size.width);
            }

            let (w, h) = constraints.clamp(max_w, total_h);
            Size::new(w, h)
        }

        Modifier::Row(children) => {
            let mut total_w = 0;
            let mut max_h = 0;

            for child in children {
                let child_ct = Constraints {
                    width: constraints.width.max_or(0).saturating_sub(total_w),
                    ..constraints
                };
                let size = measure(child, child_ct);
                total_w = total_w.saturating_add(size.w);
                max_h = max_h.max(size.h);
            }

            let (w, h) = constraints.clamp(total_w, max_h);
            Size::new(w, h)
        }

        Modifier::Layer(children) => {
            let mut max_w = 0u16;
            let mut max_h = 0u16;

            for child in children {
                let size = measure(child, constraints);
                max_w = max_w.max(size.w);
                max_h = max_h.max(size.h);
            }

            let (w, h) = constraints.clamp(max_w, max_h);
            Size::new(w, h)
        }
    }
}

fn place<'a>(node: &'a Modifier, rect: Rect, ct: Constraints) -> LayoutNode<'a> {
    match node {
        Modifier::Base(_) => LayoutNode::leaf(node, rect),

        Modifier::Style(_, child) => {
            let child_layout = place(child, rect, ct);
            LayoutNode::with_children(node, rect, vec![child_layout])
        }

        Modifier::Pad(edges, child) => {
            let inner_rect = rect.shrink(edges);
            let inner_ct = ct.shrink(edges);
            let child_layout = place(child, inner_rect, inner_ct);
            LayoutNode::with_children(node, rect, vec![child_layout])
        }

        Modifier::Size(ext_w, ext_h, child) => {
            let new_ct = ct.with_extent(*ext_w, *ext_h);
            let child_layout = place(child, rect, new_ct);
            LayoutNode::with_children(node, rect, vec![child_layout])
        }

        Modifier::Align(align, child) => {
            let child_size = measure(child, Constraints::Max(rect.width(), rect.height()));
            let (ox, oy) = align.offset(rect.size(), child_size);
            let child_rect = Rect::new(rect.x() + ox, rect.y() + oy, child_size.width, child_size.height);
            let child_ct = Constraints::Fixed(child_size.width, child_size.height);
            let child_layout = place(child, child_rect, child_ct);
            LayoutNode::with_children(node, rect, vec![child_layout])
        }

        Modifier::Stack(children) => {
            let mut y = rect.y();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_h = rect.height().saturating_sub(y - rect.y());
                let child_ct = Constraints::Max(rect.width(), remaining_h);
                let size = measure(child, child_ct);
                let child_rect = Rect::new(rect.x(), y, rect.width(), size.h);
                laid_out.push(place(child, child_rect, child_ct));
                y = y.saturating_add(size.height);
            }

            LayoutNode::with_children(node, rect, laid_out)
        }

        Modifier::Row(children) => {
            let mut x = rect.x();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_w = rect.width().saturating_sub(x - rect.x());
                let child_ct = Constraints::Max(remaining_w, rect.height());
                let size = measure(child, child_ct);
                let child_rect = Rect::new(x, rect.y(), size.w, rect.height());
                laid_out.push(place(child, child_rect, child_ct));
                x = x.saturating_add(size.width);
            }

            LayoutNode::with_children(node, rect, laid_out)
        }

        Modifier::Layer(children) => {
            let laid_out = children
                .iter()
                .map(|child| place(child, rect, ct))
                .collect();
            LayoutNode::with_children(node, rect, laid_out)
        }
    }
}


/// Render a layout tree to a canvas
pub fn render(layout: &LayoutNode<'_>, canvas: &mut impl Canvas) {
    render_node(layout, canvas, &RenderCtx::default());
}

#[derive(Clone, Default)]
struct RenderCtx {
    style: Style,
}

impl RenderCtx {
    fn with_style(&self, style: &Style) -> Self {
        Self { style: self.style.merge(style) }
    }
}

fn render_node(ln: &LayoutNode<'_>, canvas: &mut impl Canvas, ctx: &RenderCtx) {
    let rect = ln.rect;

    match ln.node {
        Modifier::Base(Primitive::Empty) => {}

        Modifier::Base(Primitive::Text(s)) => {
            let clipped = clip_text(s, rect.width());
            canvas.text(rect.x(), rect.y(), &clipped, ctx.style);
        }

        Modifier::Base(Primitive::TextWrap(tw)) => {
            let lines = wrap_text(&tw.content, rect.width());

            for (i, line) in lines.iter().enumerate() {
                let y = rect.y() + i as u16;
                if y >= rect.y() + rect.height() {
                    break;
                }

                let line_w = display_width(line);
                let x = rect.x() + match tw.align {
                    AlignX::Start => 0,
                    AlignX::Center => (rect.width().saturating_sub(line_w)) / 2,
                    AlignX::End => rect.width().saturating_sub(line_w),
                };

                canvas.text(x, y, line, ctx.style);
            }
        }

        Modifier::Base(Primitive::Fill(ch)) => {
            canvas.fill(rect, Cell::new(*ch, ctx.style));
        }

        Modifier::Style(style, _) => {
            let new_ctx = ctx.with_style(style);

            if style.bg.is_some() {
                canvas.fill(rect, Cell::new(' ', new_ctx.style));
            }

            for child in &ln.children {
                render_node(child, canvas, &new_ctx);
            }
        }

        Modifier::Pad(_, _) | Modifier::Size(_, _, _) | Modifier::Align(_, _) => {
            for child in &ln.children {
                render_node(child, canvas, ctx);
            }
        }

        Modifier::Stack(_) | Modifier::Row(_) | Modifier::Layer(_) => {
            for child in &ln.children {
                render_node(child, canvas, ctx);
            }
        }
    }
}

fn clip_text(s: &str, max_width: u16) -> String {
    use unicode_width::UnicodeWidthChar;

    let mut result = String::new();
    let mut width = 0u16;

    for ch in s.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if width + ch_width > max_width {
            break;
        }
        result.push(ch);
        width += ch_width;
    }

    result
}
