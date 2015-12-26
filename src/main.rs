extern crate rustc_serialize;
extern crate hyper;
extern crate mio;
mod irc;
mod matrix;
mod bridge;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt};
use std::thread;
use bridge::Bridge;

struct IrcHandler {
    server: irc::streams::Server,
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
                            let mut bridge = Bridge::new(client);
                            bridge.run()
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
