use async_broadcast::{InactiveReceiver, Receiver, broadcast};
use rustix::termios::{Winsize, tcgetwinsize};
use signal_hook::consts::signal::*;
use signal_hook::flag;
use signal_hook::low_level::{pipe, raise};
use smol::io::AsyncReadExt;
use smol::stream::{Stream, StreamExt};
use smol::{Async, Executor};
use std::ffi::c_int;
use std::io::{self, Read, stdin};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

/// Per-subscriber channel depth. On `Full` the message is dropped — a slow
/// consumer can lose signals rather than stall the dispatcher or grow memory.
const CHANNEL_CAPACITY: usize = 16;

pub struct Signals {
    sighup: InactiveReceiver<c_int>,
    sigterm: InactiveReceiver<c_int>,
    sigint: InactiveReceiver<c_int>,
    sigquit: InactiveReceiver<c_int>,
    sigwinch: InactiveReceiver<Winsize>,
}

impl Signals {
    pub fn new() -> &'static Self {
        pub static EXECUTOR: LazyLock<Executor<'static>> = LazyLock::new(|| Executor::new());

        static SIGNALS: LazyLock<Signals> =
            LazyLock::new(|| Signals::with(&EXECUTOR).expect("failed to register signal handlers"));

        &SIGNALS
    }
    pub fn with(ex: &Executor<'static>) -> io::Result<Self> {
        Ok(Self {
            sighup: Self::signal(ex, SIGHUP)?,
            sigterm: Self::signal(ex, SIGTERM)?,
            sigint: Self::signal(ex, SIGINT)?,
            sigquit: Self::signal(ex, SIGQUIT)?,
            sigwinch: Self::signal_with(ex, SIGWINCH, || tcgetwinsize(&stdin()).unwrap())?,
        })
    }

    fn signal(ex: &Executor<'static>, signal: c_int) -> io::Result<InactiveReceiver<c_int>> {
        Self::signal_with(ex, signal, move || signal)
    }

    fn signal_with<T: Clone + Send + 'static>(
        ex: &Executor<'static>,
        signal: c_int,
        f: impl Fn() -> T + Send + 'static,
    ) -> io::Result<InactiveReceiver<T>> {
        let (mut read, write) = Async::<UnixStream>::pair()?;
        pipe::register_raw(signal, write.as_fd().try_clone_to_owned()?)?;
        // `write` can be dropped here — signal-hook owns its cloned fd now.
        drop(write);

        let (mut tx, rx) = broadcast::<T>(16);
        tx.set_overflow(true); // drop oldest; signals are idempotent

        ex.spawn(async move {
            let mut buf = [0u8; 64];
            loop {
                match read.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let _ = tx.try_broadcast(f());
                    }
                }
            }
        })
        .detach();

        Ok(rx.deactivate())
    }

    pub fn sighup(&self) -> Receiver<c_int> {
        self.sighup.activate_cloned()
    }

    pub fn sigterm(&self) -> Receiver<c_int> {
        self.sigterm.activate_cloned()
    }

    pub fn sigint(&self) -> Receiver<c_int> {
        self.sigint.activate_cloned()
    }

    pub fn sigquit(&self) -> Receiver<c_int> {
        self.sigquit.activate_cloned()
    }

    pub fn sigwinch(&self) -> Receiver<Winsize> {
        self.sigwinch.activate_cloned()
    }
}
