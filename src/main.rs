extern crate rustc_serialize;
extern crate hyper;
mod irc;
mod matrix;
use std::thread;

fn handle_client(mut client: irc::Client) {
    let mut matrix_client = matrix::Client::new();
    let mut password: Option<String> = None;
    let mut username: Option<String> = None;
    loop {
        let message = client.read_message().unwrap();
        println!("Got a message! {:?}", message);
        match message.command {
            irc::Command::Pass => {
                password = Some(message.args[0].clone());
            }
            irc::Command::Nick => {
                client.set_nickname(message.args[0].clone());
            },
            irc::Command::User => {
                println!("User logged in: {}", message.args[0]);
                client.set_username(message.args[0].clone());
                username = Some(message.args[0].clone());
                matrix_client.login(username.unwrap().trim(), password.clone().unwrap().trim());
                password = None;
                client.welcome("Welcome!");
                matrix_client.sync();
            },
            irc::Command::Join => {
                client.join(&message.args[0]);
            },
            irc::Command::Ping => {
                client.pong();
            },
            irc::Command::Quit => {
                return;
            },
            _ =>
                println!("unhandled {:?}", message)
        }
    }
}

fn main() {
    let server = irc::IrcServer::new();
    println!("Listening on 127.0.0.1:8001");
    for client in server.iter_new_clients() {
        println!("Got a client! {:?}", client);
        thread::spawn(move|| {
            handle_client(client)
        });
    }
}
