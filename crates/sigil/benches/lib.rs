use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use derive_more::{Deref, DerefMut};
use ansi::{Color, Style};
use sigil::buffer::{Buffer, Cell};
use sigil::GraphemeArena;
use sigil::rasterizer::{Capabilities, Rasterizer as _Rasterizer};

#[derive(Clone, Debug, Deref, DerefMut)]
struct Rasterizer(
    #[deref]
    #[deref_mut]
    _Rasterizer,
    GraphemeArena
);
impl Rasterizer {
    /// Create a fullscreen rasterizer with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self(_Rasterizer::new(width, height), GraphemeArena::new())
    }

    /// Create an inline rasterizer (renders in the normal scrollback region).
    pub fn inline(width: usize, height: usize) -> Self {
        Self(_Rasterizer::inline(width, height), GraphemeArena::new())
    }

    pub fn with_capabilities(mut self, caps: Capabilities) -> Self {
        self.0 = self.0.with_capabilities(caps);
        self
    }

    fn render(&mut self, buffer: &Buffer) {
        self.0.render(buffer, &self.1);
    }
}

/// Standard terminal size matching Rezi's benchmark suite.
const W: usize = 120;
const H: usize = 40;

// ── Helpers ─────────────────────────────────────────────────────────

/// Build a buffer where every cell has the given char and style.
fn filled_buffer(width: usize, height: usize, ch: char, style: Style) -> Buffer {
    let chars: Vec<_> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (y, x, ch, style)))
        .collect();
    Buffer::from_chars(width, height, &chars)
}

/// Build a "dashboard" buffer: header row (bold white-on-blue), status bar
/// (black-on-yellow), body with 3 column-like regions of different styles,
/// and 24 "service" rows with alternating fg colors. Approximates the
/// content complexity of Rezi's `terminal-full-ui` scenario.
fn full_ui_buffer() -> Buffer {
    let header = Style::default().bold().foreground(Color::Rgb(255, 255, 255)).background(Color::Index(4));
    let status = Style::default().foreground(Color::Index(0)).background(Color::Index(3));
    let col_a = Style::default().foreground(Color::Rgb(0, 255, 0));
    let col_b = Style::default().foreground(Color::Rgb(255, 165, 0)).bold();
    let col_c = Style::default().foreground(Color::Index(6)).italic();

    let mut chars = Vec::with_capacity(W * H);

    // Row 0: header
    for x in 0..W {
        chars.push((0, x, if x < 20 { 'D' } else { ' ' }, header));
    }

    // Rows 1..38: body with 3 columns, 24 "services" cycling through them.
    for y in 1..H - 1 {
        let svc_idx = (y - 1) % 24;
        let ch = (b'A' + (svc_idx as u8 % 26)) as char;
        for x in 0..W {
            let style = if x < 40 { col_a } else if x < 80 { col_b } else { col_c };
            chars.push((y, x, ch, style));
        }
    }

    // Row 39: status bar
    for x in 0..W {
        chars.push((H - 1, x, if x < 10 { 'S' } else { ' ' }, status));
    }

    Buffer::from_chars(W, H, &chars)
}

/// Build a "strict UI" buffer: header, 3-column body (each with border
/// chars), footer, and status bar. Approximates Rezi's `terminal-strict-ui`.
fn strict_ui_buffer() -> Buffer {
    let header = Style::default().bold().foreground(Color::Rgb(200, 200, 200)).background(Color::Index(4));
    let footer = Style::default().foreground(Color::Index(7)).background(Color::Index(0));
    let status = Style::default().foreground(Color::Index(0)).background(Color::Index(2));
    let border = Style::default().foreground(Color::Index(8));
    let text_a = Style::default().foreground(Color::Rgb(255, 100, 100));
    let text_b = Style::default().foreground(Color::Rgb(100, 255, 100));
    let text_c = Style::default().foreground(Color::Rgb(100, 100, 255));

    let col_w = W / 3; // ~40 cols each

    let mut chars = Vec::with_capacity(W * H);

    // Row 0: header
    for x in 0..W { chars.push((0, x, '=', header)); }

    // Rows 1..37: 3-column body with vertical border chars at column edges.
    for y in 1..H - 2 {
        let ch = (b'a' + ((y - 1) as u8 % 26)) as char;
        for x in 0..W {
            if x == col_w || x == col_w * 2 {
                chars.push((y, x, '|', border));
            } else if x < col_w {
                chars.push((y, x, ch, text_a));
            } else if x < col_w * 2 {
                chars.push((y, x, ch, text_b));
            } else {
                chars.push((y, x, ch, text_c));
            }
        }
    }

    // Row 38: footer
    for x in 0..W { chars.push((H - 2, x, '-', footer)); }
    // Row 39: status bar
    for x in 0..W { chars.push((H - 1, x, ' ', status)); }

    Buffer::from_chars(W, H, &chars)
}

