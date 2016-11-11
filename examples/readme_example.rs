extern crate simple_signal;

use simple_signal::{Signal};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_signals| {
        r.store(false, Ordering::SeqCst);
    });
    println!("Waiting for a signal...");
    while running.load(Ordering::SeqCst) {}
    println!("Got it! Exiting...");
}
