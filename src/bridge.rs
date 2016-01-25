use irc;
use matrix;
use irc::protocol::{Command,Message};
use mio;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt,Sender};
use std::thread;
use std::collections::HashMap;

const CLIENT: Token = Token(0);
const MATRIX: Token = Token(1);

#[derive(Debug)]
pub enum Event {
    EndPoll,
    Matrix(matrix::events::Event)
}

pub struct Bridge {
    client: irc::streams::Client,
    matrix: matrix::client::Client,
    rooms: HashMap<matrix::events::RoomID, Room>,
    seenEvents: Vec<matrix::events::EventID>,
}

impl Handler for Bridge {
    type Timeout = ();
    type Message = Event;

    fn ready(&mut self, event_loop: &mut EventLoop<Bridge>, token: Token, _: EventSet) {
        match token {
            CLIENT =>
                self.handle_client(event_loop),
            _ => unreachable!()
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Bridge>, msg: Self::Message) {
        println!("Got message from matrix! {:?}", msg);
        match msg {
            Event::EndPoll => {
                self.poll_matrix(event_loop.channel());
            },
            Event::Matrix(e) =>
                self.handle_matrix(e)
        };
    }
}

unsafe impl Sync for Bridge{}

struct Room {
    id: matrix::events::RoomID,
    canonical_alias: Option<String>,
    join_rules: Option<String>,
    members: Vec<matrix::events::UserID>
}

impl Room {
    fn new(id: matrix::events::RoomID) -> Self {
        Room {
            id: id,
            canonical_alias: None,
            join_rules: None,
            members: vec![]
        }
    }

    fn handle_event<F>(&mut self, evt: matrix::events::RoomEvent, mut callback: F)
            where F: FnMut(irc::protocol::Message) {
        match evt {
            matrix::events::RoomEvent::CanonicalAlias(name) => {
                let was_empty = self.canonical_alias == None;
                self.canonical_alias = Some(name.clone());
                for ref user in &self.members {
                    callback(irc::protocol::Message {
                        prefix: Some(format!("{}@anony.oob", user.nickname)),
                        command: irc::protocol::Command::Join,
                        args: vec![self.canonical_alias.clone().unwrap()],
                        suffix: None
                    });
                }
            },
            matrix::events::RoomEvent::JoinRules(rules) =>
                self.join_rules = Some(rules.clone()),
            matrix::events::RoomEvent::Membership(user, matrix::events::MembershipAction::Join) => {
                if self.canonical_alias != None {
                    callback(irc::protocol::Message {
                        prefix: Some(format!("{}@anony.oob", user.nickname)),
                        command: irc::protocol::Command::Join,
                        args: vec![self.canonical_alias.clone().unwrap()],
                        suffix: None
                    });
                }
                self.members.push(user);
            },
            matrix::events::RoomEvent::Membership(user, matrix::events::MembershipAction::Leave) => {
                if self.canonical_alias != None {
                    callback(irc::protocol::Message {
                        prefix: Some(format!("{}@anony.oob", user.nickname)),
                        command: irc::protocol::Command::Part,
                        args: vec![self.canonical_alias.clone().unwrap()],
                        suffix: None
                    });
                }
                self.members.push(user);
            },
            matrix::events::RoomEvent::Membership(user, _) => (),
            matrix::events::RoomEvent::Message(user, text) => {
                callback(irc::protocol::Message {
                    prefix: Some(format!("{}@anony.oob", user.nickname)),
                    command: irc::protocol::Command::Privmsg,
                    args: vec![self.canonical_alias.clone().unwrap()],
                    suffix: Some(text)
                });
            },
            matrix::events::RoomEvent::Create => (),
            matrix::events::RoomEvent::Aliases => (),
            matrix::events::RoomEvent::PowerLevels => (),
            matrix::events::RoomEvent::HistoryVisibility(_) => (),
            matrix::events::RoomEvent::Name(_, _) => ()
        };
    }
}


impl Bridge {
    pub fn room(&mut self, id: &matrix::events::RoomID) -> &mut Room {
        if !self.rooms.contains_key(id) {
            self.rooms.insert(id.clone(), Room::new(id.clone()));
        }
        match self.rooms.get_mut(id) {
            Some(room) => room,
            None => unreachable!()
        }
    }

    pub fn new(client: irc::streams::Client, url: &str) -> Self {
        Bridge {
            client: client,
            matrix: matrix::client::Client::new(url),
            rooms: HashMap::new(),
            seenEvents: vec![]
        }
    }

    pub fn run(&mut self) {
        let mut events = EventLoop::new().unwrap();
        events.register(self.client.stream(), CLIENT, EventSet::all(), PollOpt::edge()).unwrap();
        events.run(self).unwrap();
    }

    fn handle_matrix(&mut self, evt: matrix::events::Event) {
        let duplicate = match evt.id {
            Some(ref id) =>
                self.seenEvents.contains(id),
            _ => false
        };
        if !duplicate {
            let mut messages: Vec<irc::protocol::Message> = vec![];
            {
                let appendMsg = |msg: irc::protocol::Message| {
                    messages.push(msg);
                };
                match evt.data {
                    matrix::events::EventData::Room(room_id, room_event) => {
                        self.room(&room_id).handle_event(room_event, appendMsg);
                    },
                    _ => println!("Unhandled {:?}", evt)
                }
            }
            match evt.id {
                Some(id) =>
                    self.seenEvents.push(id),
                None => ()
            };
            for ref msg in messages {
                self.client.send(msg);
            }
        }
    }

    fn poll_matrix(&mut self, channel: mio::Sender<Event>) -> thread::JoinHandle<()> {
        let poll = self.matrix.pollAsync();
        thread::spawn(move|| {
            for evt in poll.send() {
                channel.send(Event::Matrix(evt));
            };
            channel.send(Event::EndPoll);
        })
    }

    fn start_matrix(&mut self, channel: mio::Sender<Event>) {
        self.matrix.sync(|evt: matrix::events::Event| {
            channel.send(Event::Matrix(evt));
        });
        self.poll_matrix(channel);
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
                                    self.client.welcome(username.trim());
                                    self.start_matrix(events.channel());
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
                        Command::Privmsg => {
                            let uid = matrix::events::UserID::from_str("@tdfischer:localhost");
                            let roomid = matrix::events::RoomID::from_str("!SNCDinqFeGteFrlCoN:localhost");
                            let evt = matrix::events::EventData::Room(
                                roomid,
                                matrix::events::RoomEvent::Message(
                                    uid, message.suffix.unwrap()));
                            self.seenEvents.push(self.matrix.send(evt));
                        },
                        _ =>
                            println!("unhandled {:?}", message)
                    }
                }
            }
        }
    }
}

