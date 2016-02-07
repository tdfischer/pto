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
extern crate rustc_serialize;
extern crate hyper;
extern crate mio;
extern crate env_logger;
extern crate openssl;
extern crate c_ares;
extern crate libc;
#[macro_use]
extern crate log;
mod irc;
mod matrix;
mod bridge;
mod ssl;
mod dns;
use mio::{EventLoop,Handler,Token,EventSet,PollOpt};
use std::thread;
use bridge::Bridge;
use std::env;
use std::path::Path;
use std::net::SocketAddr;
use openssl::ssl::{SslContext, SslMethod};
use openssl::x509::X509FileType;
use irc::streams::{Server, AsEvented};

struct IrcHandler {
    server: Box<Server>,
    url: hyper::Url
}

impl Handler for IrcHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<IrcHandler>, token: Token, _: EventSet) {
        match token {
            SERVER => {
                match self.server.accept() {
                    Some(client) => {
                        let mut bridge = Bridge::new(client, self.url.clone());
                        thread::spawn(move||{
                            bridge.run()
                        });
                    },
                    None => ()
                }
            },
            _ => unreachable!("Got a really weird token in the server event loop!")
        }
    }
}

const SERVER: Token = Token(0);

fn main() {
    env_logger::init().unwrap();
    let addr: SocketAddr = match env::args().nth(2) {
        Some(arg) => arg,
        None => "127.0.0.1:8001".to_string()
    }.parse().unwrap();

    let domain =  env::args().nth(1).unwrap();
    let url = match dns::probe_url(&*domain) {
        Some(u) => u,
        None => hyper::Url::parse(&*domain).unwrap()
    };

    let is_loopback = match addr {
        SocketAddr::V4(ref a) => {
            a.ip().octets() == [127, 0, 0, 1]
        },
        _ => true
    };
    let server: Box<Server> = if !is_loopback {
        let mut ssl = SslContext::new(SslMethod::Sslv23).expect("SSL setup failed");
        ssl.set_certificate_file(Path::new("pto.crt"), X509FileType::PEM).expect("Could not load pto.crt");
        ssl.set_private_key_file(Path::new("pto.key"), X509FileType::PEM).expect("Could not load pto.key");
        Box::new(ssl::SslServer::new(&addr, ssl))
    } else {
        warn!("SSL IS DISABLED!! IRC LOGINS ARE IN THE CLEAR AND UNENCRYPTED");
        Box::new(ssl::TcpServer::new(&addr))
    };
    info!("Listening on {}", addr);
    info!("Using matrix URL at {}", url);
    let mut events = EventLoop::new().unwrap();
    events.register(server.as_evented(), SERVER, EventSet::all(), PollOpt::edge()).unwrap();
    events.run(&mut IrcHandler{
        server: server,
        url: url
    }).unwrap();
}
