extern crate rustc_serialize;
extern crate hyper;
extern crate mio;
extern crate env_logger;
#[macro_use]
extern crate log;
mod irc;
mod matrix;
mod bridge;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt};
use std::thread;
use bridge::Bridge;
use std::env;

struct IrcHandler {
    server: irc::streams::Server,
    url: String
}

impl Handler for IrcHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<IrcHandler>, token: Token, _: EventSet) {
        match token {
            SERVER => {
                match self.server.accept() {
                    Some(client) => {
                        let mut bridge = Bridge::new(client, self.url.trim());
                        thread::spawn(move||{
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
    env_logger::init().unwrap();
    let addr = "127.0.0.1:8001".parse().unwrap();
    let url =  env::args().nth(1).unwrap();
    let server = irc::streams::Server::new(&addr);
    info!("Listening on 127.0.0.1:8001");
    let mut events = EventLoop::new().unwrap();
    events.register(server.listener(), SERVER, EventSet::all(), PollOpt::edge()).unwrap();
    events.run(&mut IrcHandler{
        server: server,
        url: url
    }).unwrap();
}
