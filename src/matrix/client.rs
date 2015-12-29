use std::collections::BTreeMap;
use std::io::Read;
use hyper;
use hyper::status::StatusCode;
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::collections::HashMap;
use std::fmt;
use std::thread;
use mio;
use irc;
use matrix::json as mjson;

#[derive(Clone)]
pub struct AccessToken {
    access: String,
    refresh: String
}

pub struct Client {
    http: hyper::Client,
    token: Option<AccessToken>
}

impl fmt::Debug for Client {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl Client {
    pub fn new() -> Self {
        Client {
            http: hyper::Client::new(),
            token: None
        }
    }

    pub fn login(&mut self, username: &str, password: &str) {
        let mut d = BTreeMap::new();
        d.insert("user".to_string(), Json::String(username.to_string()));
        d.insert("password".to_string(), Json::String(password.to_string()));
        d.insert("type".to_string(), Json::String("m.login.password".to_string()));
        println!("Logging in to matrix");
        let mut res = self.http.post(self.url("login", &HashMap::new()))
            .body(Json::Object(d).to_string().trim()).send().unwrap();
        assert_eq!(res.status, StatusCode::Ok);
        let mut response = String::new();
        res.read_to_string(&mut response);
        match Json::from_str(response.trim()) {
            Ok(js) => {
                let obj = js.as_object().unwrap();
                self.token = Some(AccessToken {
                    access: obj.get("access_token").unwrap().as_string().unwrap().to_string(),
                    refresh: obj.get("refresh_token").unwrap().as_string().unwrap().to_string()
                })
            },
            Err(e) => panic!(e)
        }
    }

    fn url(&self, endpoint: &str, args: &HashMap<&str, &str>) -> hyper::Url {
        let mut ret = "https://oob.systems/_matrix/client/api/v1/".to_string();
        ret.push_str(endpoint);
        ret.push_str("?");
        match self.token {
            None => (),
            Some(ref token) => {
                ret.push_str("access_token=");
                ret.push_str(token.access.trim());
                ret.push_str("&");
            }
        }
        for (name, value) in args {
            ret.push_str(name);
            ret.push_str("=");
            ret.push_str(value);
            ret.push_str("&");
        }
        hyper::Url::parse(ret.trim()).unwrap()
    }

    pub fn pollAsync(&mut self, channel: mio::Sender<irc::protocol::Message>) {
        let url = self.url("events", &HashMap::new());
        let http = hyper::Client::new();
        thread::spawn(move||{
            let mut res = http.get(url).send().unwrap();

            let msg = irc::protocol::Message {
                command: irc::protocol::Command::Join,
                prefix: Some("tdfischer@tdfischer@anony.oob".to_string()),
                args: vec!["#pto".to_string()],
                suffix: None
            };
            channel.send(msg);
        });
    }

    pub fn sync(&mut self, channel: mio::Sender<irc::protocol::Message>) {
        println!("Syncing...");
        let mut args = HashMap::new();
        args.insert("limit", "0");
        let url = self.url("initialSync", &args);
        let mut res = self.http.get(url).send().unwrap();
        let mut response = String::new();
        res.read_to_string(&mut response);
        match Json::from_str(response.trim()) {
            Ok(js) => {
                let rooms = mjson::array(&js, "rooms");
                for r in rooms {
                    let roomState = mjson::array(r, "state");
                    let mut roomName: Option<&str> = None;
                    for ref s in roomState {
                        let state = mjson::string(s, "type").trim();
                        match state {
                            "m.room.canonical_alias" => {
                                roomName = Some(mjson::string(s, "content.alias"));
                            },
                            _ => ()
                        };
                    };
                    if roomName == None {
                        continue;
                    }
                    let msg = irc::protocol::Message {
                        command: irc::protocol::Command::Join,
                        prefix: Some("tdfischer!tdfischer@anony.oob".to_string()),
                        args: vec![roomName.unwrap().to_string()],
                        suffix: None
                    };
                    channel.send(msg);
                }
            },
            Err(e) => panic!(e)
        }
    }
}
