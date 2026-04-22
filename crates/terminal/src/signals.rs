use async_broadcast::{InactiveReceiver, Receiver, broadcast};
use async_channel::bounded;
use rustix::termios::{Winsize, tcgetwinsize};
use signal_hook::consts::signal::*;
use signal_hook::flag;
use signal_hook::low_level::{pipe, raise};
use signal_hook_async_std::Signals as S;
use smol::{Async, Executor, future, io::*, lock::*, stream::*};
use std::ffi::c_int;
use std::io::{self, Read, stdin};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::unix::net::UnixStream;
use std::panic::catch_unwind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;

/// Per-subscriber channel depth. On `Full` the message is dropped — a slow
/// consumer can lose signals rather than stall the dispatcher or grow memory.
const CHANNEL_CAPACITY: usize = 16;

pub struct Signals {
    sighup: InactiveReceiver<()>,
    sigterm: InactiveReceiver<()>,
    sigint: InactiveReceiver<()>,
    sigquit: InactiveReceiver<()>,
    sigwinch: InactiveReceiver<Winsize>,
}

impl Signals {
    pub fn global() -> &'static Self {
        static EXECUTOR: OnceCell<Executor<'_>> = OnceCell::new();
        static SIGNALS: LazyLock<Signals> =
            LazyLock::new(|| Signals::new(executor()).expect("failed signals"));

        fn executor() -> &'static Executor<'static> {
            EXECUTOR.get_or_init_blocking(|| {
                thread::Builder::new()
                    .name(format!("signals-thread"))
                    .spawn(|| {
                        loop {
                            catch_unwind(|| {
                                smol::block_on(executor().run(future::pending::<()>()))
                            })
                            .ok();
                        }
                    })
                    .expect("cannot spawn executor thread");

                // Prevent spawning another thread by running the process driver on this thread.
                let ex = Executor::new();
                ex.spawn(async_process::driver()).detach();
                ex
            })
        }

        &SIGNALS
    }

    pub fn new(ex: &Executor<'static>) -> io::Result<Self> {
        Ok(Self {
            sighup: Self::signal(ex, SIGHUP)?,
            sigterm: Self::signal(ex, SIGTERM)?,
            sigint: Self::signal(ex, SIGINT)?,
            sigquit: Self::signal(ex, SIGQUIT)?,
            sigwinch: Self::signal_with(
                ex,
                SIGWINCH,
                || tcgetwinsize(stdin()).map_err(Into::into),
                8,
            )?,
        })
    }

    pub fn sighup(&self) -> Receiver<()> {
        self.sighup.activate_cloned()
    }

    pub fn sigterm(&self) -> Receiver<()> {
        self.sigterm.activate_cloned()
    }

    pub fn sigint(&self) -> Receiver<()> {
        self.sigint.activate_cloned()
    }

    pub fn sigquit(&self) -> Receiver<()> {
        self.sigquit.activate_cloned()
    }

    pub fn sigwinch(&self) -> Receiver<Winsize> {
        self.sigwinch.activate_cloned()
    }

    pub fn shutdown(&self) -> impl Stream<Item = c_int> {
        let sighup = self.sighup().map(|_| SIGHUP);
        let sigterm = self.sigterm().map(|_| SIGTERM);
        let sigint = self.sigint().map(|_| SIGINT);
        let sigquit = self.sigquit().map(|_| SIGQUIT);

        sighup.or(sigterm).or(sigint).or(sigquit)
    }

    fn signal(ex: &Executor<'static>, signal: c_int) -> io::Result<InactiveReceiver<()>> {
        Self::signal_with(ex, signal, || Ok(()), 8)
    }

    fn signal_with<T: Clone + Send + 'static>(
        ex: &Executor<'static>,
        signal: c_int,
        value: impl Fn() -> io::Result<T> + Send + 'static,
        size: usize,
    ) -> io::Result<InactiveReceiver<T>> {
        let (mut srx, stx) = Async::<UnixStream>::pair()?;
        pipe::register_raw(signal, stx.as_fd().try_clone_to_owned()?)?;
        // `write` can be dropped here — signal-hook owns its cloned fd now.
        drop(stx);

        let (mut tx, rx) = broadcast::<T>(size);
        tx.set_overflow(true); // drop oldest; signals are idempotent
        tx.set_await_active(false);
        ex.spawn(async move {
            loop {
                match srx.read(&mut [0]).await {
                    Ok(0) => break,
                    Ok(_) => match value() {
                        Ok(t) => {
                            let _ = tx.try_broadcast(t);
                        }
                        Err(e) => eprintln!("signal handler failed: {e}"),
                    },
                    Err(e) => eprintln!("signal handler failed: {e}"),
                }
            }
        })
        .detach();

        Ok(rx.deactivate())
    }
}
