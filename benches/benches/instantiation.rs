use bench_helper::bench_cmd;
use bench_helper::instantiation_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(instantiation,  "lox-rs", "interpreter_main", instantiation_program!, [500000]);

criterion_group! {
    name = instantiation_benchs;
    config = Criterion::default().sample_size(10);
    targets = instantiation_bench_fn,
}

criterion_main!(instantiation_benchs);
