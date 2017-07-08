extern crate mio;

use std::io::{Read, Write, stdin, stdout};
use std::os::unix::io::RawFd;

use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::tcp::TcpStream;
use mio::unix::EventedFd;

// A token is used to identify an event.
const SOCKET: Token = Token(0);
const STDIN_FD: RawFd = 0;
const STDIN_TOKEN: Token = Token(1);

fn main() {
    let addr = "172.217.4.238:80".parse().unwrap();
    let mut stream = TcpStream::connect(&addr).unwrap();

    // Create an object to monitor events.
    let poll = Poll::new().unwrap();

    // Register for read event on the socket.
    poll.register(&stream, SOCKET, Ready::readable(), PollOpt::edge()).unwrap();

    // Register for read event on stdin.
    poll.register(&EventedFd(&STDIN_FD), STDIN_TOKEN, Ready::readable(), PollOpt::edge()).unwrap();

    // Create a collection where the ready events will go.
    let mut events = Events::with_capacity(2);
    let mut string = String::new();
    let mut buffer = [0; 4096];

    loop {
        print!("> ");
        stdout().flush().unwrap();

        // Wait for events.
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SOCKET => {
                    stream.read(&mut buffer).unwrap();
                    print!("{}", String::from_utf8_lossy(&buffer));
                },
                STDIN_TOKEN => {
                    stdin().read_line(&mut string).unwrap();
                    write!(stream, "{}", string).unwrap();
                    string.clear();
                },
                _ => unreachable!(),
            }
        }
    }
}
