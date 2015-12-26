use irc;
use irc::protocol::Command;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt};

const CLIENT: Token = Token(1);

pub struct Bridge {
    client: irc::streams::Client
}

impl Handler for Bridge {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Bridge>, token: Token, _: EventSet) {
        match token {
            CLIENT =>
                handle_client(&mut self.client),
            _ => unreachable!()
        }
    }
}

impl Bridge {
    pub fn new(client: irc::streams::Client) -> Self {
        Bridge {
            client: client
        }
    }

    pub fn run(&mut self) {
        let mut events = EventLoop::new().unwrap();
        events.register(self.client.stream(), CLIENT, EventSet::all(), PollOpt::edge()).unwrap();
        events.run(self).unwrap();
    }
}

fn handle_client(client: &mut irc::streams::Client) {
    loop {
        match client.read_message() {
            None => return,
            Some(message) => {
                println!("Got a message! {:?}", message);
                match message.command {
                    Command::Pass => {
                        client.auth.set_password(message.args[0].clone())
                    }
                    Command::Nick => {
                        client.set_nickname(message.args[0].clone());
                    },
                    Command::User => {
                        println!("User logged in: {}", message.args[0]);
                        client.auth.set_username(message.args[0].clone());
                        let auth = client.auth.consume();
                        match (auth.username, auth.password) {
                            (Some(username), Some(password)) => {
                                client.matrix.login(username.trim(), password.trim());
                                client.welcome("Welcome!");
                                client.matrix.sync();
                                println!("Logged in {:?}", username);
                            },
                            _ => panic!("Username and/or password missing")
                        };
                    },
                    Command::Join => {
                        client.join(&message.args[0]);
                    },
                    Command::Ping => {
                        client.pong();
                    },
                    Command::Quit => {
                        return;
                    },
                    _ =>
                        println!("unhandled {:?}", message)
                }
            }
        }
    }
}
