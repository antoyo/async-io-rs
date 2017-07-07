extern crate libc;

use std::cmp::max;
use std::io::{Read, Write, stdin, stdout};
use std::mem;
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr::null_mut;

use libc::{fd_set, select, FD_ISSET, FD_SET, FD_ZERO};

const STDIN_FD: RawFd = 0;

/// A file descriptor set is a set of fd that we want to listen for events (in this case, read
/// events).
struct FdSet {
    highest: RawFd,
    set: fd_set,
}

impl FdSet {
    fn new() -> Self {
        let set =
            unsafe {
                let mut set = mem::uninitialized();
                // Initialize the fd set.
                FD_ZERO(&mut set);
                set
            };
        FdSet {
            highest: 0,
            set,
        }
    }

    /// Check if a fd is in the set.
    fn is_set(&mut self, fd: RawFd) -> bool {
        unsafe { FD_ISSET(fd, &mut self.set) }
    }

    /// Add a fd to the set.
    fn set(&mut self, fd: RawFd) {
        self.highest = max(self.highest, fd + 1);
        unsafe { FD_SET(fd, &mut self.set) };
    }
}

/// Wait for a read event to happen on any fd in the set.
fn wait(fdset: &mut FdSet) -> Result<u32, ()> {
    let result = unsafe { select(fdset.highest, &mut fdset.set, null_mut(), null_mut(), null_mut()) };
    if result < 0 {
        Err(())
    }
    else {
        Ok(result as u32)
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
        let count = wait(&mut fdset).unwrap();

        if count > 0 {
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
}
