use std::collections::VecDeque;
use std::fs::File;
use polling::{Event as PollingEvent, Events, Poller};
use std::io::{self, Read};
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::net::UnixStream;
use rustix::{termios::isatty};
use std::time::{Duration};
use filedescriptor::FileDescriptor;
use crate::parser::{Handler, Parser};
use super::Event;
use signal_hook::low_level::pipe;
use geometry::Size;

const TTY: usize = 1;
const SIGWINCH: usize = 2;

pub struct Listener {
    poller: Poller,
    events: Events,
    tty: FileDescriptor,
    signals: UnixStream,
    queue: VecDeque<Event>,
    parser: Parser,
}

impl Listener {
    pub fn new() -> io::Result<Self> {
        let stdin = rustix::stdio::stdin();
        let poller = Poller::new()?;

        // TTY
        let mut tty = FileDescriptor::new(if isatty(stdin) {
            stdin.as_raw_fd()
        } else {
            let dev_tty = File::options().read(true).write(true).open("/dev/tty")?;
            dev_tty.as_raw_fd()
        });
        tty.set_non_blocking(true).map_err(|_| io::ErrorKind::WouldBlock)?;


        // SIGWINCH
        let (signals, signals_tx) = UnixStream::pair()?;
        signals.set_nonblocking(true)?;

        pipe::register_raw(signal_hook::consts::signal::SIGWINCH, signals_tx.as_fd().try_clone_to_owned()?)?;
        drop(signals_tx);


        unsafe {
            poller.add(&tty, PollingEvent::readable(TTY))?;
            poller.add(&signals, PollingEvent::readable(SIGWINCH))?;
        }

        Ok(Self {
            poller,
            tty,
            signals,
            parser: Parser::new(),
            events: Events::new(),
            queue: Default::default(),
        })
    }

    /// Wait for events up to `timeout`. Returns `Ok(true)` if events are ready.
    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<bool> {
        struct InputReaderHandler;
        impl Handler for InputReaderHandler {}

        // Drain already‑parsed events first
        if !self.queue.is_empty() { return Ok(true); }

        self.events.clear();
        self.poller.wait(&mut self.events, timeout)?;

        for ev in self.events.iter() {
            match ev.key {
                TTY => {
                    // Read all available bytes without blocking
                    let mut buf = [0u8; 1024];
                    loop {
                        match self.tty.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => self.parser.advance(&mut InputReaderHandler, &buf[..n]),
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(e) => return Err(e),
                        };
                    }
                }
                SIGWINCH => {
                    // Drain the pipe (consume the byte that was written)
                    let mut buf = [0u8; 16];
                    while let Ok(n) = self.signals.read(&mut buf) {
                        if n == 0 { break; }
                    }

                    match rustix::termios::tcgetwinsize(&self.tty) {
                        Ok(size) => {
                            self.queue.push_back(Event::Resize(Size::new(size.ws_col, size.ws_row)));
                        },
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                },
                _ => {}
            }
        }

        // Convert parsed bytes into events
        // while let Some(event) = self.parser.try_pop() {
        //     self.queue.push_back(event);
        // }

        Ok(!self.queue.is_empty())
    }

    /// Block until an event is available.
    pub fn read(&mut self) -> io::Result<Event> {
        loop {
            if let Some(e) = self.queue.pop_front() {
                return Ok(e);
            }
            self.poll(None)?;
        }
    }

}
// Iterator ergonomics: non‑blocking, “give me an event if you have one now”
impl Iterator for Listener {
    type Item = io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.poll(Some(Duration::ZERO)) {
            Ok(true) => self.queue.pop_front().map(Ok),
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        }
    }
}