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

use std::io::{Read, Write};
use std::io;

use irc::util::LineReader;
use irc::protocol::*;
use irc::security::AuthSession;

#[derive(Debug)]
pub struct Client<IrcStream>
        where IrcStream: Read + Write {
    stream: IrcStream,
    line_reader: LineReader,
    nickname: Option<String>,
    username: Option<String>,
    pub auth: AuthSession,
}

impl<IrcStream> Client<IrcStream>
        where IrcStream: Read + Write {
    pub fn new(stream: IrcStream) -> Self {
        Client {
            stream: stream,
            line_reader: LineReader::new(),
            nickname: None,
            username: None,
            auth: AuthSession::new(),
        }
    }

    pub fn stream(&self) -> &IrcStream {
        &self.stream
    }

    pub fn read_message(&mut self) -> Option<Message> {
        match self.line_reader.read(&mut self.stream) {
            Some(line) => {
                trace!("<< {}", line);
                Some(Message::from_str(line.trim()))
            },
            None => None
        }
    }

    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = Some(nickname);
    }

    pub fn join(&mut self, channel: &str) -> io::Result<usize> {
        let pfx = self.nickname.clone().unwrap();
        self.send(&Message {
            prefix: Some(pfx),
            command: Command::Join,
            args: vec![channel.to_string()],
            suffix: None
        })
    }

    pub fn pong(&mut self) -> io::Result<usize> {
        self.send(&Message::from(Command::Pong))
    }

    pub fn welcome(&mut self, message: &str) -> io::Result<usize> {
        self.send(&Message {
            prefix: Some("pto".to_string()),
            command: Command::Numeric(1),
            args: vec![message.to_string()],
            suffix: None
        })
    }

    pub fn send(&mut self, message: &Message) -> io::Result<usize> {
        trace!(">>> {}", message.to_string());
        self.stream.write(&message.to_string().trim().as_bytes())
            .and(self.stream.write("\r\n".as_bytes()))
    }
}

pub trait Server {
    type Client;
    fn accept(&mut self) -> Option<Self::Client>;
}
