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
                 let ssl = SslStream::accept(&self.ssl, socket).expect("Could not construct SSL stream");
                 Some(Client::new(Box::new(ssl)))
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
