use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;

use lox_interpreter::Interpreter;
use lox_interpreter::Lexer;
use lox_interpreter::Parser;
use lox_interpreter::Resolver;

use bench_helper::loop_program;
use bench_helper::TestWriter;

use benches::NUM_ITERS;

fn loop_bench_fn(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop");
    for loop_size in NUM_ITERS {
        let input = loop_program!(num_iter = loop_size);
        group.bench_with_input(
            BenchmarkId::from_parameter(loop_size),
            &input,
            |b, input| {
                let fake_stdout = TestWriter::new();
                b.iter(|| {
                    let lexer = Lexer::new(input.chars()).unwrap();
                    let tokens: std::result::Result<Vec<_>, _> = lexer.into_iter().collect();
                    let tokens = tokens.expect("lexing error");
                    let stmts = Parser::new(tokens.into_iter())
                        .program()
                        .expect("parsing error");
                    let mut interpreter = Interpreter::new(fake_stdout.clone());
                    let mut resolver = Resolver::new();
                    resolver
                        .resolve(&stmts, &mut interpreter)
                        .expect("variable resolution error");
                    interpreter.run_many(stmts).expect("interpret error");
                });
            },
        );
    }
}

criterion_group! {
name = loop_benches;
config = Criterion::default();
targets = loop_bench_fn,
}

criterion_main!(loop_benches);
