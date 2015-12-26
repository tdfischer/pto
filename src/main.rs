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
}

struct ClientHandler {
    client: irc::streams::Client,
}

impl ClientHandler {
    pub fn new(client: irc::streams::Client) -> Self {
        ClientHandler {
            client: client,
        }
    }
}

impl Handler for ClientHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<ClientHandler>, token: Token, _: EventSet) {
        match token {
            CLIENT =>
                bridge::handle_client(&mut self.client),
            _ => unreachable!()
        }
    }
}

impl Handler for IrcHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<IrcHandler>, token: Token, _: EventSet) {
        match token {
            SERVER => {
                match self.server.accept() {
                    Some(client) => {
                        thread::spawn(move||{
                            let mut events = EventLoop::new().unwrap();
                            events.register(client.stream(), CLIENT, EventSet::all(), PollOpt::edge()).unwrap();
                            events.run(&mut ClientHandler {
                                client: client
                            }).unwrap();
                        });
                    },
                    None => ()
                }
            },
            _ => unreachable!()
        }
    }
}

const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);

fn main() {
    let addr = "127.0.0.1:8001".parse().unwrap();
    let server = irc::streams::Server::new(&addr);
    println!("Listening on 127.0.0.1:8001");
    let mut events = EventLoop::new().unwrap();
    events.register(server.listener(), SERVER, EventSet::all(), PollOpt::edge()).unwrap();
    events.run(&mut IrcHandler{
        server: server,
    }).unwrap();
}
