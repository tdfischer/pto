use irc;
use matrix;
use irc::protocol::{Command,Message};
use mio::{EventLoop,Handler,Token,EventSet,PollOpt,Sender};
use std::thread;

const CLIENT: Token = Token(0);
const MATRIX: Token = Token(1);

pub struct Bridge {
    client: irc::streams::Client,
    matrix: matrix::Client,
}

impl Handler for Bridge {
    type Timeout = ();
    type Message = irc::protocol::Message;

    fn ready(&mut self, event_loop: &mut EventLoop<Bridge>, token: Token, _: EventSet) {
        match token {
            CLIENT =>
                self.handle_client(event_loop),
            _ => unreachable!()
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Bridge>, msg: Self::Message) {
        println!("Got message from matrix! {:?}", msg);
        self.client.send(&msg);
    }
}

unsafe impl Sync for Bridge{}

impl Bridge {
    pub fn new(client: irc::streams::Client) -> Self {
        Bridge {
            client: client,
            matrix: matrix::Client::new()
        }
    }

    pub fn run(&mut self) {
        let mut events = EventLoop::new().unwrap();
        events.register(self.client.stream(), CLIENT, EventSet::all(), PollOpt::edge()).unwrap();
        events.run(self).unwrap();
    }

    fn handle_client(&mut self, events: &mut EventLoop<Bridge>) {
        loop {
            match self.client.read_message() {
                None => return,
                Some(message) => {
                    println!("Got a message! {:?}", message);
                    match message.command {
                        Command::Pass => {
                            self.client.auth.set_password(message.args[0].clone())
                        }
                        Command::Nick => {
                            self.client.set_nickname(message.args[0].clone());
                        },
                        Command::User => {
                            println!("User logged in: {}", message.args[0]);
                            self.client.auth.set_username(message.args[0].clone());
                            let auth = self.client.auth.consume();
                            match (auth.username, auth.password) {
                                (Some(username), Some(password)) => {
                                    self.matrix.login(username.trim(), password.trim());
                                    self.client.welcome("Welcome!");
                                    self.matrix.sync(events.channel());
                                    self.matrix.pollAsync(events.channel());
                                    println!("Logged in {:?}", username);
                                },
                                _ => panic!("Username and/or password missing")
                            };
                        },
                        Command::Join => {
                            self.client.join(&message.args[0]);
                        },
                        Command::Ping => {
                            self.client.pong();
                        },
                        Command::Quit => {
                            // FIXME: Logout of matrix and exit thread
                            return;
                        },
                        _ =>
                            println!("unhandled {:?}", message)
                    }
                }
            }
        }
    }
}

