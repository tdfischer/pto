/*
 * Copyright 2015-2016 Torrie Fischer <tdfischer@hackerbots.net>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use irc;
use matrix;
use irc::protocol::{Command,Message};
use irc::streams::AsEvented;
use mio;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt,Sender};
use std::thread;
use std::collections::{HashMap, BTreeSet};
use std::io;
use hyper;

const CLIENT: Token = Token(0);

#[derive(Debug)]
pub enum Event {
    EndPoll,
    Matrix(matrix::events::Event)
}

pub struct Bridge {
    client: irc::streams::Client,
    matrix: matrix::client::Client,
    rooms: HashMap<matrix::model::RoomID, Room>,
    seen_events: Vec<matrix::model::EventID>,
    last_token: String
}

impl Handler for Bridge {
    type Timeout = ();
    type Message = Event;

    fn ready(&mut self, event_loop: &mut EventLoop<Bridge>, token: Token, _: EventSet) {
        match token {
            CLIENT =>
                self.handle_client(event_loop),
            _ => unreachable!("Got a really weird Token in the mio event loop!")
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Bridge>, msg: Self::Message) {
        match msg {
            Event::EndPoll => {
                self.poll_matrix(event_loop.channel());
            },
            Event::Matrix(e) =>
                match self.handle_matrix(e) {
                    Err(err) => warn!("Could not handle matrix event: {:?}", err),
                    _ => ()
                }
        };
    }
}

unsafe impl Sync for Bridge{}

struct Room {
    id: matrix::model::RoomID,
    irc_name: Option<String>,
    canonical_alias: Option<String>,
    join_rules: Option<String>,
    members: BTreeSet<matrix::model::UserID>,
    aliases: Vec<String>,
    pending_events: Vec<(u64, matrix::events::RoomEvent)>,
    pending_sync: bool,
    is_pm: bool
}

impl Room {
    fn userid_to_irc(uid: &matrix::model::UserID) -> String {
        format!("{}!{}@{}", uid.nickname, uid.nickname, uid.homeserver)
    }

    fn has_irc_name(&self) -> bool {
        self.irc_name != None
    }

    fn handle_part<F>(&mut self, user: matrix::model::UserID, mut callback: &mut F)
            where F: FnMut(irc::protocol::Message) {

        let did_exist = self.members.remove(&user);
        if self.has_irc_name() && did_exist {
            callback(irc::protocol::Message {
                prefix: Some(Room::userid_to_irc(&user)),
                command: irc::protocol::Command::Part,
                args: vec![self.irc_name.clone().unwrap()],
                suffix: None
            });
        }
    }

    fn handle_join<F>(&mut self, user: matrix::model::UserID, mut callback: &mut F)
            where F: FnMut(irc::protocol::Message) {
        let uid = Room::userid_to_irc(&user);
        let was_added = self.members.insert(user);
        if self.has_irc_name() && was_added {
            callback(irc::protocol::Message {
                prefix: Some(uid),
                command: irc::protocol::Command::Join,
                args: vec![self.irc_name.clone().unwrap()],
                suffix: None
            });
        }
    }

    fn new(id: matrix::model::RoomID) -> Self {
        Room {
            id: id,
            canonical_alias: None,
            join_rules: None,
            members: BTreeSet::new(),
            pending_events: vec![],
            aliases: vec![],
            pending_sync: true,
            irc_name: None,
            is_pm: false
        }
    }

    fn run_pending<F>(&mut self, mut callback: &mut F)
            where F: FnMut(irc::protocol::Message) {
        assert!(self.pending_sync);
        self.pending_events.sort_by(|a, b|{
            a.0.cmp(&b.0)
        });
        while let Some((age, evt)) = self.pending_events.pop() {
            self.handle_with_alias(evt, callback, age);
        }
    }

    fn update_irc_name(&mut self, my_uid: &matrix::model::UserID) {
        // First check if we have a local alias that matches our homeserver
        for a in &self.aliases {
            if a.ends_with(&*format!(":{}", my_uid.homeserver)) {
                self.irc_name = Some(a.clone());
                break;
            }
        }
        // If we didn't get one from above, try some other heuristics
        if self.irc_name == None {
            self.irc_name = match self.canonical_alias {
                // No canonical alias set. See if we can grab some other alias.
                None => {
                    if self.aliases.len() == 0 {
                        // A room with two people is probably a PM.
                        // FIXME: Some other heuristics around private rooms, permissions, etc
                        if self.members.len() == 2 {
                            self.is_pm = true;
                            Some(format!("{}", self.members.iter().nth(0).unwrap().nickname))
                        } else {
                            Some(format!("#{}:{}", self.id.id, self.id.homeserver))
                        }
                    } else {
                        // First one is good as any I guess!
                        Some(self.aliases[0].clone())
                    }
                },
                // There's a canonical_alias set, so use that
                Some(ref a) => Some(a.clone())
            }
        }
    }

    pub fn finish_sync<F>(&mut self, my_uid: &matrix::model::UserID, mut callback: &mut F)
            where F: FnMut(irc::protocol::Message) {
        self.update_irc_name(my_uid);
        if self.pending_sync {
            if !self.is_pm {
                // Send the initial join for the current user on this connection, now that we have an IRC friendly channel name
                callback(irc::protocol::Message {
                    prefix: Some(Room::userid_to_irc(my_uid)),
                    command: irc::protocol::Command::Join,
                    args: vec![self.irc_name.clone().unwrap()],
                    suffix: None
                });
                // And then send the nicklist
                let mut usernames: Vec<String> = vec![];
                for u in &self.members {
                    usernames.push(format!("{}", u.nickname));
                }
                callback(irc::protocol::Message {
                    prefix: Some("pto".to_string()),
                    command: irc::protocol::Command::Numeric(353),
                    args: vec![my_uid.nickname.clone(), "@".to_string(), self.irc_name.clone().unwrap()],
                    suffix: Some(usernames.join(" "))
                });
            }
            self.run_pending(callback);
            self.pending_sync = false;
        }
    }

    fn handle_with_alias<F>(&mut self, evt: matrix::events::RoomEvent, mut callback: &mut F, age: u64)
            where F: FnMut(irc::protocol::Message) {
        if self.has_irc_name() {
            match evt {
                matrix::events::RoomEvent::Membership(_, _) => (),
                matrix::events::RoomEvent::Message(user, text) => {
                    if self.is_pm {
                        if self.irc_name == Some(user.nickname.clone()) {
                            callback(irc::protocol::Message {
                                prefix: Some(Room::userid_to_irc(&user)),
                                command: irc::protocol::Command::Privmsg,
                                args: vec![self.irc_name.clone().unwrap()],
                                suffix: Some(text)
                            });
                        } else {
                            callback(irc::protocol::Message {
                                prefix: None,
                                command: irc::protocol::Command::Privmsg,
                                args: vec![self.irc_name.clone().unwrap()],
                                suffix: Some(text)
                            });
                        }
                    } else {
                        callback(irc::protocol::Message {
                            prefix: Some(Room::userid_to_irc(&user)),
                            command: irc::protocol::Command::Privmsg,
                            args: vec![self.irc_name.clone().unwrap()],
                            suffix: Some(text)
                        });
                    }
                },
                matrix::events::RoomEvent::Topic(user, topic) => {
                    callback(irc::protocol::Message {
                        prefix: Some(Room::userid_to_irc(&user)),
                        command: irc::protocol::Command::Topic,
                        args: vec![self.irc_name.clone().unwrap()],
                        suffix: Some(topic.clone())
                    });
                },
                _ => {
                    warn!("Unhandled event {:?}", evt)
                }
            }
        } else {
            self.pending_events.push((age, evt));
        }
    }

    fn handle_event<F>(&mut self, evt: matrix::events::RoomEvent, mut callback: F, age: u64)
            where F: FnMut(irc::protocol::Message) {
        match evt {
            matrix::events::RoomEvent::CanonicalAlias(name) => {
                self.canonical_alias = Some(name.clone());
            },
            matrix::events::RoomEvent::JoinRules(rules) =>
                self.join_rules = Some(rules.clone()),
            matrix::events::RoomEvent::Create => (),
            matrix::events::RoomEvent::Aliases(aliases) =>
                self.aliases = aliases,
            matrix::events::RoomEvent::PowerLevels => (),
            matrix::events::RoomEvent::HistoryVisibility(_) => (),
            matrix::events::RoomEvent::Name(_, _) => (),
            matrix::events::RoomEvent::Avatar(_, _) => (),
            matrix::events::RoomEvent::Membership(user, matrix::events::MembershipAction::Join) => {
                self.handle_join(user, &mut callback);
            },
            matrix::events::RoomEvent::Membership(user, matrix::events::MembershipAction::Leave) => {
                self.handle_part(user, &mut callback);
            },
            matrix::events::RoomEvent::Unknown(unknown_type, json) => {
                warn!("Unknown room event {}", unknown_type);
                if cfg!(raw_logs) {
                    trace!("raw event: {:?}", json);
                }
            }
            _ => self.handle_with_alias(evt, &mut callback, age)
        };
    }
}


impl Bridge {
    fn room_from_matrix(&mut self, id: &matrix::model::RoomID) -> &mut Room {
        if !self.rooms.contains_key(id) {
            self.rooms.insert(id.clone(), Room::new(id.clone()));
        }
        match self.rooms.get_mut(id) {
            Some(room) => room,
            None => unreachable!("Couldn't find the room that we just created")
        }
    }

    fn room_from_irc(&mut self, id: &String) -> Option<&mut Room> {
        let mut room_id: Option<matrix::model::RoomID> = None;
        for (_, r) in self.rooms.iter_mut() {
            if let Some(ref alias) = r.irc_name {
                if alias == id {
                    room_id = Some(r.id.clone())
                }
            }
        }
        match room_id {
            Some(id) => Some(self.room_from_matrix(&id)),
            None => None
        }
    }

    pub fn new(client: irc::streams::Client, url: hyper::Url) -> Self {
        Bridge {
            client: client,
            matrix: matrix::client::Client::new(url),
            rooms: HashMap::new(),
            seen_events: vec![],
            last_token: String::new()
        }
    }

    pub fn run(&mut self) {
        let mut events = EventLoop::new().unwrap();
        events.register(self.client.as_evented(), CLIENT, EventSet::all(), PollOpt::edge()).unwrap();
        events.run(self).unwrap();
    }

    fn finish_sync<F>(&mut self, mut callback: &mut F, token: String)
            where F: FnMut(irc::protocol::Message) {
        for (_, mut room) in &mut self.rooms {
            room.finish_sync(&self.matrix.uid.as_ref().unwrap(), callback);
        };
        self.last_token = token;
    }

    fn handle_matrix(&mut self, evt: matrix::events::Event) -> io::Result<usize> {
        let duplicate = match evt.id {
            Some(ref id) =>
                self.seen_events.contains(id),
            _ => false
        };
        if !duplicate {
            let mut messages: Vec<irc::protocol::Message> = vec![];
            {
                let mut append_msg = |msg: irc::protocol::Message| {
                    messages.push(msg);
                };
                match evt.data {
                    matrix::events::EventData::Room(room_id, room_event) => {
                        self.room_from_matrix(&room_id).handle_event(room_event, append_msg, evt.age);
                    },
                    matrix::events::EventData::Typing(_) => (),
                    matrix::events::EventData::EndOfSync(token) => self.finish_sync(&mut append_msg, token),
                    _ => warn!("Unhandled {}", evt.data.type_str())
                }
            }
            match evt.id {
                Some(id) =>
                    self.seen_events.push(id),
                None => ()
            };
            let mut res: Option<io::Result<usize>> = None;
            for ref msg in messages {
                res = Some(match res {
                    None => self.client.send(msg),
                    Some(r) => r.and(self.client.send(msg))
                })
            }
            match res {
                None => Ok(0),
                Some(e) => e
            }
        } else {
            Ok(0)
        }
    }

    fn poll_matrix(&mut self, channel: mio::Sender<Event>) ->
        thread::JoinHandle<matrix::client::Result> {
        let poll = self.matrix.sync(Some(&*self.last_token));
        thread::spawn(move|| {
            poll.send().and_then(|evts| {
                for evt in evts {
                    channel.send(Event::Matrix(evt)).unwrap();
                };
                channel.send(Event::EndPoll).unwrap();
                Ok(())
            })
        })
    }

    fn start_matrix(&mut self, channel: mio::Sender<Event>) ->
        matrix::client::Result {
        self.matrix.sync(None).send().and_then(|events| {
            for e in events {
                match self.handle_matrix(e) {
                    // FIXME: Return error
                    Err(err) => warn!("Could not handle matrix event: {:?}", err),
                    _ => ()
                }
            }
            self.poll_matrix(channel);
            Ok(())
        })
    }

    fn handle_client(&mut self, events: &mut EventLoop<Bridge>) {
        loop {
            match self.client.read_message() {
                None => return,
                Some(message) => {
                    match message.command {
                        Command::Pass => {
                            self.client.auth.set_password(message.args[0].clone())
                        }
                        Command::Nick => {
                            let nickname = match message.suffix {
                                None => message.args[0].clone(),
                                Some(n) => n
                            };
                            self.client.set_nickname(nickname)
                        },
                        Command::User => {
                            self.client.auth.set_username(message.args[0].clone());
                            let auth = self.client.auth.consume();
                            match (auth.username, auth.password) {
                                (Some(username), Some(password)) => {
                                    self.matrix.login(&*username, &*password)
                                        .and_then(|_| {
                                            self.start_matrix(events.channel())
                                        })
                                        .and_then(|_| {
                                            self.client.welcome("Welcome to Perpetually Talking Online!").unwrap();
                                            debug!("Logged in a user");
                                            Ok(())
                                        }).expect("Could not login!");
                                },
                                (Some(_), None) => {
                                    self.matrix.anon_login()
                                        .and_then(|_| {
                                            self.start_matrix(events.channel())
                                        })
                                        .and_then(|_| {
                                            self.client.welcome("Welcome to Perpetually Talking Online!").unwrap();
                                            debug!("Logged in a user");
                                            Ok(())
                                        }).expect("Could not login!");
                                },
                                _ => panic!("Username missing, and anonymous access isn't built yet.")
                            };
                        },
                        Command::Join => {
                            // FIXME: Send no such channel message
                        },
                        Command::Ping => {
                            self.client.pong().expect("Could not send PONG");
                        },
                        Command::Quit => {
                            // FIXME: Logout of matrix and exit thread
                            return;
                        },
                        Command::Privmsg => {
                            let room_id = match self.room_from_irc(&message.args[0]) {
                                None => return (),
                                Some(room) => room.id.clone()
                            };
                            let evt = {
                                let id = self.matrix.uid.clone().unwrap();
                                let message_text = if message.suffix == None {
                                    message.args[1].clone()
                                } else {
                                    message.suffix.unwrap()
                                };
                                matrix::events::EventData::Room(
                                    room_id,
                                    matrix::events::RoomEvent::Message(
                                        id, message_text))
                            };
                            self.seen_events.push(self.matrix.send(evt).expect("Could not send event"));
                        },
                        _ =>
                            warn!("unhandled {:?}", message)
                    }
                }
            }
        }
    }
}

