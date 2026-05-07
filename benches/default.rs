use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
#[allow(unused_imports)] // Allow commenting out code
use rust_chess::types::{
    board::{PyBoard, PyRepetitionDetectionMode},
    board_batch::PyBoardBatch,
    r#move::PyMove,
};

fn board_benchmark(c: &mut Criterion) {
    // c.bench_function("new_board", |b| {
    //     b.iter(|| PyBoard::new(black_box(None), black_box(PyRepetitionDetectionMode::Full)))
    // });

    let board = PyBoard::new(None, PyRepetitionDetectionMode::Full).unwrap();

    // c.bench_function("make_move_new", |b| {
    //     b.iter(|| {
    //         board.make_move_new(
    //             black_box(PyMove::from_uci("e2e4")).unwrap(),
    //             black_box(true),
    //         )
    //     })
    // });

    c.bench_function("display", |b| b.iter(|| board.display(black_box(false))));
    c.bench_function("display_unicode", |b| {
        b.iter(|| board.display_unicode(black_box(false), black_box(true)));
    });
    c.bench_function("display_color", |b| {
        b.iter(|| board.display_color(black_box(false), black_box(false)));
    });
}

fn board_batch_benchmark(c: &mut Criterion) {
    c.bench_function("new_board_batch", |b| {
        b.iter(|| PyBoardBatch::new(black_box(25), black_box(PyRepetitionDetectionMode::Full)));
    });

    let board = PyBoardBatch::new(25, PyRepetitionDetectionMode::Full);
    
    c.bench_function("get_fens", |b| {
        b.iter(|| board.get_fens());
    });

    // c.bench_function("make_move_new", |b| {
    //     b.iter(|| {
    //         board.make_move_new(
    //             black_box(PyMove::from_uci("e2e4")).unwrap(),
    //             black_box(true),
    //         )
    //     })
    // });

    c.bench_function("display_batch", |b| {
        b.iter(|| board.display(black_box(false)));
    });
    c.bench_function("display_unicode_batch", |b| {
        b.iter(|| board.display_unicode(black_box(false), black_box(true)));
    });
    c.bench_function("display_color_batch", |b| {
        b.iter(|| board.display_color(black_box(false), black_box(false)));
    });

    // c.bench_function("display_tiled_batch", |b| {
    //     b.iter(|| board.display_tiled(black_box(false)))
    // });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(1000).measurement_time(std::time::Duration::from_secs(10));
    targets = board_benchmark, board_batch_benchmark
}
criterion_main!(benches);

// use criterion::{BenchmarkGroup, BenchmarkId};

// fn bench_repetition_modes(c: &mut Criterion) {
//     let mut group = c.benchmark_group("repetition_detection");

//     for mode in [
//         PyRepetitionDetectionMode::Full,
//         PyRepetitionDetectionMode::None,
//     ] {
//         group.bench_with_input(
//             BenchmarkId::new("new_board", format!("{:?}", mode)),
//             &mode,
//             |b, mode| {
//                 b.iter(|| PyBoard::new(black_box(None), black_box(*mode)))
//             },
//         );
//     }

//     group.finish();
// }

// use criterion::{BenchmarkGroup, BenchmarkId};

// fn bench_repetition_modes(c: &mut Criterion) {
//     let mut group = c.benchmark_group("repetition_detection");

//     for mode in [
//         PyRepetitionDetectionMode::Full,
//         PyRepetitionDetectionMode::None,
//     ] {
//         group.bench_with_input(
//             BenchmarkId::new("new_board", format!("{:?}", mode)),
//             &mode,
//             |b, mode| {
//                 b.iter(|| PyBoard::new(black_box(None), black_box(*mode)))
//             },
//         );
//     }

//     group.finish();
// }
