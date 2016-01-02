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
    Privmsg,
    Numeric(u32),
    Unknown(String)
}

impl Command {
    pub fn as_string(&self) -> String {
        match self {
            &Command::Nick => "NICK".to_string(),
            &Command::Join => "JOIN".to_string(),
            &Command::Pong => "PONG".to_string(),
            &Command::Privmsg => "PRIVMSG".to_string(),
            &Command::User => "USER".to_string(),
            &Command::Quit => "QUIT".to_string(),
            &Command::Ping => "PING".to_string(),
            &Command::Mode => "MODE".to_string(),
            &Command::Pass => "PASS".to_string(),
            &Command::Numeric(n)=> format!("{:0>3}", n),
            &Command::Unknown(ref s) => s.clone()
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
        ret.push_str(self.command.as_string().trim());
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