/// Build a "table" buffer: 40 rows × 8 columns, each column ~15 chars
/// with alternating row styles. Approximates Rezi's `terminal-table`.
fn table_buffer() -> Buffer {
    let hdr = Style::default().bold().foreground(Color::Rgb(255, 255, 255)).background(Color::Index(4));
    let even = Style::default().foreground(Color::Index(7));
    let odd = Style::default().foreground(Color::Index(15)).background(Color::Index(0));
    let sep = Style::default().foreground(Color::Index(8));

    let col_w = W / 8; // 15 chars per column

    let mut chars = Vec::with_capacity(W * H);

    for y in 0..H {
        let style = if y == 0 { hdr } else if y % 2 == 0 { even } else { odd };
        for x in 0..W {
            if x % col_w == 0 && x > 0 {
                chars.push((y, x, '|', sep));
            } else {
                let ch = (b'0' + ((x / col_w) as u8 % 10)) as char;
                chars.push((y, x, ch, style));
            }
        }
    }

    Buffer::from_chars(W, H, &chars)
}

/// Build a "virtual list" buffer: 40 visible rows of a list, each row
/// is one line of text. Approximates Rezi's `terminal-virtual-list` viewport.
fn virtual_list_buffer(offset: usize) -> Buffer {
    let style = Style::default().foreground(Color::Index(7));
    let highlight = Style::default().bold().foreground(Color::Rgb(255, 255, 0));

    let mut chars = Vec::with_capacity(W * H);

    for y in 0..H {
        let item_idx = offset + y;
        let s = if y == 0 { highlight } else { style };
        // Simulate list item text: "item NNNNN  ..."
        let label = format!("item {:>5}  ", item_idx);
        for (x, ch) in label.chars().enumerate() {
            if x < W { chars.push((y, x, ch, s)); }
        }
        // Fill rest with spaces.
        for x in label.len()..W {
            chars.push((y, x, ' ', s));
        }
    }

    Buffer::from_chars(W, H, &chars)
}

// ═══════════════════════════════════════════════════════════════════
// Terminal-level workloads (Rezi-comparable, 40×120)
// ═══════════════════════════════════════════════════════════════════

/// `terminal-rerender`: stable frame, one value changes per frame.
fn terminal_rerender(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Rgb(0, 200, 100));
    let buf1 = filled_buffer(W, H, 'x', style);

    // Change a single cell in the middle of the screen.
    let mut buf2 = buf1.clone();
    buf2[(H / 2, W / 2)] = Cell::from_char('!', style);


    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-rerender", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// `terminal-frame-fill/1-dirty-line`: 1 row changes out of 40.
fn terminal_frame_fill_1(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Index(7));
    let buf1 = filled_buffer(W, H, 'a', style);

    let mut buf2 = buf1.clone();
    let changed = Cell::from_char('Z', style);
    for x in 0..W { buf2[(20, x)] = changed; }

    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-frame-fill/1-dirty-line", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// `terminal-frame-fill/40-dirty-lines`: all 40 rows change (full repaint).
fn terminal_frame_fill_40(c: &mut Criterion) {
    let style_a = Style::default().foreground(Color::Index(1));
    let style_b = Style::default().foreground(Color::Index(2));
    let buf_a = filled_buffer(W, H, 'A', style_a);
    let buf_b = filled_buffer(W, H, 'B', style_b);

    let mut r = Rasterizer::new(W, H);
    r.render(&buf_a);
    r.clear_output();

    c.bench_function("terminal-frame-fill/40-dirty-lines", |b| {
        b.iter(|| {
            r.render(black_box(&buf_b));
            black_box(r.output());
            r.clear_output();
            r.render(&buf_a);
            r.clear_output();
        });
    });
}

