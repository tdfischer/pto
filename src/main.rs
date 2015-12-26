extern crate rustc_serialize;
extern crate hyper;
extern crate mio;
mod irc;
mod matrix;
mod bridge;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt};
use std::thread;

struct IrcHandler {
    server: irc::streams::Server,
    clients: Vec<irc::streams::Client>,
}

impl Handler for IrcHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<IrcHandler>, token: Token, _: EventSet) {
        match token {
            SERVER => {
                match self.server.accept() {
                    Some(client) => {
                        event_loop.register(client.stream(), client.token(), EventSet::readable() | EventSet::hup(), PollOpt::edge()).unwrap();
                        self.clients.push(client);
                    },
                    None => ()
                }
            },
            _ => {
                for ref mut c in &mut self.clients {
                    if c.token() == token {
                        bridge::handle_client(c);
                    }
                }
            },
        }
    }
}

const SERVER: Token = Token(0);

fn main() {
    let addr = "127.0.0.1:8001".parse().unwrap();
    let server = irc::streams::Server::new(&addr);
    println!("Listening on 127.0.0.1:8001");
    let mut events = EventLoop::new().unwrap();
    events.register(server.listener(), SERVER, EventSet::all(), PollOpt::edge()).unwrap();
    events.run(&mut IrcHandler{
        server: server,
        clients: vec![]
    }).unwrap();
}
