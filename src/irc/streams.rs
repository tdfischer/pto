use mio::tcp::{TcpListener,TcpStream};
use std::net::SocketAddr;
use std::io::Write;
use std::io;

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

    pub fn read_message(&mut self) -> Option<Message> {
        match self.line_reader.read(&mut self.stream) {
            Some(line) =>
                Some(Message::from_str(line.trim())),
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
        self.stream.write(&message.to_string().trim().as_bytes())
            .and(self.stream.write("\r\n".as_bytes()))
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

