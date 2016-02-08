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

use std::collections::BTreeMap;
use std::collections::HashMap;
use hyper;
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::fmt;
use std::result;
use matrix::json as mjson;
use matrix::events;
use matrix::model;

enum ApiVersion {
    V1,
    V2Alpha
}

#[derive(Debug)]
pub enum ClientError {
    Http(hyper::Error),
    UrlNotFound,
    Json(json::ParserError)
}

pub type Result<T = ()> = result::Result<T, ClientError>;

mod http {
    use rustc_serialize::json::Json;
    use hyper;
    use std::io::Read;
    use matrix::client::{Result,ClientError};

    pub fn json(http: hyper::client::RequestBuilder) -> Result<Json> {
        let mut response = String::new();
        http.send().map_err(|err|{
            ClientError::Http(err)
        }).and_then(|mut res|{
            match res.status  {
                hyper::status::StatusCode::Ok =>  {
                    res.read_to_string(&mut response).expect("Could not read response");
                    Json::from_str(&response).map_err(|err|{
                        ClientError::Json(err)
                    })
                },
                _ => Err(ClientError::UrlNotFound)
            }
        })
    }
}

pub struct AsyncPoll {
    http: hyper::client::Client,
    url: hyper::Url
}

impl AsyncPoll {
    pub fn send(self) -> Result<Vec<events::Event>> {
        http::json(self.http.get(self.url)).and_then(|json| {
            let joined_rooms = mjson::path(&json, "rooms.join").as_object().unwrap();
            let mut ret: Vec<events::Event> = vec![];
            for (id, r) in joined_rooms {
                let room_state = mjson::array(r, "state.events");
                for ref evt in room_state {
                    trace!("<<< {}", evt);
                    let mut e = evt.as_object().unwrap().clone();
                    e.insert("room_id".to_string(), Json::String(id.clone()));
                    // FIXME: It'd be nice to return to the previous
                    // callback-based mechanism to avoid memory bloat
                    ret.push(events::Event::from_json(&Json::Object(e)));
                };
                let timeline = mjson::array(r, "timeline.events");
                for ref evt in timeline {
                    trace!("<<< {}", evt);
                    let mut e = evt.as_object().unwrap().clone();
                    e.insert("room_id".to_string(), Json::String(id.clone()));
                    // FIXME: It'd be nice to return to the previous
                    // callback-based mechanism to avoid memory bloat
                    ret.push(events::Event::from_json(&Json::Object(e)));
                };
            }
            Ok(ret)
        })
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
    next_id: u32,
    baseurl: hyper::Url,
    pub uid: Option<model::UserID>
}

impl fmt::Debug for Client {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl Client {
    pub fn new(baseurl: hyper::Url) -> Self {
        if !baseurl.scheme.starts_with("https") {
            warn!("YOU ARE CONNECTING TO A MATRIX SERVER WITHOUT SSL");
        }
        let mut http = hyper::Client::new();
        http.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);
        Client {
            http: http,
            token: None,
            next_id: 0,
            baseurl: baseurl,
            uid: None
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result {
        let mut d = BTreeMap::new();
        d.insert("user".to_string(), Json::String(username.to_string()));
        d.insert("password".to_string(), Json::String(password.to_string()));
        d.insert("type".to_string(), Json::String("m.login.password".to_string()));
        debug!("Logging in to matrix");
        http::json(self.http.post(self.url(ApiVersion::V1, "login", &HashMap::new()))
            .body(&Json::Object(d).to_string()))
            .and_then(|js| {
                let obj = js.as_object().unwrap();
                self.token = Some(AccessToken {
                    access: obj.get("access_token").unwrap().as_string().unwrap().to_string(),
                    refresh: obj.get("refresh_token").unwrap().as_string().unwrap().to_string()
                });
                let domain = self.baseurl.host().unwrap().serialize();
                self.uid = Some(model::UserID::from_str(&format!("@{}:{}", username, domain)));
                Ok(())
            })
    }

    fn url(&self, version: ApiVersion, endpoint: &str, args: &HashMap<&str, &str>) -> hyper::Url {
        let mut ret = self.baseurl.clone();
        ret.path_mut().unwrap().append(&mut vec!["client".to_string()]);
        ret.path_mut().unwrap().append(&mut match version {
            ApiVersion::V1 =>
                vec!["api".to_string(), "v1".to_string()],
            ApiVersion::V2Alpha =>
                vec!["v2_alpha".to_string()]
        });
        ret.path_mut().unwrap().push(endpoint.to_string());
        let args_with_auth = match self.token {
            None => args.clone(),
            Some(ref token) => {
                let mut a = args.clone();
                a.insert("access_token", &*token.access);
                a
            }
        };
        ret.set_query_from_pairs(args_with_auth);
        ret
    }

    pub fn poll_async(&mut self) -> AsyncPoll {
        let url = self.url(ApiVersion::V2Alpha, "sync", &HashMap::new());
        let mut http = hyper::client::Client::new();
        http.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);
        AsyncPoll {
            http: http,
            url: url
        }
    }

    pub fn send(&mut self, evt: events::EventData) -> Result<model::EventID> {
        self.next_id += 1;
        match evt {
            events::EventData::Room(ref id, _) => {
                let url = self.url(ApiVersion::V1, &format!("rooms/{}/send/{}/{}",
                                           id,
                                           evt.type_str(),
                                           self.next_id),
                                   &HashMap::new());
                trace!("Sending events to {:?}", url);
                // FIXME: This seems needed since hyper will pool HTTP client
                // connections for pipelining. Sometimes the server will close
                // the pooled connection and everything will catch on fire here.
                let mut http = hyper::client::Client::new();
                http.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);
                http::json(http.put(url).body(&format!("{}", evt.to_json())))
            },
            _ => panic!("Don't know where to send {}", evt.to_json())
        }.and_then(|response| {
            trace!(">>> {} {:?}", evt.to_json(), response);
            Ok(model::EventID::from_str(mjson::string(&response, "event_id")))
        })
    }

    pub fn sync(&mut self) -> Result<Vec<events::Event>> {
        debug!("Syncing...");
        let mut args = HashMap::new();
        args.insert("full_state", "true");
        let url = self.url(ApiVersion::V2Alpha, "sync", &args);
        http::json(self.http.get(url)).and_then(|js| {
            println!("Got Sync JS {}", js.pretty());
            let joined_rooms = mjson::path(&js, "rooms.join").as_object().unwrap();
            let mut ret: Vec<events::Event> = vec![];
            for (id, r) in joined_rooms {
                let room_state = mjson::array(r, "state.events");
                for ref evt in room_state {
                    trace!("<<< {}", evt);
                    let mut e = evt.as_object().unwrap().clone();
                    e.insert("room_id".to_string(), Json::String(id.clone()));
                    // FIXME: It'd be nice to return to the previous
                    // callback-based mechanism to avoid memory bloat
                    ret.push(events::Event::from_json(&Json::Object(e)));
                };
            }
            ret.push(events::Event {
                data: events::EventData::EndOfSync,
                id: None
            });
            Ok(ret)
        })
    }
}
