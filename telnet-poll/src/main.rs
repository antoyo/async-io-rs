extern crate libc;

use std::io::{Read, Write, stdin, stdout};
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};

use libc::{poll, pollfd, POLLIN};

const STDIN_FD: RawFd = 0;

/// A file descriptor set is a set of fd that we want to listen for events (in this case, read
/// events).
struct FdSet {
    len: usize,
    set: Vec<pollfd>,
}

impl FdSet {
    fn new() -> Self {
        FdSet {
            len: 0,
            set: vec![],
        }
    }

    /// Check if a read event happened on the specified fd.
    fn is_set(&self, fd: RawFd) -> bool {
        for item in &self.set {
            if item.fd == fd && item.revents & POLLIN == POLLIN {
                return true;
            }
        }
        false
    }

    /// Add a fd to the set.
    fn add(&mut self, fd: RawFd) {
        self.len += 1;
        self.set.push(pollfd {
            fd,
            events: POLLIN,
            revents: 0,
        });
    }
}

/// Wait for a read event to happen on any fd in the set.
fn wait(fdset: &mut FdSet) -> Result<u32, ()> {
    let result = unsafe { poll(fdset.set.as_mut_ptr(), fdset.len as u64, -1)};
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
    fdset.add(STDIN_FD);
    fdset.add(stream.as_raw_fd());

    let mut string = String::new();
    let mut buffer = [0; 4096];

    loop {
        print!("> ");
        stdout().flush().unwrap();
        let count = wait(&mut fdset).unwrap();

        if count > 0 {
            if fdset.is_set(STDIN_FD) {
                stdin().read_line(&mut string).unwrap();
                write!(stream, "{}", string).unwrap();
                string.clear();
            }
            else {
                stream.read(&mut buffer).unwrap();
                print!("{}", String::from_utf8_lossy(&buffer));
            }
        }
    }
}
