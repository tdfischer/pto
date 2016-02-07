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

use std::str::FromStr;

#[derive(Debug,PartialEq,Eq)]
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
    Topic,
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
            &Command::Topic => "TOPIC".to_string(),
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
        ret.push_str(&self.command.as_string());
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
        if line.starts_with(":") {
            for c in line[1..].chars() {
                prefix_end += 1;
                if c == ' ' {
                    break
                }
            }
        }

        let mut args_end = line.len();
        let mut found_space = false;
        for (i, c) in line[prefix_end..].char_indices() {
            if c == ' ' {
                found_space = true;
            } else if c == ':' && found_space {
                args_end = prefix_end + i - 1;
                break
            } else {
                found_space = false;
            }
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
        let parsed_command: Result<Command, Command> = split[0].parse();
        Message{
            prefix: parts.0,
            command: parsed_command.ok().unwrap(),
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
            "TOPIC" => Ok(Command::Topic),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classic_irc_session() {
        let msg = Message::from_str("USER nick 0 * hostname");
        assert_eq!(msg.prefix, None);
        assert_eq!(msg.command, Command::User);
        assert_eq!(msg.args, &["nick", "0", "*", "hostname"]);
        assert_eq!(msg.suffix, None);

        let msg = Message::from_str("NICK nick");
        assert_eq!(msg.prefix, None);
        assert_eq!(msg.command, Command::Nick);
        assert_eq!(msg.args, &["nick"]);
        assert_eq!(msg.suffix, None);

        let msg = Message::from_str(":nick!nick@hostname JOIN #foo");
        assert_eq!(msg.prefix, Some("nick!nick@hostname".to_owned()));
        assert_eq!(msg.command, Command::Join);
        assert_eq!(msg.args, &["#foo"]);
        assert_eq!(msg.suffix, None);

        let msg = Message::from_str(":nick!nick@hostname PRIVMSG #foo :Hello World!");
        assert_eq!(msg.prefix, Some("nick!nick@hostname".to_owned()));
        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.args, &["#foo"]);
        assert_eq!(msg.suffix, Some("Hello World!".to_owned()));

        let msg = Message::from_str(":nick!nick@hostname QUIT :Goodbye!");
        assert_eq!(msg.prefix, Some("nick!nick@hostname".to_owned()));
        assert_eq!(msg.command, Command::Quit);
        assert!(msg.args.len() == 0);
        assert_eq!(msg.suffix, Some("Goodbye!".to_owned()));
    }

    #[test]
    fn weird_cases() {
        let msg = Message::from_str("USER nick:name 0 * hostname");
        assert_eq!(msg.prefix, None);
        assert_eq!(msg.command, Command::User);
        assert_eq!(msg.args, &["nick:name", "0", "*", "hostname"]);
        assert_eq!(msg.suffix, None);

        let msg = Message::from_str("USER  nick  0  *  hostname");
        assert_eq!(msg.prefix, None);
        assert_eq!(msg.command, Command::User);
        assert_eq!(msg.args, &["", "nick", "", "0", "", "*", "", "hostname"]);
        assert_eq!(msg.suffix, None);
    }

    #[test]
    fn utf8_messages() {
        let msg = Message::from_str(":nick!nick@hostname PRIVMSG #foo :Some utf8 fun éèàåöþœðßä");
        assert_eq!(msg.prefix, Some("nick!nick@hostname".to_owned()));
        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.args, &["#foo"]);
        assert_eq!(msg.suffix, Some("Some utf8 fun éèàåöþœðßä".to_owned()));

        let msg = Message::from_str(":nick!nick@hostname PRIVMSG #héhé :In a chan with utf8 in its name!");
        assert_eq!(msg.prefix, Some("nick!nick@hostname".to_owned()));
        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.args, &["#héhé"]);
        assert_eq!(msg.suffix, Some("In a chan with utf8 in its name!".to_owned()));
    }
}
