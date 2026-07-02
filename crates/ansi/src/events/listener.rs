use std::collections::VecDeque;
use std::fs::File;
use polling::{Event as PollingEvent, Events, Poller};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use rustix::{termios::isatty};
use std::time::{Duration};
use filedescriptor::FileDescriptor;
use crate::parser::{Handler, Parser};
use super::Event;

pub struct InputReader {
    poller: Poller,
    events: Events,
    tty: FileDescriptor,
    tty_handle: usize,
    queue: VecDeque<Event>,
    parser: Parser,
    // Optionally: signal_fd for SIGWINCH
}

impl InputReader {
    pub fn new() -> io::Result<Self> {
        let stdin = rustix::stdio::stdin();
        let mut tty = FileDescriptor::new(if isatty(stdin) {
           stdin.as_raw_fd()
        } else {
            let dev_tty = File::options().read(true).write(true).open("/dev/tty")?;
            dev_tty.as_raw_fd()
        });

        tty.set_non_blocking(true).map_err(|_| io::ErrorKind::WouldBlock)?;

        let poller = Poller::new()?;
        let tty_handle = tty.as_raw_fd() as usize;
        unsafe { poller.add(&tty, PollingEvent::readable(tty_handle))?; }

        Ok(Self {
            poller,
            tty,
            parser: Parser::new(),
            tty_handle,
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
            if ev.key == self.tty_handle {
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
            // Handle signal / resize keys similarly (omitted for brevity)
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
impl Iterator for InputReader {
    type Item = io::Result<Event>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.poll(Some(Duration::ZERO)) {
            Ok(true) => self.queue.pop_front().map(Ok),
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        }
    }
}