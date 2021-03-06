use bench_helper::bench_cmd;
use bench_helper::fib_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(fib,  "lox-rs", "interpreter_main", fib_program!, [35]);

criterion_group! {
    name = fib_benchs;
    config = Criterion::default().sample_size(10);
    targets = fib_bench_fn,
}

criterion_main!(fib_benchs);
