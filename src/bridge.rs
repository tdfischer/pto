use irc;
use irc::protocol::Command;

pub fn handle_client(client: &mut irc::streams::Client) {
    let message = client.read_message().unwrap();
    println!("Got a message! {:?}", message);
    match message.command {
        Command::Pass => {
            client.auth.set_password(message.args[0].clone())
        }
        Command::Nick => {
            client.set_nickname(message.args[0].clone());
        },
        Command::User => {
            println!("User logged in: {}", message.args[0]);
            client.auth.set_username(message.args[0].clone());
            let auth = client.auth.consume();
            client.matrix.login(auth.username.unwrap().trim(), auth.password.unwrap().trim());
            client.welcome("Welcome!");
            matrix_client.sync();
        },
        Command::Join => {
            client.join(&message.args[0]);
        },
        Command::Ping => {
            client.pong();
        },
        Command::Quit => {
            return;
        },
        _ =>
            println!("unhandled {:?}", message)
    }
}
