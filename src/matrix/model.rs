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
use std::fmt;
use std::cmp;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RoomID {
    pub id: String,
    pub homeserver: String
}

impl fmt::Display for RoomID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "!{}:{}", self.id, self.homeserver)
    }
}

impl RoomID {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split(":").collect();
        RoomID {
            id: parts[0][1..].to_string(),
            homeserver: parts[1].to_string()
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UserID {
    pub nickname: String,
    pub homeserver: String 
}

impl fmt::Display for UserID{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{}:{}", self.nickname, self.homeserver)
    }
}

impl cmp::PartialOrd for UserID {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for UserID {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        format!("@{}:{}", self.nickname, self.homeserver).cmp(&format!("@{}:{}", other.nickname, other.homeserver))
    }
}

impl UserID {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split(":").collect();
        UserID {
            nickname: parts[0][1..].to_string(),
            homeserver: parts[1].to_string()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventID {
    pub id: String,
    pub homeserver: String
}

impl EventID {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split(":").collect();
        EventID {
            id: parts[0][1..].to_string(),
            homeserver: parts[1].to_string()
        }
    }
}
