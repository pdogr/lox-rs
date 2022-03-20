use bench_helper::bench_cmd;
use bench_helper::equality_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(equality,  "lox-rs", "interpreter_main", equality_program!, [10000000]);

criterion_group! {
    name = equality_benchs;
    config = Criterion::default().sample_size(10);
    targets = equality_bench_fn,
}

criterion_main!(equality_benchs);
