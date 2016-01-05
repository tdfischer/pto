use rustc_serialize::json::Json;
use matrix::json as mjson;

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
    Join
}

impl MembershipAction {
    pub fn from_str(s: &str) -> Self {
        match s {
            "join" => MembershipAction::Join,
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
    PowerLevels
}

#[derive(Debug)]
pub enum TextEvent {
    RoomMessage(UserID, RoomID, String)
}

#[derive(Debug)]
pub struct TypingEvent {
    pub users: Vec<UserID>,
    pub room: RoomID,
}

#[derive(Debug)]
pub enum EventData {
    Room(RoomID, RoomEvent),
    Text(TextEvent),
    Typing(TypingEvent),
    Presence(PresenceEvent)
}

#[derive(Debug)]
pub struct Event {
    pub id: Option<String>,
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
            Some(i) => Some(i.as_string().unwrap().to_string()),
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
                e => panic!("Unknown room event {:?}: {:?}", e, json)
            }
        )
    }
}
