use std::collections::BTreeMap;
use std::io::Read;
use hyper;
use hyper::status::StatusCode;
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::fmt;
use std::thread;
use mio;
use irc;

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
        let mut res = self.http.post("https://oob.systems/_matrix/client/api/v1/login")
            .body(Json::Object(d).to_string().trim()).send().unwrap();
        assert_eq!(res.status, StatusCode::Ok);
        let mut response = String::new();
        res.read_to_string(&mut response);
        match Json::from_str(response.trim()) {
            Ok(js) => {
                let obj = js.as_object().unwrap();
                println!("decode: {:?}", obj);
                self.token = Some(AccessToken {
                    access: obj.get("access_token").unwrap().as_string().unwrap().to_string(),
                    refresh: obj.get("refresh_token").unwrap().as_string().unwrap().to_string()
                })
            },
            Err(e) => panic!(e)
        }
    }

    pub fn pollAsync(&mut self, channel: mio::Sender<irc::protocol::Message>) {
        let url = "https://oob.systems/_matrix/client/api/v1/events?access_token=".to_string() + &self.token.clone().unwrap().access;
        println!("poll?");
        let http = hyper::Client::new();
        thread::spawn(move||{
            println!("Pulling {:?}", url);
            let mut res = http.get(url.trim()).send().unwrap();

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
        let mut res = self.http.get(("https://oob.systems/_matrix/client/api/v1/initialSync?limit=0&access_token=".to_string() + &self.token.clone().unwrap().access).trim())
            .send().unwrap();
        let mut response = String::new();
        res.read_to_string(&mut response);
        match Json::from_str(response.trim()) {
            Ok(js) => {
                let obj = js.as_object().unwrap();
                let rooms = obj.get("rooms").unwrap().as_array().unwrap();
                for r in rooms {
                    let room = r.as_object().unwrap();
                    let roomState = room.get("state").unwrap().as_array().unwrap();
                    let mut roomName: Option<String> = None;
                    for ref s in roomState {
                        let state = s.as_object().unwrap();
                        match state.get("type").unwrap().as_string() {
                            Some("m.room.canonical_alias") => {
                                println!("alias! {:?}", state);
                                roomName = Some(state.get("content").unwrap().as_object().unwrap().get("alias").unwrap().as_string().unwrap().to_string());
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
                        args: vec![roomName.unwrap()],
                        suffix: None
                    };
                    channel.send(msg);
                }
                println!("State: {:?}", obj);
            },
            Err(e) => panic!(e)
        }
    }
}
