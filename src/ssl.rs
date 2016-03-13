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
use irc::streams::{Server, Client, AsEvented};
use mio::tcp::TcpListener;
use mio::Evented;
use openssl::ssl::{SslContext, SslStream};
use std::net::SocketAddr;

pub struct TcpServer {
    listener: TcpListener
}

impl TcpServer {
    pub fn new(addr: &SocketAddr) -> Self {
        TcpServer {
            listener: TcpListener::bind(addr).unwrap()
        }
    }
}

pub struct SslServer {
    listener: TcpListener,
    ssl: SslContext
}

impl SslServer {
    pub fn new(addr: &SocketAddr, ssl: SslContext) -> Self {
        SslServer {
            listener: TcpListener::bind(addr).unwrap(),
            ssl: ssl
        }
    }
}

impl AsEvented for SslServer {
    fn as_evented(&self) -> &Evented {
        &self.listener
    }
}

impl AsEvented for TcpServer {
    fn as_evented(&self) -> &Evented {
        &self.listener
    }
}

impl Server for SslServer {
    fn accept(&mut self) -> Option<Client> {
         match self.listener.accept() {
             Ok(None) => None,
             Ok(Some((socket, _))) => {
                 // FIXME: This should be in a separate thread to protect against openssl vulns
                 match SslStream::accept(&self.ssl, socket) {
                     Ok(ssl) =>
                         Some(Client::new(Box::new(ssl))),
                    _ => None
                 }
             },
             Err(e) => panic!(e),
         }
    }
}

impl Server for TcpServer {
    fn accept(&mut self) -> Option<Client> {
         match self.listener.accept() {
             Ok(None) => None,
             Ok(Some((socket, _))) => {
                 Some(Client::new(Box::new(socket)))
             },
             Err(e) => panic!(e),
         }
    }
}
