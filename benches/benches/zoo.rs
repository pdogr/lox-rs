use bench_helper::bench_cmd;
use bench_helper::tif;
use bench_helper::zoo_program;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(zoo,  "lox-rs", "interpreter_main", zoo_program!, [10000000]);

criterion_group! {
    name = zoo_benchs;
    config = Criterion::default().sample_size(10);
    targets = zoo_bench_fn,
}

criterion_main!(zoo_benchs);
