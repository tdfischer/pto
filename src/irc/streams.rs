use mio::tcp::{TcpListener,TcpStream};
use mio::Token;
use std::net::SocketAddr;
use std::io::Write;

use irc::util::LineReader;
use irc::protocol::*;
use irc::security::AuthSession;

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    line_reader: LineReader,
    nickname: Option<String>,
    username: Option<String>,
    pub auth: AuthSession,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client {
            stream: stream,
            line_reader: LineReader::new(),
            nickname: None,
            username: None,
            auth: AuthSession::new(),
        }
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    fn split_parts(line: &str) -> (Option<String>, &str, Option<String>) {
        let mut prefix_end = 0;
        let mut args_end = 0;
        if line.chars().nth(0).unwrap() == ':' {
            for c in line[1..].chars() {
                prefix_end += 1;
                if c == ' ' {
                    break
                }
            }
        }

        let mut found_space = false;
        for c in line[prefix_end..].chars() {
            if c == ' ' {
                found_space = true;
            } else if c == ':' && found_space {
                args_end -= 1;
                break
            } else {
                found_space = false;
            }
            args_end += 1;
        }

        let prefix = match prefix_end {
            0 => None,
            _ => Some(line[1..prefix_end].to_string())
        };
        let args = &line[prefix_end..args_end].trim();
        let len = line.len();
        let suffix = if args_end == len {
            None
        } else {
            Some(line[args_end+2..].to_string())
        };

        (prefix, args, suffix)
    }

    pub fn read_message(&mut self) -> Option<Message> {
        match self.line_reader.read(&mut self.stream) {
            Some(line) => {
                let parts = Self::split_parts(line.trim());
                let split: Vec<&str> = parts.1.split(" ").collect();
                let mut args = Vec::new();
                for s in split[1..].iter() {
                    args.push(s.to_string());
                }
                let parsedCommand: Result<Command, Command> = split[0].parse();
                Some(Message{
                    prefix: parts.0,
                    command: parsedCommand.ok().unwrap(),
                    args: args,
                    suffix: parts.2
                })
            },
            None => None
        }
    }

    pub fn set_nickname(&mut self, nickname: String) {
        self.nickname = Some(nickname);
    }

    pub fn join(&mut self, channel: &str) {
        let pfx = self.nickname.clone().unwrap();
        self.send(&Message {
            prefix: Some(pfx),
            command: Command::Join,
            args: vec![channel.to_string()],
            suffix: None
        });
    }

    pub fn pong(&mut self) {
        self.send(&Message::from(Command::Pong));
    }

    pub fn welcome(&mut self, message: &str) {
        self.send(&Message {
            prefix: Some("pto".to_string()),
            command: Command::Numeric(1),
            args: vec![message.to_string()],
            suffix: None
        });
    }

    pub fn send(&mut self, message: &Message) {
        self.stream.write(&message.to_string().trim().as_bytes());
        self.stream.write("\r\n".as_bytes());
        println!("Wrote {:?}", message.to_string());
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(addr: &SocketAddr) -> Self {
        Server {
            listener: TcpListener::bind(addr).unwrap(),
        }
    }

    pub fn accept(&mut self) -> Option<Client> {
         match self.listener.accept() {
             Ok(None) => None,
             Ok(Some((socket, _))) =>
                 Some(Client::new(socket)),
             Err(e) => panic!(e),
         }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

