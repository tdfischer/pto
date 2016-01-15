use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Read;
use hyper;
use hyper::status::StatusCode;
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::fmt;
use std::thread;
use irc;
use matrix::json as mjson;
use matrix::events;

pub struct AsyncPoll {
    http: hyper::client::Client,
    url: hyper::Url
}

impl AsyncPoll {
    pub fn send(self) -> Vec<events::Event> {
        let req = self.http.get(self.url);
        let mut res = req.send().unwrap();
        let mut response = String::new();
        res.read_to_string(&mut response);
        println!("Response! {}", response);
        let mut ret: Vec<events::Event> = vec![];
        match Json::from_str(response.trim()) {
            Ok(js) => {
                let events = mjson::array(&js, "chunk");
                for ref evt in events {
                    ret.push(events::Event::from_json(evt))
                }
            },
            Err(e) => panic!(e)
        }
        ret
    }
}

#[derive(Clone)]
pub struct AccessToken {
    access: String,
    refresh: String
}

pub struct Client {
    http: hyper::Client,
    token: Option<AccessToken>,
    nextID: u32,
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
            token: None,
            nextID: 0
        }
    }

    pub fn login(&mut self, username: &str, password: &str) {
        let mut d = BTreeMap::new();
        d.insert("user".to_string(), Json::String(username.to_string()));
        d.insert("password".to_string(), Json::String(password.to_string()));
        d.insert("type".to_string(), Json::String("m.login.password".to_string()));
        println!("Logging in to matrix");
        let mut res = match self.http.post(self.url("login", &HashMap::new()))
            .body(Json::Object(d).to_string().trim()).send() {
                Ok(r) => r,
                Err(e) => panic!(e)
        };
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
        let mut ret = "http://localhost:8008/_matrix/client/api/v1/".to_string();
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

    pub fn pollAsync(&mut self) -> AsyncPoll {
        let url = self.url("events", &HashMap::new());
        AsyncPoll {
            http: hyper::client::Client::new(),
            url: url
        }
    }

    pub fn send(&mut self, evt: events::EventData) {
        self.nextID += 1;
        match evt {
            events::EventData::Room(ref id, _) => {
                let url = self.url(format!("rooms/{}/send/{}/{}", id, evt.type_str(), self.nextID).trim(), &HashMap::new());
                let req = match self.http.put(url).body(format!("{}", evt.to_json()).trim()).send() {
                    Ok(r) => r,
                    Err(e) => panic!(e)
                };
                println!("sent! {} {:?}", evt.to_json(), req);
            },
            _ => unreachable!()
        }
    }

    pub fn sync<F>(&mut self, callback: F)
            where F: Fn(events::Event) {
        println!("Syncing...");
        let mut args = HashMap::new();
        args.insert("limit", "0");
        let url = self.url("initialSync", &args);
        let mut res = match self.http.get(url).send() {
            Ok(r) => r,
            Err(e) => panic!(e)
        };
        let mut response = String::new();
        res.read_to_string(&mut response);
        match Json::from_str(response.trim()) {
            Ok(ref js) => {
                let rooms = mjson::array(js, "rooms");
                for ref r in rooms {
                    let roomState = mjson::array(r, "state");
                    let mut roomName: Option<String> = None;
                    for ref evt in roomState {
                        callback(events::Event::from_json(evt));
                    };
                }
            },
            Err(e) => panic!(e)
        }
    }
}
