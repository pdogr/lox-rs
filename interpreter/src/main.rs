use std::env::args;

use lox_interpreter::Runner;

fn main() {
    Runner::run(args().nth(1).as_ref());
}
