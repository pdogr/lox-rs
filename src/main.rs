use std::env::args;

use jlox_rs::Runner;

fn main() {
    Runner::run(args().nth(1).as_ref());
}
