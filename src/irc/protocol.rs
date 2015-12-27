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
    Pass,
    Numeric(u32),
    Unknown(String)
}

impl Command {
    pub fn as_str(&self) -> &'static str {
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
    pub fn to_string(&self) -> String {
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
            "PASS" => Ok(Command::Pass),
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
