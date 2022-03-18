#[macro_export]
macro_rules! generate_bench {
    ($bench_name: ident, $impl_name: literal, $binary_name: literal, $input_program:tt !, $($e: tt)*) => {
        paste! {
                fn [<$bench_name _bench_fn >](c: &mut Criterion) {
                    let mut group = c.benchmark_group(stringify!($bench_name));
                    let cmd = CommandUnderTest::new($binary_name.to_string());
                    dbg!(&cmd);
                    for num_iter in $(
                        $e
                    )* {
                        let input =
                            $input_program!(num_iter = num_iter);
                        let fin = tif(input);
                        group.bench_with_input(BenchmarkId::new($impl_name, num_iter), &fin, |b, fin| {
                            b.iter(|| bench_cmd(cmd.clone_cmd(), &[fin.path().to_str().unwrap()]));
                        });
                    }

            }
        }
    };
}
