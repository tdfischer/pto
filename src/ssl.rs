use irc::streams::{Server, Client};
use mio::tcp::TcpListener;
use openssl::ssl::{SslContext, SslStream};
use std::net::SocketAddr;

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

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

impl Server for SslServer {
    fn accept(&mut self) -> Option<Client> {
         match self.listener.accept() {
             Ok(None) => None,
             Ok(Some((socket, _))) => {
                 let ssl = SslStream::accept(&self.ssl, socket).expect("Could not construct SSL stream");
                 Some(Client::new(ssl))
             },
             Err(e) => panic!(e),
         }
    }
}
