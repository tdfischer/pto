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
use mio::Evented;
use openssl::ssl::SslStream;
use mio::tcp::TcpStream;

use irc::util::LineReader;
use irc::protocol::*;
use irc::security::AuthSession;

pub trait AsEvented {
    fn as_evented(&self) -> &Evented;
}

pub trait IrcStream: Read + Write + AsEvented + Send {}

impl IrcStream for SslStream<TcpStream> {}
impl IrcStream for TcpStream {}

impl AsEvented for TcpStream {
    fn as_evented(&self) -> &Evented {
        self
    }
}

impl AsEvented for SslStream<TcpStream> {
    fn as_evented(&self) -> &Evented {
        self.get_ref()
    }
}

impl AsEvented for Client {
    fn as_evented(&self) -> &Evented {
        self.stream.as_evented()
    }
}

pub struct Client {
    stream: Box<IrcStream>,
    line_reader: LineReader,
    nickname: Option<String>,
    username: Option<String>,
    pub auth: AuthSession,
}

impl Client {
    pub fn new(stream: Box<IrcStream>) -> Self {
        Client {
            stream: stream,
            line_reader: LineReader::new(),
            nickname: None,
            username: None,
            auth: AuthSession::new(),
        }
    }

    pub fn read_message(&mut self) -> Option<Message> {
        match self.line_reader.read(&mut self.stream) {
            Some(line) => {
                trace!("<< {}", line);
                let stripped = line.trim();
                if stripped.len() == 0 {
                    None
                } else {
                    Some(Message::from_str(stripped))
                }
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
        let nickname = self.nickname.clone().unwrap();
        self.send(&Message {
            prefix: Some("pto".to_string()),
            command: Command::Numeric(1),
            args: vec![nickname.clone()],
            suffix: Some(format!("Welcome to Perpetually Talking Online {}", nickname).to_string())
        }).and(self.send(&Message {
            prefix: Some("pto".to_string()),
            command: Command::Numeric(2),
            args: vec![nickname.clone()],
            suffix: Some("Your host is running Perpetually Talking Online, the IRC frontend to Matrix.".to_string())
        })).and(self.send(&Message {
            prefix: Some("pto".to_string()),
            command: Command::Numeric(5),
            args: vec![nickname.clone(), "CHANTYPES=# NETWORK=matrix CHARSET=utf-8".to_string()],
            suffix: Some("are supported by this server".to_string())
        }))
    }

    pub fn send(&mut self, message: &Message) -> io::Result<usize> {
        trace!(">>> {}", message.to_string());
        self.stream.write(&message.to_string().trim().as_bytes())
            .and(self.stream.write("\r\n".as_bytes()))
    }
}

pub trait Server: AsEvented {
    fn accept(&mut self) -> Option<Client>;
}