/// `terminal-screen-transition`: full-screen content swap.
fn terminal_screen_transition(c: &mut Criterion) {
    let style_a = Style::default().bold().foreground(Color::Rgb(255, 0, 0)).background(Color::Index(0));
    let style_b = Style::default().italic().foreground(Color::Rgb(0, 0, 255)).background(Color::Index(7));

    let buf_a = filled_buffer(W, H, '#', style_a);
    let buf_b = filled_buffer(W, H, '.', style_b);

    let mut r = Rasterizer::new(W, H);
    r.render(&buf_a);
    r.clear_output();

    c.bench_function("terminal-screen-transition", |b| {
        b.iter(|| {
            r.render(black_box(&buf_b));
            black_box(r.output());
            r.clear_output();
            r.render(&buf_a);
            r.clear_output();
        });
    });
}

/// `terminal-full-ui`: composite dashboard with panels, status bar, 24 services.
fn terminal_full_ui(c: &mut Criterion) {
    let buf1 = full_ui_buffer();

    // Second frame: same layout, one service row changes (simulates data update).
    let mut buf2 = buf1.clone();
    let update_style = Style::default().foreground(Color::Rgb(255, 0, 0)).bold();
    for x in 0..W { buf2[(15, x)] = Cell::from_char('!', update_style); }

    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-full-ui", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// `terminal-strict-ui`: multi-panel layout (header, 3-column body, footer, status bar).
fn terminal_strict_ui(c: &mut Criterion) {
    let buf1 = strict_ui_buffer();

    // Second frame: update a few rows in each column.
    let mut buf2 = buf1.clone();
    let upd = Style::default().foreground(Color::Rgb(255, 255, 0)).bold();
    for y in [10, 20, 30] {
        for x in 0..W { buf2[(y, x)] = Cell::from_char('*', upd); }
    }

    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-strict-ui", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// `terminal-virtual-list`: 40-row viewport over a large list, scrolled by 1.
fn terminal_virtual_list(c: &mut Criterion) {
    let buf1 = virtual_list_buffer(0);
    let buf2 = virtual_list_buffer(1); // Scroll by 1 item.

    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-virtual-list", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// `terminal-table`: 40-row, 8-column data table.
fn terminal_table(c: &mut Criterion) {
    let buf1 = table_buffer();

    // Second frame: update 5 cells scattered across different rows/columns.
    let mut buf2 = buf1.clone();
    let upd = Style::default().foreground(Color::Rgb(255, 50, 50)).bold();
    for &(y, x) in &[(5, 10), (12, 30), (20, 60), (30, 90), (38, 110)] {
        buf2[(y, x)] = Cell::from_char('!', upd);
    }

    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-table", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// `terminal-fps-stream`: 12-channel streaming data update. Each channel
/// is a row region that changes every frame.
fn terminal_fps_stream(c: &mut Criterion) {
    let base_style = Style::default().foreground(Color::Index(7));
    let buf1 = filled_buffer(W, H, '.', base_style);

    // 12 channels: update rows 2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35.
    let mut buf2 = buf1.clone();
    let channel_styles = [
        Style::default().foreground(Color::Rgb(255, 0, 0)),
        Style::default().foreground(Color::Rgb(0, 255, 0)),
        Style::default().foreground(Color::Rgb(0, 0, 255)),
        Style::default().foreground(Color::Rgb(255, 255, 0)),
        Style::default().foreground(Color::Rgb(255, 0, 255)),
        Style::default().foreground(Color::Rgb(0, 255, 255)),
        Style::default().foreground(Color::Rgb(200, 100, 50)),
        Style::default().foreground(Color::Rgb(50, 200, 100)),
        Style::default().foreground(Color::Rgb(100, 50, 200)),
        Style::default().foreground(Color::Rgb(255, 128, 0)),
        Style::default().foreground(Color::Rgb(128, 0, 255)),
        Style::default().foreground(Color::Rgb(0, 128, 128)),
    ];
    for (i, &style) in channel_styles.iter().enumerate() {
        let y = 2 + i * 3;
        if y < H {
            let cell = Cell::from_char('#', style);
            for x in 0..W { buf2[(y, x)] = cell; }
        }
    }

    let mut r = Rasterizer::new(W, H);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("terminal-fps-stream", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

// ═══════════════════════════════════════════════════════════════════
// Sigil-specific workloads
// ═══════════════════════════════════════════════════════════════════

/// Cold first render (no previous frame to diff against).
fn first_render(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Rgb(0, 255, 0));
    let buffer = filled_buffer(W, H, 'X', style);

    c.bench_function("first-render", |b| {
        b.iter(|| {
            let mut r = Rasterizer::new(W, H);
            r.render(black_box(&buffer));
            black_box(r.output());
        });
    });
}

/// Identical frame: measures row-hash fast path when nothing changed.
fn identical_frame(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Index(2));
    let buffer = filled_buffer(W, H, 'A', style);

    let mut r = Rasterizer::new(W, H);
    r.render(&buffer);
    r.clear_output();

    c.bench_function("identical-frame", |b| {
        b.iter(|| {
            r.render(black_box(&buffer));
            black_box(r.output());
            r.clear_output();
        });
    });
}

/// Scroll-up optimization using DECSTBM + SU.
fn scroll_up(c: &mut Criterion) {
    let caps = Capabilities::DEFAULT | Capabilities::SCROLL_REGION | Capabilities::SCROLL;
    let style = Style::None;

    let chars1: Vec<_> = (0..H)
        .flat_map(|y| {
            let ch = (b'A' + (y as u8 % 26)) as char;
            (0..W).map(move |x| (y, x, ch, style))
        })
        .collect();
    let buf1 = Buffer::from_chars(W, H, &chars1);

    let chars2: Vec<_> = (0..H)
        .flat_map(|y| {
            let ch = if y < H - 1 {
                (b'A' + ((y + 1) as u8 % 26)) as char
            } else {
                '!'
            };
            (0..W).map(move |x| (y, x, ch, style))
        })
        .collect();
    let buf2 = Buffer::from_chars(W, H, &chars2);

    let mut r = Rasterizer::new(W, H).with_capabilities(caps);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("scroll-up", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// Invalidate + full re-render of identical content.
fn invalidate_rerender(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Index(5));
    let buffer = filled_buffer(W, H, 'R', style);

    let mut r = Rasterizer::new(W, H);
    r.render(&buffer);
    r.clear_output();

    c.bench_function("invalidate-rerender", |b| {
        b.iter(|| {
            r.invalidate();
            r.render(black_box(&buffer));
            black_box(r.output());
            r.clear_output();
        });
    });
}

/// Inline mode first render.
fn inline_first_render(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Rgb(100, 200, 50));
    let buffer = filled_buffer(W, 10, 'I', style);

    c.bench_function("inline-first-render", |b| {
        b.iter(|| {
            let mut r = Rasterizer::inline(W, 10);
            r.render(black_box(&buffer));
            black_box(r.output());
        });
    });
}

/// Inline mode diff (1 row changes).
fn inline_rerender(c: &mut Criterion) {
    let style = Style::default().foreground(Color::Rgb(100, 200, 50));
    let buf1 = filled_buffer(W, 10, 'I', style);

    let mut buf2 = buf1.clone();
    let changed = Cell::from_char('J', style);
    for x in 0..W { buf2[(5, x)] = changed; }

    let mut r = Rasterizer::inline(W, 10);
    r.render(&buf1);
    r.clear_output();

    c.bench_function("inline-rerender", |b| {
        b.iter(|| {
            r.render(black_box(&buf2));
            black_box(r.output());
            r.clear_output();
            r.render(&buf1);
            r.clear_output();
        });
    });
}

/// REP optimization for long runs of identical characters.
fn rep_long_run(c: &mut Criterion) {
    let caps = Capabilities::DEFAULT | Capabilities::REP;
    let style = Style::None;
    let buffer = filled_buffer(200, 1, 'X', style);

    c.bench_function("rep-long-run", |b| {
        b.iter(|| {
            let mut r = Rasterizer::new(200, 1).with_capabilities(caps);
            r.render(black_box(&buffer));
            black_box(r.output());
        });
    });
}

// ═══════════════════════════════════════════════════════════════════

criterion_group!(
    terminal,
    terminal_rerender,
    terminal_frame_fill_1,
    terminal_frame_fill_40,
    terminal_screen_transition,
    terminal_full_ui,
    terminal_strict_ui,
    terminal_virtual_list,
    terminal_table,
    terminal_fps_stream,
);

criterion_group!(
    sigil,
    first_render,
    identical_frame,
    scroll_up,
    invalidate_rerender,
    inline_first_render,
    inline_rerender,
    rep_long_run,
);

criterion_main!(terminal, sigil);
