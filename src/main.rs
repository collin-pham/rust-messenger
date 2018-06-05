extern crate rust_messenger;

extern crate websocket;
extern crate serde;
extern crate firebase;
extern crate hyper;
extern crate serde_json;


use std::thread;
use rust_messenger::{db, protocol};
use websocket::OwnedMessage;
use websocket::sync::Server;
use serde_json::Value;


// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";


fn main() {

    let server = Server::bind(format!("{}:{}", IPADDRESS, PORT)).unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {

            let firebase = db::connect();

            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            let client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Text(string) => {
                        println!("JSON data {:?}", string);
                        let json_v: Value = serde_json::from_str(string.as_str()).unwrap();

                        let action = match json_v.get("action") {
                            Some(a) => a.as_str().unwrap(),
                            None => return,
                        };

                        match protocol::take_action(&action, &json_v, &firebase) {
                            Ok(res) =>
                                { let reply = serde_json::to_string(&res).unwrap();
                                  println!("Reply to frontend is {:?}", reply);
                                  let message = OwnedMessage::Text(reply);
                                  sender.send_message(&message).unwrap();
                                }
                            Err(_)  => panic!("Thread encountered an error!"),
                        }
                    }

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
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }
}