use std::ops::BitOr;
use derive_more::Deref;
use unicode_width::UnicodeWidthStr;
use ansi::{Color, Escape, Style};
use ansi::io::Write;
use crate::{Align, Alignment, Buffer, BufferIndex, Constraint, Constraints, Edges, Point, Rect, Size};
use crate::position::{Position, Region};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    Empty,
    Text(String),
    Fill(char),
}

#[derive(Clone, Debug)]
pub enum Node {
    Base(Content),
    Style(Style, Box<Node>),
    Pad(Edges, Box<Node>),
    Size(Constraints, Box<Node>),
    Align(Alignment, Box<Node>),
    Stack(Vec<Node>),
    Row(Vec<Node>),
    Layer(Vec<Node>),
}


/// Layout result: node reference with resolved rect
#[derive(Debug, Deref)]
pub struct LayoutNode<'a> {
    #[deref]
    pub node: &'a Node,
    pub bounds: Rect,
    pub children: Vec<LayoutNode<'a>>,
}

impl<'a> LayoutNode<'a> {
    pub fn new(node: &'a Node, bounds: Rect, children: Vec<LayoutNode<'a>>) -> Self {
        Self { node, bounds, children }
    }

    pub fn leaf(node: &'a Node, bounds: Rect) -> Self {
        Self::new(node, bounds, vec![])
    }
}


#[derive(Clone, Default)]
#[derive(Debug)]
pub struct Context {
    style: Style,
}

impl Context {
    fn add(&self, style: &Style) -> Self {
        Self { style: style.bitor(self.style) }
    }
}

pub fn layout<'a>(node: &'a Node, bounds: Rect, constraints: Constraints) -> LayoutNode<'a> {
    match node {
        Node::Base(_) => LayoutNode::leaf(node, bounds),

        Node::Style(_, child) => {
            let child_layout = layout(child, bounds, constraints);
            LayoutNode::new(node, bounds, vec![child_layout])
        }

        Node::Pad(edges, child) => {
            let inner_rect = bounds.shrink(edges);
            let inner_ct = constraints.shrink(edges);
            let child_layout = layout(child, inner_rect, inner_ct);
            LayoutNode::new(node, bounds, vec![child_layout])
        }

        Node::Size(node_constraints, child) => {
            let new_ct = node_constraints.constrain(constraints);
            let child_node = layout(child, bounds, new_ct);
            LayoutNode::new(node, bounds, vec![child_node])
        }

        Node::Align(alignment, child) => {
            let child_size = measure(child, Constraints::Max(bounds.width(), bounds.height()));
            let offset = alignment.offset(bounds.size(), child_size);
            let child_node = layout(child, Rect::new((bounds.min + offset), Point::new(child_size.width, child_size.height)), Constraints::Fixed(child_size.width, child_size.height));
            LayoutNode::new(node, bounds, vec![child_node])
        }

        Node::Stack(children) => {
            let mut y = bounds.y();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_h = bounds.height().saturating_sub(y - bounds.y());
                let child_ct = Constraints::Max(bounds.width(), remaining_h);
                let size = measure(child, child_ct);
                let child_rect = Rect::new((bounds.x(), y), (bounds.width(), size.height));
                laid_out.push(layout(child, child_rect, child_ct));
                y = y.saturating_add(size.height);
            }

            LayoutNode::new(node, bounds, laid_out)
        }

        Node::Row(children) => {
            let mut x = bounds.x();
            let mut laid_out = Vec::with_capacity(children.len());

            for child in children {
                let remaining_w = bounds.width().saturating_sub(x - bounds.x());
                let child_ct = Constraints::Max(remaining_w, bounds.height());
                let size = measure(child, child_ct);
                let child_rect = Rect::new((x, bounds.y()), (size.width, bounds.height()));
                laid_out.push(layout(child, child_rect, child_ct));
                x = x.saturating_add(size.width);
            }

            LayoutNode::new(node, bounds, laid_out)
        }

        Node::Layer(children) => {
            let laid_out = children
                .iter()
                .map(|child| layout(child, bounds, constraints))
                .collect();
            LayoutNode::new(node, bounds, laid_out)
        }
    }
}

