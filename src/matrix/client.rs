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

mod http {
    use rustc_serialize::json::{Json, ParserError};
    use hyper;
    use std::io::Read;

    pub fn json(http: hyper::client::RequestBuilder) -> Result<Json, ParserError> {
        let mut response = String::new();
        http.send().unwrap().read_to_string(&mut response);
        Json::from_str(response.trim())
    }
}

pub struct AsyncPoll {
    http: hyper::client::Client,
    url: hyper::Url
}

impl AsyncPoll {
    pub fn send(self) -> Vec<events::Event> {
        let json = http::json(self.http.get(self.url)).unwrap();

        println!("Response! {:?}", json);
        let mut ret: Vec<events::Event> = vec![];
        let events = mjson::array(&json, "chunk");
        for ref evt in events {
            ret.push(events::Event::from_json(evt))
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
    baseurl: String
}

impl fmt::Debug for Client {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl Client {
    pub fn new(baseurl: &str) -> Self {
        Client {
            http: hyper::Client::new(),
            token: None,
            nextID: 0,
            baseurl: baseurl.to_string()
        }
    }

    pub fn login(&mut self, username: &str, password: &str) {
        let mut d = BTreeMap::new();
        d.insert("user".to_string(), Json::String(username.to_string()));
        d.insert("password".to_string(), Json::String(password.to_string()));
        d.insert("type".to_string(), Json::String("m.login.password".to_string()));
        println!("Logging in to matrix");
        let js = http::json(self.http.post(self.url("login", &HashMap::new()))
            .body(Json::Object(d).to_string().trim())).unwrap();
        let obj = js.as_object().unwrap();
        self.token = Some(AccessToken {
            access: obj.get("access_token").unwrap().as_string().unwrap().to_string(),
            refresh: obj.get("refresh_token").unwrap().as_string().unwrap().to_string()
        })
    }

    fn url(&self, endpoint: &str, args: &HashMap<&str, &str>) -> hyper::Url {
        let mut ret = self.baseurl.clone();
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

    pub fn send(&mut self, evt: events::EventData) -> events::EventID {
        self.nextID += 1;
        let response = match evt {
            events::EventData::Room(ref id, _) => {
                let url = self.url(format!("rooms/{}/send/{}/{}", id, evt.type_str(), self.nextID).trim(), &HashMap::new());
                http::json(self.http.put(url).body(format!("{}", evt.to_json()).trim()))
            },
            _ => unreachable!()
        }.unwrap();
        println!("sent! {} {:?}", evt.to_json(), response);
        events::EventID::from_str(mjson::string(&response, "event_id"))
    }

    pub fn sync<F>(&mut self, callback: F)
            where F: Fn(events::Event) {
        println!("Syncing...");
        let mut args = HashMap::new();
        args.insert("limit", "0");
        let url = self.url("initialSync", &args);
        let js = http::json(self.http.get(url)).unwrap();
        let rooms = mjson::array(&js, "rooms");
        for ref r in rooms {
            let roomState = mjson::array(r, "state");
            let mut roomName: Option<String> = None;
            for ref evt in roomState {
                callback(events::Event::from_json(evt));
            };
        }
    }
}
