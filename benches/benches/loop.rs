use bench_helper::bench_cmd;
use bench_helper::loop_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(loop,  "lox-rs", "interpreter_main", loop_program!, [100,1000]);

criterion_group! {
    name = loop_benches;
    config = Criterion::default().sample_size(20);
    targets = loop_bench_fn,
}

criterion_main!(loop_benches);
