use bench_helper::bench_cmd;
use bench_helper::string_equality_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(string_equality,  "lox-rs", "interpreter_main", string_equality_program!, [100000]);

criterion_group! {
    name = string_equality_benchs;
    config = Criterion::default().sample_size(10);
    targets = string_equality_bench_fn,
}

criterion_main!(string_equality_benchs);
