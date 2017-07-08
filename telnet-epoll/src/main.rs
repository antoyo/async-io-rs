extern crate libc;

use std::io::{Read, Write, stdin, stdout};
use std::mem;
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};

use libc::{c_int, epoll_create1, epoll_ctl, epoll_event, epoll_wait, EPOLL_CTL_ADD, EPOLLIN};

const STDIN_FD: RawFd = 0;
const FD_COUNT: usize = 2;

/// A file descriptor set is a set of fd that we want to listen for events (in this case, read
/// events).
struct FdSet {
    events: [epoll_event; FD_COUNT],
    event_count: usize,
    set: c_int,
}

impl FdSet {
    fn new() -> Self {
        // Create the fd set.
        let set = unsafe { epoll_create1(0) };
        FdSet {
            events: unsafe { mem::uninitialized() },
            event_count: 0,
            set,
        }
    }

    /// Check if a fd is in the set.
    fn is_set(&mut self, fd: RawFd) -> bool {
        for i in 0..self.event_count {
            if self.events[i].u64 == fd as u64 {
                return true;
            }
        }
        false
    }

    /// Add a fd to the set.
    fn set(&mut self, fd: RawFd) {
        let mut event = epoll_event {
            events: EPOLLIN as u32,
            u64: fd as u64,
        };
        unsafe { epoll_ctl(self.set, EPOLL_CTL_ADD, fd, &mut event) };
    }

    /// Wait for a read event to happen on any fd in the set.
    fn wait(&mut self) -> Result<(), ()> {
        let result = unsafe { epoll_wait(self.set, self.events.as_mut_ptr(), FD_COUNT as c_int, -1) };
        if result < 0 {
            Err(())
        } else {
            self.event_count = result as usize;
            Ok(())
        }
    }
}

fn main() {
    let mut stream = TcpStream::connect("google.com:80").unwrap();

    let mut fdset = FdSet::new();
    // We want to check for read events on stdin and the socket.
    fdset.set(STDIN_FD);
    fdset.set(stream.as_raw_fd());

    let mut string = String::new();
    let mut buffer = [0; 4096];

    loop {
        print!("> ");
        stdout().flush().unwrap();
        fdset.wait().unwrap();

        if fdset.is_set(STDIN_FD) {
            // Add back the socket fd as it was removed by the call to select.
            fdset.set(stream.as_raw_fd());
            stdin().read_line(&mut string).unwrap();
            write!(stream, "{}", string).unwrap();
            string.clear();
        }
        else {
            // Add back the stdin fd as it was removed by the call to select.
            fdset.set(STDIN_FD);
            stream.read(&mut buffer).unwrap();
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }
}
