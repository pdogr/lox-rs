use bench_helper::bench_cmd;
use bench_helper::tif;
use bench_helper::trees_program;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(trees,  "lox-rs", "interpreter_main", trees_program!, [8]);

criterion_group! {
    name = trees_benchs;
    config = Criterion::default().sample_size(10);
    targets = trees_bench_fn,
}

criterion_main!(trees_benchs);
