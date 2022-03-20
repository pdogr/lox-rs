use bench_helper::bench_cmd;
use bench_helper::binary_trees_program;
use bench_helper::tif;
use bench_helper::CommandUnderTest;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use paste::paste;

use benches::generate_bench;

generate_bench!(binary_trees,  "lox-rs", "interpreter_main", binary_trees_program!, [14]);

criterion_group! {
    name = binary_trees_benchs;
    config = Criterion::default().sample_size(10);
    targets = binary_trees_bench_fn,
}

criterion_main!(binary_trees_benchs);