/// Measure a node given constraints, return its desired size
pub fn measure(node: &Node, constraints: Constraints) -> Size {
    match node {
        Node::Base(Content::Empty) => {
            Size::ZERO
        }

        Node::Base(Content::Text(string)) => {
            let w = string.width();
            constraints.clamp(w, 1)
        }

        // Node::Base(Primitive::TextWrap(tw)) => {
        //     let lines = wrap_text(&tw.content, constraints.max_w);
        //     let h = lines.len() as u16;
        //     let w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
        //     let (w, h) = constraints.clamp(w, h);
        //     Size::new(w, h)
        // }

        Node::Base(Content::Fill(_)) => constraints.max(),

        Node::Style(_, child) | Node::Align(_, child) => measure(child, constraints),

        Node::Pad(edges, child) => {
            let inner = measure(child, constraints.shrink(edges));
            Size::new(
                inner.width + edges.horizontal(),
                inner.height + edges.vertical(),
            )
        }

        Node::Size(inner_constraints, child) => {
            measure(child, inner_constraints.constrain(constraints))
        }

        Node::Stack(children) => {
            let mut total_h = 0;
            let mut max_w = 0;

            for child in children {
                let size = measure(child, Constraints {
                    height: Constraint::Max(constraints.height.max_or(0).saturating_sub(total_h)),
                    ..constraints
                });
                total_h = total_h.saturating_add(size.height as usize);
                max_w = max_w.max(size.width as usize);
            }

            constraints.clamp(max_w, total_h)
        }

        Node::Row(children) => {
            let mut total_w = 0;
            let mut max_h = 0;

            for child in children {
                let size = measure(child, Constraints {
                    width: Constraint::Max(constraints.width.max_or(0).saturating_sub(total_w)),
                    ..constraints
                });
                total_w = total_w.saturating_add(size.width as usize);
                max_h = max_h.max(size.height as usize);
            }

            constraints.clamp(total_w, max_h)
        }

        Node::Layer(children) => {
            let mut max_w = 0;
            let mut max_h = 0;

            for child in children {
                let size = measure(child, constraints);
                max_w = max_w.max(size.width as usize);
                max_h = max_h.max(size.height as usize);
            }

            constraints.clamp(max_w, max_h)
        }
    }
}

pub fn render(layout: &LayoutNode<'_>, buffer: &mut Buffer, ctx: &Context) {
    let rect = layout.bounds;
    let region = Region::from(rect);

    dbg!(layout.bounds);

    match layout.node {
        Node::Base(Content::Empty) => {}

        Node::Base(Content::Text(s)) => {
            buffer.text(region.min..region.max, &s, &ctx.style);
        }

        // Node::Base(Primitive::TextWrap(tw)) => {
        //     let lines = wrap_text(&tw.content, rect.width());
        //
        //     for (i, line) in lines.iter().enumerate() {
        //         let y = rect.y + i as u16;
        //         if y >= rect.y + rect.h {
        //             break;
        //         }
        //
        //         let line_w = display_width(line);
        //         let x = rect.x + match tw.align {
        //             AlignX::Start => 0,
        //             AlignX::Center => (rect.w.saturating_sub(line_w)) / 2,
        //             AlignX::End => rect.w.saturating_sub(line_w),
        //         };
        //
        //         canvas.text(x, y, line, ctx.style);
        //     }
        // }

        Node::Base(Content::Fill(ch)) =>  {
            for pos in region {
                unsafe { buffer.get_unchecked_mut(pos) }.set_char(*ch);
            }
        }

        Node::Style(style, _) => {
            let new_ctx = ctx.add(style);


            if style.bg.is_some() {
                for pos in region {
                    unsafe { buffer.get_unchecked_mut(pos) }.bg = style.bg;
                }
            }

            for child in &layout.children {
                render(child, buffer, &new_ctx);
            }
        }

        Node::Pad(_, _) | Node::Size(_, _) | Node::Align(_, _) => {
            for child in &layout.children {
                render(child, buffer, ctx);
            }
        }

        Node::Stack(_) | Node::Row(_) | Node::Layer(_) => {
            for child in &layout.children {
                render(child, buffer, ctx);
            }
        }
    }
}
