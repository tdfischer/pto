mod irc;
use std::thread;

fn handle_client(mut client: irc::Client) {
    loop {
        let message = client.read_message().unwrap();
        println!("Got a message! {:?}", message);
        match message.command {
            irc::Command::Nick => {
                client.set_nickname(message.args[0].clone())
            },
            irc::Command::User => {
                println!("User logged in: {}", message.args[0]);
                client.set_username(message.args[0].clone());
                client.welcome("Welcome!");
                client.join("#pto");
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
