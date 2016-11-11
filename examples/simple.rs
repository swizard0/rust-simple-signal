// WARNING: If you run this example with `cargo run --example simple` and press Ctrl-C, the executable will still be kept running with high CPU usage. This is because cargo seems to kill the process even though you have set up Ctrl-C handler of your own.

extern crate simple_signal;

use simple_signal::{Signal};
use std::thread;

fn main() {
    simple_signal::set_handler(&[Signal::Int, Signal::Term], |signals| println!("Caught: {:?}", signals));
    loop {
        thread::yield_now();
    }
}
