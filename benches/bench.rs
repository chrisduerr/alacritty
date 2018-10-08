#[macro_use]
extern crate criterion;
extern crate alacritty;
extern crate serde_json as json;

use std::fs::File;
use std::io::{self, Read};
use std::mem;

use criterion::Criterion;

use alacritty::ansi::{self, ClearMode, Handler, Processor};
use alacritty::config::Config;
use alacritty::grid::{Grid, Scroll};
use alacritty::term::{cell::Cell, SizeInfo, Term};

fn setup_term(num_lines: usize) -> (Term, Processor, String) {
    // Create buffer with `num_lines` * "y\n"
    let mut buf = String::with_capacity(num_lines * 2);
    for _ in 0..num_lines {
        buf.push_str("y\n");
    }
    let size = SizeInfo {
        width: 100.,
        height: 100.,
        cell_width: 10.,
        cell_height: 20.,
        padding_x: 0.,
        padding_y: 0.,
    };
    let terminal = Term::new(&Default::default(), size);
    let parser = ansi::Processor::new();

    (terminal, parser, buf)
}

fn populate_term(num_lines: usize) -> Term {
    let (mut term, mut parser, buf) = setup_term(num_lines);

    for byte in buf.bytes() {
        parser.advance(&mut term, byte, &mut io::sink());
    }

    term
}

fn setup_render_iter() -> (Term, Config) {
    fn read_string(path: &str) -> String {
        let mut buf = String::new();
        File::open(path)
            .and_then(|mut f| f.read_to_string(&mut buf))
            .unwrap();
        buf
    }

    // Need some realistic grid state; using one of the ref files.
    let serialized_grid = read_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/ref/vim_large_window_scroll/grid.json"
    ));
    let serialized_size = read_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/ref/vim_large_window_scroll/size.json"
    ));

    let mut grid: Grid<Cell> = json::from_str(&serialized_grid).unwrap();
    let size: SizeInfo = json::from_str(&serialized_size).unwrap();

    let config = Config::default();

    let mut terminal = Term::new(&config, size);
    mem::swap(terminal.grid_mut(), &mut grid);

    (terminal, config)
}

/// Benchmark for the renderable cells iterator
///
/// The renderable cells iterator yields cells that require work to be
/// displayed (that is, not a an empty background cell). This benchmark
/// measures how long it takes to process the whole iterator.
///
/// When this benchmark was first added, it averaged ~78usec on my macbook
/// pro. The total render time for this grid is anywhere between ~1500 and
/// ~2000usec (measured imprecisely with the visual meter).
fn render_iter(params: (Term, Config)) {
    let (term, config) = params;
    let iter = term.renderable_cells(&config, false);
    for cell in iter {
        criterion::black_box(cell);
    }
}

fn clear_screen(mut term: Term) {
    term.clear_screen(ClearMode::All);
    criterion::black_box(term);
}

fn scroll_display(mut term: Term) {
    for _ in 0..10_000 {
        term.scroll_display(Scroll::Lines(1));
    }
    for _ in 0..10_000 {
        term.scroll_display(Scroll::Lines(-1));
    }
    criterion::black_box(term);
}

fn advance_parser(params: (Term, Processor, String)) {
    let (mut term, mut parser, buf) = params;
    for byte in buf.bytes() {
        parser.advance(&mut term, byte, &mut io::sink());
    }
    criterion::black_box((term, parser));
}

fn cell_reset(mut cell: Cell) {
    cell.reset(&Cell::default());
    criterion::black_box(cell);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("clear_screen", |b| {
        b.iter_with_large_setup(|| populate_term(10_000), clear_screen)
    });

    c.bench_function("scroll_display", |b| {
        b.iter_with_large_setup(|| populate_term(10_000), scroll_display)
    });

    c.bench_function("advance_parser", |b| {
        b.iter_with_large_setup(|| setup_term(1_000), advance_parser)
    });

    c.bench_function("render_iter", |b| {
        b.iter_with_large_setup(setup_render_iter, render_iter)
    });

    c.bench_function("cell_reset", |b| {
        b.iter_with_setup(Cell::default, cell_reset)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
