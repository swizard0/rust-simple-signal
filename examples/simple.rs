// Warning: If you run this example with `cargo run --example simple`
// and hit Ctrl-C, the executable will still be kept running using
// 100% of CPU. This is because cargo seems to kill the process even
// though you have set up Ctrl-C handler of your own.

extern crate simple_signal;
use simple_signal::{Signals, Signal};
fn main() {
    Signals::set_handler(&[Signal::Int, Signal::Term], |signals| println!("Caught: {:?}", signals));
    loop {}
}
