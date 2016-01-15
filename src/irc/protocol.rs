use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    Nick,
    User,
    Join,
    Part,
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
            &Command::Part => "PART".to_string(),
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

        match self.suffix {
            Some(ref sfx) => {
                ret.push_str(" :");
                ret.push_str(sfx.trim());
            },
            None => ()
        };

        return ret;
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

    pub fn from_str(line: &str) -> Self {
        let parts = Self::split_parts(line.trim());
        let split: Vec<&str> = parts.1.split(" ").collect();
        let mut args = Vec::new();
        for s in split[1..].iter() {
            args.push(s.to_string());
        }
        let parsedCommand: Result<Command, Command> = split[0].parse();
        Message{
            prefix: parts.0,
            command: parsedCommand.ok().unwrap(),
            args: args,
            suffix: parts.2
        }
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
            "PART" => Ok(Command::Part),
            "QUIT" => Ok(Command::Quit),
            "PING" => Ok(Command::Ping),
            "MODE" => Ok(Command::Mode),
            "PASS" => Ok(Command::Pass),
            "PRIVMSG" => Ok(Command::Privmsg),
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
