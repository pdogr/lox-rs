# Rust implementation of Lox interpreter

An implementation of a tree-walking interpreter for [The Lox Langauge](https://craftinginterpreters.com/the-lox-language.html).
A tree-walking interpreter parses the input into an AST and "walks" over each node consuming it in process. The interpreter may need do specific operations depending on the type of AST node. These are much simpler as one only needs to write the frontend till the IR and then handle actions for each AST node type. The code would also work on all systems irrespective of the host architecture. 
Written in safe rust there is "room for improvement" by using unsafe rust. 

The runtime performance is considerably worse than a bytecode interpreter, hence you would not find these in production though there are exceptions. Ruby & R used to have tree walking interpreters (why is it always these languages starting with R).

Possible reasons for this 
- Poor cache locality: Always a problem when using heap pointers. You never know where the next memory is allocated unless you game the allocator. 
- Non linear memory access pattern: This is almost similar to the previous reason. To compute an expression you would need to recurse over its subexpressions. Then there are arbitary jumps out of a loop, branches. 
- Bigger AST nodes: I don't know how much this affects performance, but having a small AST node definitely helps. Keep it in the cache kids.
- Retouching the same node: A tree-walking interpreter would touch a node multiple times. This can cause a lot of cache misses if say the looping node is quite big (though don't have concrete numbers for this).

## Examples
### Fibonacci
```cpp
fun fib(n) {
  if (n < 2) return n;
  return fib(n - 2) + fib(n - 1);
}

print fib(30);
```
```
cargo run --release fibonacci.lox
```
This prints `832040` on my machine. 

### Tree 
```cpp
class Tree {
  init(depth) {
    this.depth = depth;
    if (depth > 0) {
      this.a = Tree(depth - 1);
      this.b = Tree(depth - 1);
      this.c = Tree(depth - 1);
      this.d = Tree(depth - 1);
      this.e = Tree(depth - 1);
    }
  }

  walk() {
    if (this.depth == 0) return 0;
    return this.depth
        + this.a.walk()
        + this.b.walk()
        + this.c.walk()
        + this.d.walk()
        + this.e.walk();
  }
}

var tree = Tree(8);
for (var i = 0; i < 100; i = i + 1) {
  if (tree.walk() != 122068) print "Error";
}
```
```
cargo run --release tree.lox
```

## Clone and build
```shell
git clone https://github.com/pdogr/lox-rs.git
cd lox-rs
cargo build --release
```

## Launch repl
```shell
$ cargo run --release  
    Finished release [optimized] target(s) in 0.03s
    Running `target/release/interpreter_main`
> print "hello world!";
"hello world!"
```

## Compile and run a script 
```shell
cargo run --release test.lox
```
## Running tests
Tests have been added to check the sanity of the implementation. The [test suite](https://github.com/munificent/craftinginterpreters/tree/master/test) included in the book has been added and made to work with rust. 

### Unit tests
```shell
cargo test --lib
cargo test --release --lib
```
### Integration tests

The intergration test files are in `data/` directory. To run the integration tests the binary needs to be built in that respective mode. Hence `cargo build` or `cargo build --release` need to be run before running the integration test. 

```shell
# Run integration test in debug mode
cargo build 
cargo test --all

# Run integration test in release mode
cargo build --release
cargo test --release --all
```
## Benchmarking
The benchmarks are in [benches](https://github.com/pdogr/lox-rs/tree/main/benches) directory and some utilities are in [bench_helpers](https://github.com/pdogr/lox-rs/tree/main/bench_helper). 

List of benchmarks
- binary_trees
- equality
- instantiation
- invocation
- loop
- method_call
- fib
- properties
- string_equality
- trees
- zoo

### Run benchmark
To run a benchmark we use `cargo bench`. The benchmark run artifacts will be saved in `benches/target/criterion/<benchmark_name>`. 
```shell
cd benches
cargo bench --release --bench <benchmark_name>
# Run all benchmarks
cargo bench --all
```

