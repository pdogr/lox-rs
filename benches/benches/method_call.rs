use bench_helper::bench_cmd;
use bench_helper::method_call_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(method_call,  "lox-rs", "interpreter_main", method_call_program!, [100000]);

criterion_group! {
    name = method_call_benchs;
    config = Criterion::default().sample_size(10);
    targets = method_call_bench_fn,
}

criterion_main!(method_call_benchs);
