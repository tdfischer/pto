use std::net::{TcpListener,TcpStream};
use std::io::Read;
use std::io::Write;
use std::ops::DerefMut;
use std::str;
use std::sync::Mutex;
use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    Nick,
    User,
    Join,
    Quit,
    Ping,
    Mode,
    Pong,
    Numeric(u32),
    Unknown(String)
}

impl Command {
    fn as_str(&self) -> &'static str {
        match *self {
            Command::Nick => "NICK",
            Command::Join => "JOIN",
            Command::Pong => "PONG",
            Command::Numeric(1) => "001",
            _ => ""
        }
    }
}

impl Message {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        match self.prefix {
            Some(ref pfx) => {
                ret.push(':');
                ret.push_str(pfx.trim());
                ret.push(' ');
            },
            None => ()
        };
        ret.push_str(self.command.as_str());
        for ref arg in self.args.iter() {
            ret.push(' ');
            ret.push_str(arg.trim());
        }

        return ret;
    }
}

impl From<Command> for Message {
    fn from(c: Command) -> Message {
        Message {
            prefix: None,
            command: c,
            args: vec![],
            suffix: None
        }
    }
}

impl FromStr for Command {
    type Err = Command;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NICK" => Ok(Command::Nick),
            "USER" => Ok(Command::User),
            "JOIN" => Ok(Command::Join),
            "QUIT" => Ok(Command::Quit),
            "PING" => Ok(Command::Ping),
            "MODE" => Ok(Command::Mode),
            _ => Ok(Command::Unknown(s.to_string()))
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: Command,
    pub args: Vec<String>,
    pub suffix: Option<String>
}

#[derive(Debug)]
struct LineReader {
    linebuf: String
}

impl LineReader {
    pub fn new() -> Self {
        LineReader {
            linebuf: String::new()
        }
    }

    fn read(&mut self, stream: &mut Read) -> Option<String> {
        match self.split_next_line() {
            None => self.read_and_split(stream),
            Some(line) => Some(line)
        }
    }

    fn read_and_split(&mut self, stream: &mut Read) -> Option<String> {
        let mut buf = [0; 1024];
        let nextMsg = stream.read(&mut buf);
        match nextMsg {
            Ok(count) => {
                self.linebuf.push_str(str::from_utf8(&buf[0..count]).unwrap());
                self.split_next_line()
            }
            Err(_) => None
        }
    }

    fn split_next_line(&mut self) -> Option<String> {
        let newStr;
        let split;
        match self.linebuf.find("\r\n") {
            Some(idx) => {
                newStr = self.linebuf.clone();
                split = newStr.split_at(idx);
            },
            None =>
                return None

        }
        self.linebuf = split.1[2..].to_string().clone();
        Some(split.0.to_string())
    }
}

#[derive(Debug)]
pub struct Client {
    stream: Mutex<TcpStream>,
    line_reader: Mutex<LineReader>,
    nickname: Option<String>,
    username: Option<String>,
    pub server_prefix: Option<String>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client {
            stream: Mutex::new(stream),
            line_reader: Mutex::new(LineReader::new()),
            nickname: None,
            username: None,
            server_prefix: None
        }
    }

    pub fn iter(&self) -> ClientMessageIterator {
        ClientMessageIterator {
            client: self
        }
    }

    pub fn read_message(&self) -> Option<Message> {
        let mut stream = self.stream.lock().unwrap();
        let line = self.line_reader.lock().unwrap().read(stream.deref_mut()).unwrap();
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

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    pub fn join(&mut self, channel: &str) {
        self.send(&Message {
            prefix: Some(self.nickname.clone().unwrap()),
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

    pub fn send(&self, message: &Message) {
        let mut stream = self.stream.lock().unwrap();
        stream.write(&message.to_string().trim().as_bytes());
        stream.write("\r\n".as_bytes());
        println!("Wrote {:?}", message.to_string());
    }
}

pub struct ClientMessageIterator<'a> {
    client: &'a Client
}

impl<'a> Iterator for ClientMessageIterator<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        self.client.read_message()
    }
}

pub struct ServerIterator<'a> {
    server: &'a IrcServer
}

impl<'a> Iterator for ServerIterator<'a> {
    type Item = Client;

    fn next(&mut self) -> Option<Client> {
        let nextStream = self.server.listener.accept();
        match nextStream {
            Ok((stream, address)) => Some(Client::new(stream)),
            Err(_) => None
        }
    }
}

pub struct IrcServer {
    listener: TcpListener
}

impl IrcServer {
    pub fn new() -> Self {
        IrcServer {
            listener: TcpListener::bind("127.0.0.1:8001").unwrap()
        }
    }

    pub fn iter_new_clients<'a>(&'a self) -> ServerIterator<'a> {
        ServerIterator {
            server: self
        }
    }
}
