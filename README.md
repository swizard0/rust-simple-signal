# simple-signal
A simple wrapper for handling Unix process signals.

## Example Usage
```rust
extern crate simple_signal;
use simple_signal::{Signals, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    Signals::set_handler(&[Signal::Int, Signal::Term], move |_signals| {
        r.store(false, Ordering::SeqCst);
    });
    println!("Waiting for a signal...");
    while running.load(Ordering::SeqCst) {}
    println!("Got it! Exiting...");
}
```

#### Try the example yourself
`cargo run --example readme_example`

## Building
If you're using a nightly compiler, I suggest building with `cargo build --features nightly` to avoid the dependency on *lazy_static*. On stable and beta compilers, just run `cargo build`.
