//! A simple easy to use wrapper around unix signals.

#![cfg_attr(feature="nightly", feature(static_condvar))]
#![cfg_attr(feature="nightly", feature(static_mutex))]

#[cfg(feature="stable")]
#[macro_use]
extern crate lazy_static;

use std::sync::atomic::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Signal {
    Hup,
    Int,
    Quit,
    Ill,
    Abrt,
    Fpe,
    Kill,
    Segv,
    Pipe,
    Alrm,
    Term,
}

#[cfg(feature="nightly")]
mod features {
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
    use std::sync::{StaticCondvar, CONDVAR_INIT, StaticMutex, MUTEX_INIT};
    pub static CVAR: StaticCondvar = CONDVAR_INIT;
    pub static MUTEX: StaticMutex = MUTEX_INIT;
    pub static MASK: AtomicUsize = ATOMIC_USIZE_INIT;
}
#[cfg(not(feature="nightly"))]
mod features {
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
    use std::sync::{Condvar, Mutex};
    lazy_static! {
        pub static ref CVAR: Condvar = Condvar::new();
        pub static ref MUTEX: Mutex<bool> = Mutex::new(false);
    }
    pub static MASK: AtomicUsize = ATOMIC_USIZE_INIT;
}
use self::features::*;

#[cfg(unix)]
mod platform {
    extern crate libc;
    use self::libc::{c_int, signal, sighandler_t};
    use self::libc::{SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGABRT, SIGFPE, SIGKILL, SIGSEGV, SIGPIPE, SIGALRM, SIGTERM};
    use std::sync::atomic::Ordering;
    use super::Signal;

    pub extern "C" fn handler(sig: c_int) {
        let mask = match sig {
            SIGHUP => 1,
            SIGINT => 2,
            SIGQUIT => 4,
            SIGILL => 8,
            SIGABRT => 16,
            SIGFPE => 32,
            SIGKILL => 64,
            SIGSEGV => 128,
            SIGPIPE => 256,
            SIGALRM => 512,
            SIGTERM => 1024,
            _ => return,
        };

        loop {
            let prev_mask = super::features::MASK.load(Ordering::Relaxed);
            let new_mask = prev_mask | mask;
            if super::features::MASK.compare_and_swap(prev_mask, new_mask, Ordering::Relaxed) == new_mask {
                break
            }
        }
        super::features::CVAR.notify_all();
    }
    #[inline]
    pub unsafe fn set_os_handler(sig: Signal) {
        let os_sig = match sig {
            Signal::Hup => SIGHUP,
            Signal::Int => SIGINT,
            Signal::Quit => SIGQUIT,
            Signal::Ill => SIGILL,
            Signal::Abrt => SIGABRT,
            Signal::Fpe => SIGFPE,
            Signal::Kill => SIGKILL,
            Signal::Segv => SIGSEGV,
            Signal::Pipe => SIGPIPE,
            Signal::Alrm => SIGALRM,
            Signal::Term => SIGTERM,
        };

        signal(os_sig, ::std::mem::transmute::<_, sighandler_t>(handler as extern "C" fn(_)));
    }
}

use self::platform::*;

pub struct Signals;
impl Signals {
    /// Sets up some signals handler
    /// # Example
    /// ```
    /// use simple_signal::{Signals, Signal};
    /// Signals::set_handler(&[Signal::Int, Signal::Term], |signals| println!("Caught: {:?}", signals));
    /// ```
    pub fn set_handler<F>(signals: &[Signal], user_handler: F) where F: Fn(&[Signal]) + 'static + Send {
        for &signal in signals.iter() {
            unsafe { set_os_handler(signal) }
        }
        ::std::thread::spawn(move || {
            let mut signals = Vec::new();
            loop {
                let mask = MASK.load(Ordering::Relaxed);
                if mask == 0 {
                    let _ = CVAR.wait(MUTEX.lock().unwrap());
                    continue
                }
                signals.clear();
                if mask & 1 != 0 { signals.push(Signal::Hup) }
                if mask & 2 != 0 { signals.push(Signal::Int) }
                if mask & 4 != 0 { signals.push(Signal::Quit) }
                if mask & 8 != 0 { signals.push(Signal::Ill) }
                if mask & 16 != 0 { signals.push(Signal::Abrt) }
                if mask & 32 != 0 { signals.push(Signal::Fpe) }
                if mask & 64 != 0 { signals.push(Signal::Kill) }
                if mask & 128 != 0 { signals.push(Signal::Segv) }
                if mask & 256 != 0 { signals.push(Signal::Pipe) }
                if mask & 512 != 0 { signals.push(Signal::Alrm) }
                if mask & 1024 != 0 { signals.push(Signal::Term) }
                MASK.store(0, Ordering::Relaxed);
                user_handler(&signals);
            }
        });
    }
}

#[cfg(test)]
mod test {
    extern crate libc;
    use std::sync::mpsc::sync_channel;
    use self::libc::c_int;
    use self::libc::{SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGABRT, SIGFPE, SIGKILL, SIGSEGV, SIGPIPE, SIGALRM, SIGTERM};
    use super::{Signal, Signals};
    use super::platform::handler;

    fn to_os_signal(signal: Signal) -> c_int {
        match signal {
            Signal::Hup => SIGHUP,
            Signal::Int => SIGINT,
            Signal::Quit => SIGQUIT,
            Signal::Ill => SIGILL,
            Signal::Abrt => SIGABRT,
            Signal::Fpe => SIGFPE,
            Signal::Kill => SIGKILL,
            Signal::Segv => SIGSEGV,
            Signal::Pipe => SIGPIPE,
            Signal::Alrm => SIGALRM,
            Signal::Term => SIGTERM,
        }
    }

    #[test]
    fn all_signals() {
        let signals = [Signal::Hup, Signal::Int, Signal::Quit, Signal::Abrt, Signal::Term];
        let (tx, rx) = sync_channel(0);
        Signals::set_handler(&signals, move |signals| tx.send(signals.to_owned()).unwrap());
        // check one by one
        for &signal in signals.iter() {
            handler(to_os_signal(signal));
            assert_eq!(rx.recv().unwrap(), vec![signal]);
        }
        // check all simultaneously
        for &signal in signals.iter() {
            handler(to_os_signal(signal))
        }
        assert_eq!(rx.recv().unwrap(), signals.to_owned());
    }
}
