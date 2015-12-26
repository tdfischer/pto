use mio::tcp::{TcpListener,TcpStream};
use mio::Token;
use std::net::SocketAddr;
use std::io::Write;

use irc::util::LineReader;
use irc::protocol::*;
use irc::security::AuthSession;
use matrix;

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    line_reader: LineReader,
    nickname: Option<String>,
    username: Option<String>,
    token: Token,
    pub auth: AuthSession,
    pub matrix: matrix::Client
}

impl Client {
    pub fn new(stream: TcpStream, token: Token) -> Self {
        Client {
            stream: stream,
            line_reader: LineReader::new(),
            nickname: None,
            username: None,
            token: token,
            auth: AuthSession::new(),
            matrix: matrix::Client::new()
        }
    }

    pub fn token(&self) -> Token {
        self.token
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn read_message(&mut self) -> Option<Message> {
        let line = self.line_reader.read(&mut self.stream).unwrap();
        let split: Vec<&str> = line.split(" ").collect();
        let mut args = Vec::new();
        for s in split[1..].iter() {
            args.push(s.to_string());
        }
        let parsedCommand: Result<Command, Command> = split[0].parse();
        Some(Message{
            prefix: None,
            command: parsedCommand.ok().unwrap(),
            args: args,
            suffix: None
        })
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
    nextToken: usize
}

impl Server {
    pub fn new(addr: &SocketAddr) -> Self {
        Server {
            listener: TcpListener::bind(addr).unwrap(),
            nextToken: 1
        }
    }

    pub fn accept(&mut self) -> Option<Client> {
         match self.listener.accept() {
             Ok(None) => None,
             Ok(Some((socket, _))) => {
                 let token = self.nextToken;
                 self.nextToken += 1;
                 Some(Client::new(socket, Token(token)))
             }
             Err(e) => panic!(e),
         }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

