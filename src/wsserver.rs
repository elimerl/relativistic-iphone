use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn server(channel: mpsc::Sender<Acceleration>) {
    let server = Server::bind("127.0.0.1:3005").unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        let mut client = request.accept().unwrap();

        let ip = client.peer_addr().unwrap();

        println!("Connection from {}", ip);

        let message = OwnedMessage::Text("Hello".to_string());

        let (mut receiver, mut sender) = client.split().unwrap();

        for message in receiver.incoming_messages() {
            let message = message.unwrap();
            match message {
                OwnedMessage::Close(_) => {
                    let message = OwnedMessage::Close(None);
                    sender.send_message(&message).unwrap();
                    println!("Client {} disconnected", ip);
                    return;
                }
                OwnedMessage::Ping(ping) => {
                    let message = OwnedMessage::Pong(ping);
                    sender.send_message(&message).unwrap();
                }
                OwnedMessage::Text(text) => {
                    if let Ok(data) = (serde_json::from_str::<Acceleration>(&text)) {
                        channel.send(data).unwrap();
                    }
                }
                _ => sender.send_message(&message).unwrap(),
            }
        }
    }
}
