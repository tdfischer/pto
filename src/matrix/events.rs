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

use rustc_serialize::json::Json;
use rustc_serialize::json;
use matrix::json as mjson;
use matrix::model;

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
    Membership(model::UserID, MembershipAction),
    HistoryVisibility(String),
    Create,
    Aliases(Vec<String>),
    Message(model::UserID, String),
    PowerLevels,
    Name(model::UserID, String),
    Avatar(model::UserID, String),
    Topic(model::UserID, String),
    Unknown(String, Json)
}

#[derive(Debug)]
pub struct TypingEvent {
    pub users: Vec<model::UserID>,
    pub room: model::RoomID,
}

#[derive(Debug)]
pub enum EventData {
    Room(model::RoomID, RoomEvent),
    Typing(TypingEvent),
    Presence(PresenceEvent),
    Unknown(String, Json),
    EndOfSync(String)
}

impl EventData {
    pub fn type_str(&self) -> String {
        match self {
            &EventData::Room(_, RoomEvent::Message(_, _)) =>
                "m.room.message".to_string(),
            &EventData::Room(_, RoomEvent::CanonicalAlias(_)) =>
                "m.room.canonical_alias".to_string(),
            &EventData::Room(_, RoomEvent::JoinRules(_)) =>
                "m.room.join_rules".to_string(),
            &EventData::Room(_, RoomEvent::Membership(_, _)) =>
                "m.room.member".to_string(),
            &EventData::Room(_, RoomEvent::HistoryVisibility(_)) =>
                "m.room.history_visibility".to_string(),
            &EventData::Room(_, RoomEvent::Create )=>
                "m.room.create".to_string(),
            &EventData::Room(_, RoomEvent::Aliases(_)) =>
                "m.room.aliases".to_string(),
            &EventData::Room(_, RoomEvent::PowerLevels) =>
                "m.room.power_levels".to_string(),
            &EventData::Room(_, RoomEvent::Name(_, _)) =>
                "m.room.name".to_string(),
            &EventData::Room(_, RoomEvent::Avatar(_, _)) =>
                "m.room.avatar".to_string(),
            &EventData::Room(_, RoomEvent::Topic(_, _)) =>
                "m.room.topic".to_string(),
            &EventData::Room(_, RoomEvent::Unknown(ref unknown_type, _)) =>
                format!("m.room.{}", unknown_type),
            &EventData::Typing(_) =>
                "m.typing".to_string(),
            &EventData::Presence(_) =>
                "m.presence".to_string(),
            &EventData::Unknown(ref unknown_type, _) => unknown_type.clone(),
            &EventData::EndOfSync(_) => panic!("EndOfSync is a special value")
        }
    }

    pub fn to_json(&self) -> json::Json {
        let mut ret = json::Object::new();
        match self {
            &EventData::Room(ref _id, ref evt) => {
                match evt {
                    &RoomEvent::Message(_, ref text) => {
                        ret.insert("msgtype".to_string(), json::Json::String("m.text".to_string()));
                        ret.insert("body".to_string(), json::Json::String(text.clone()));
                    },
                    _ => panic!("Can only serialize m.room.message events :(")
                }
            },
            _ => panic!("Can only serialize m.room.message events :(")
        }
        json::Json::Object(ret)
    }
}

#[derive(Debug)]
pub struct Event {
    pub age: u64,
    pub id: Option<model::EventID>,
    pub data: EventData
}

#[derive(Debug)]
pub struct PresenceEvent {
    pub presence: String,
    pub user: model::UserID
}

impl Event {
    pub fn from_json(json: &Json) -> Self {
        let age = match json.find_path(&["unsigned", "age"]) {
            Some(a) => a.as_u64().unwrap(),
            None => 0
        };
        let tokens: Vec<&str> = mjson::string(json, "type").split(".").collect();
        let id = match json.as_object().unwrap().get("event_id") {
            Some(i) => Some(model::EventID::from_str(i.as_string().unwrap())),
            None => None
        };
        if tokens[0] != "m" {
            Event {
                age: age,
                id: id,
                data: EventData::Unknown(json.as_object().unwrap().get("type").unwrap().as_string().unwrap().to_string(), json.clone()),
            }
        } else {
            Event {
                age: age,
                id: id,
                data: match tokens[1] {
                    "room" =>
                        Self::from_room_json(tokens[2], json),
                    "typing" =>
                        EventData::Typing(TypingEvent {
                            users: vec![],
                            room: model::RoomID::from_str(mjson::string(json, "room_id"))
                        }),
                    "presence" =>
                        EventData::Presence(PresenceEvent{
                            presence: mjson::string(json, "content.presence").to_string(),
                            user: model::UserID::from_str(mjson::string(json, "sender"))
                        }),
                    e =>
                        EventData::Unknown(e.to_string(), json.clone())
                }
            }
        }
    }

    fn from_room_json(event_type: &str, json: &Json) -> EventData {

        if mjson::path(json, "content").as_object().unwrap().len() == 0 {
            // probably a redaction.
            return EventData::Room(
                model::RoomID::from_str(mjson::string(json, "room_id")),
                RoomEvent::Unknown(event_type.to_string(), json.clone())
            )
        }

        EventData::Room(
            model::RoomID::from_str(mjson::string(json, "room_id")),
            match event_type {
                "canonical_alias" =>
                    RoomEvent::CanonicalAlias(mjson::string(json, "content.alias").to_string()),
                "join_rules" => {
                        if json.find_path(&["content", "join_rules"]) == None {
                            RoomEvent::JoinRules(mjson::string(json, "content.join_rule").to_string())
                        } else {
                            RoomEvent::JoinRules(mjson::string(json, "content.join_rules").to_string())
                        }
                    },
                "member" =>
                    RoomEvent::Membership(model::UserID::from_str(mjson::string(json, "sender")), MembershipAction::from_str(mjson::string(json, "content.membership"))),
                "history_visibility" =>
                    RoomEvent::HistoryVisibility(mjson::string(json, "content.history_visibility").to_string()),
                "create" =>
                    RoomEvent::Create,
                "aliases" => {
                    let aliases = mjson::array(json, "content.aliases");
                    let mut alias_list: Vec<String> = vec![];
                    for alias in aliases {
                        alias_list.push(alias.as_string().unwrap().to_string());
                    }
                    RoomEvent::Aliases(alias_list)
                },
                "power_levels" =>
                    RoomEvent::PowerLevels,
                "message" =>
                    RoomEvent::Message(model::UserID::from_str(mjson::string(json, "sender")), mjson::string(json, "content.body").to_string()),
                "name" =>
                    RoomEvent::Name(model::UserID::from_str(mjson::string(json, "sender")), mjson::string(json, "content.name").to_string()),
                "topic" =>
                    RoomEvent::Topic(model::UserID::from_str(mjson::string(json, "sender")), mjson::string(json, "content.topic").to_string()),
                "avatar" =>
                    RoomEvent::Avatar(model::UserID::from_str(mjson::string(json, "sender")), mjson::string(json, "content.url").to_string()),
                unknown_type => RoomEvent::Unknown(unknown_type.to_string(), json.clone())
            }
        )
    }
}
