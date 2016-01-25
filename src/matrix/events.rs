use rustc_serialize::json::Json;
use rustc_serialize::json;
use matrix::json as mjson;
use std::fmt;

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

#[derive(Clone, Debug)]
pub struct UserID {
    pub nickname: String,
    pub homeserver: String 
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

#[derive(Debug)]
pub enum MembershipAction {
    Join,
    Leave,
    Ban,
    Invite,
}

impl MembershipAction {
    pub fn from_str(s: &str) -> Self {
        match s {
            "join" => MembershipAction::Join,
            "leave" => MembershipAction::Leave,
            "ban" => MembershipAction::Ban,
            "invite" => MembershipAction::Invite,
            _ => panic!("unknown membership action {:?}", s)
        }
    }
}

#[derive(Debug)]
pub enum RoomEvent {
    CanonicalAlias(String),
    JoinRules(String),
    Membership(UserID, MembershipAction),
    HistoryVisibility(String),
    Create,
    Aliases,
    Message(UserID, String),
    PowerLevels,
    Name(UserID, String)
}

#[derive(Debug)]
pub struct TypingEvent {
    pub users: Vec<UserID>,
    pub room: RoomID,
}

#[derive(Debug)]
pub enum EventData {
    Room(RoomID, RoomEvent),
    Typing(TypingEvent),
    Presence(PresenceEvent)
}

impl EventData {
    pub fn type_str(&self) -> &'static str {
        match self {
            &EventData::Room(_, RoomEvent::Message(_, _)) => {
                "m.room.message"
            },
            _ => unreachable!()
        }
    }

    pub fn to_json(&self) -> json::Json {
        let mut ret = json::Object::new();
        match self {
            &EventData::Room(ref id, ref evt) => {
                match evt {
                    &RoomEvent::Message(ref sender, ref text) => {
                        ret.insert("msgtype".to_string(), json::Json::String("m.text".to_string()));
                        ret.insert("body".to_string(), json::Json::String(text.clone()));
                    },
                    _ => unreachable!()
                }
            },
            _ => unreachable!()
        }
        json::Json::Object(ret)
    }
}

#[derive(Debug)]
pub struct Event {
    pub id: Option<EventID>,
    pub data: EventData
}

#[derive(Debug)]
pub struct PresenceEvent {
    pub presence: String,
    pub user: UserID
}

impl Event {
    pub fn from_json(json: &Json) -> Self {
        let tokens: Vec<&str> = mjson::string(json, "type").trim().split(".").collect();
        assert!(tokens[0] == "m");
        let id = match json.as_object().unwrap().get("event_id") {
            Some(i) => Some(EventID::from_str(i.as_string().unwrap())),
            None => None
        };
        Event {
            id: id,
            data: match tokens[1] {
                "room" =>
                    Self::from_room_json(tokens[2], json),
                "typing" =>
                    EventData::Typing(TypingEvent {
                        users: vec![],
                        room: RoomID::from_str(mjson::string(json, "room_id"))
                    }),
                "presence" =>
                    EventData::Presence(PresenceEvent{
                        presence: mjson::string(json, "content.presence").to_string(),
                        user: UserID::from_str(mjson::string(json, "content.user_id"))
                    }),
                e => panic!("Unknown event {:?}!\nRaw JSON: {:?}", e, json)
            }
        }
    }

    fn from_room_json(event_type: &str, json: &Json) -> EventData {
        EventData::Room(
            RoomID::from_str(mjson::string(json, "room_id")),
            match event_type {
                "canonical_alias" =>
                    RoomEvent::CanonicalAlias(mjson::string(json, "content.alias").to_string()),
                "join_rules" =>
                    RoomEvent::JoinRules(mjson::string(json, "content.join_rule").to_string()),
                "member" =>
                    RoomEvent::Membership(UserID::from_str(mjson::string(json, "user_id")), MembershipAction::from_str(mjson::string(json, "content.membership"))),
                "history_visibility" =>
                    RoomEvent::HistoryVisibility(mjson::string(json, "content.history_visibility").to_string()),
                "create" =>
                    RoomEvent::Create,
                "aliases" =>
                    RoomEvent::Aliases,
                "power_levels" =>
                    RoomEvent::PowerLevels,
                "message" =>
                    RoomEvent::Message(UserID::from_str(mjson::string(json, "user_id")), mjson::string(json, "content.body").to_string()),
                "name" =>
                    RoomEvent::Name(UserID::from_str(mjson::string(json, "user_id")), mjson::string(json, "content.name").to_string()),
                e => panic!("Unknown room event {:?}: {:?}", e, json)
            }
        )
    }
}
