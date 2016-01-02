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

pub enum RoomEvent {
    CanonicalAlias(String),
    JoinRules(String),
    Membership(UserID, MembershipAction),
    HistoryVisibility(String),
    Create,
    Aliases,
    PowerLevels
}

pub enum TextEvent {
    RoomMessage(UserID, RoomID, String)
}

pub enum EventData {
    Room(RoomID, RoomEvent),
    Text(TextEvent),
}

pub struct Event {
    pub id: String,
    pub data: EventData
}

impl Event {
    pub fn from_json(json: &Json) -> Self {
        let tokens: Vec<&str> = mjson::string(json, "type").trim().split(".").collect();
        assert!(tokens[0] == "m");
        Event {
            id: mjson::string(json, "event_id").to_string(),
            data: match tokens[1] {
                "room" =>
                    Self::from_room_json(tokens[2], json),
                e => panic!("Unknown event {:?}!", e)
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
