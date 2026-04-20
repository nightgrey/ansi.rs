use ansi::{Color, Style};

use criterion::*;
use criterion::{Criterion, criterion_group, criterion_main};
use geometry::{Bound, Point, Position, p};
use gloss::*;
use std::hint::black_box;
use std::io;
use tree::At;

// ═══════════════════════════════════════════════════════════════════
// Terminal-level workloads (Rezi-comparable, 40×120)
// ═══════════════════════════════════════════════════════════════════

/// Standard terminal size matching Rezi's benchmark suite.
const W: usize = 120;
const H: usize = 40;

/// Setup an engine with some layouted elements.
fn setup<'a>(width: usize, height: usize) -> Engine<'a> {
    let mut engine = Engine::new(width, height);

    let root = engine.root_mut();
    root.background = Some(Color::Red);
    root.color = Some(Color::White);
    root.padding = 1.into();
    root.display = Display::Flex;
    root.flex_direction = FlexDirection::Column;
    root.align_items = AlignItems::Center.into();
    root.justify_content = JustifyContent::Center.into();

    engine.set_root(
        Element::Div()
            .background(Color::Red)
            .color(Color::White)
            .padding(1)
            .flex()
            .flex_col()
            .items_center()
            .content_center(),
    );

    engine.insert(
        Element::Span("👨🏿👨🏿 Hello")
            .background(Color::None)
            .border(Border::Bold)
            .bold(),
    );

    let abc = engine.insert(Element::Div().border(Border::Bold));
    let a = engine.insert_at(Element::Div().background(Color::Green), At::Child(abc));
    let b = engine.insert_at(Element::Div().background(Color::Yellow), At::Child(abc));
    let c = engine.insert_at(Element::Div().background(Color::Blue), At::Child(abc));

    engine.insert_at(Element::Span("A"), At::Child(a));
    engine.insert_at(Element::Span("B"), At::Child(b));
    engine.insert_at(Element::Span("C"), At::Child(c));

    engine
}

/// `terminal-rerender`: stable frame, one value changes per frame.
fn terminal_rerender(c: &mut Criterion) {
    let mut engine = Engine::new(W, H);
    engine.layout_and_paint();
    engine.paint_with(|ctx| {
        ctx.char(p!(W / 2, H / 2), 'A');
    });
    let mut output = io::Cursor::new(Vec::<u8>::new());

    c.bench_function("terminal-rerender", |b| {
        b.iter(|| {
            engine.render(&mut output);
            engine.invalidate();
            engine.render(&mut output);
        });
    });
}

// ═══════════════════════════════════════════════════════════════════
// Diffing
// ═══════════════════════════════════════════════════════════════════

fn setup_diff(width: usize, height: usize) -> (Buffer, Buffer) {
    let mut engine = setup(width, height);
    engine.layout_and_paint();

    (engine.back_buffer().clone(), engine.front_buffer().clone())
}

fn buffer_diffs(c: &mut Criterion) {
    let (back, front) = setup_diff(1000, 1000);

    c.bench_function("1000x1000", |b| {
        b.iter(|| {
            while let Some(run) = BufferDiff::new(&back, &front).next() {
                black_box(run);
            }
        });
    });
}

// ═══════════════════════════════════════════════════════════════════

criterion_group!(terminal, terminal_rerender,);

criterion_group!(buffer, buffer_diffs);

criterion_main!(buffer);
