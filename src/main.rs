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
use websocket::sender::Writer;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use serde_json::Value;


// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";


fn main() {

    let server = Server::bind(format!("{}:{}", IPADDRESS, PORT)).unwrap();

    let connected_users: Arc<Mutex<HashMap<String, Writer<TcpStream>>>> = Arc::new(Mutex::new(HashMap::new()));

    for request in server.filter_map(Result::ok) {

        let user_id = match &request.request.subject.1 {
            hyper::uri::RequestUri::AbsolutePath(path) => {
                let user_id = str::replace(&path, "/?user_id=", "");
                println!("{:?}", user_id);
                user_id
            }
            _ => { "TODO: reject websocket connection".to_owned() }
        };
        let clone = connected_users.clone();

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

            clone.lock().unwrap().insert(user_id.clone(), sender);

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

                        if action == "send_message" {
                            match clone.lock().unwrap().get_mut("19yW68EJ8eOW6csXxs0V25Q9PoK2") {
                                Some(receiver) => {
                                    let message = OwnedMessage::Text("{\"message\":\"This is from Safari\"}".to_owned());
                                    receiver.send_message(&message);
                                }
                                None => {println!("User not connected!")}
                            }
                        }
                        
                        match protocol::take_action(&action, &json_v, &firebase) {
                            Ok(res) => {
                                let reply = serde_json::to_string(&res).unwrap();
                                println!("Reply to frontend is {:?}", reply);
                                let message = OwnedMessage::Text(reply);
                                clone.lock().unwrap().get_mut(&user_id)
                                      .unwrap().send_message(&message).unwrap();
//                                  sender.send_message(&message).unwrap();
                                }
                            Err(_)  => panic!("Thread encountered an error!"),
                        }
                    }

                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        clone.lock().unwrap().get_mut(&user_id)
                            .unwrap().send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        clone.lock().unwrap().get_mut(&user_id)
                            .unwrap().send_message(&message).unwrap();
                    }
                    _ => { clone.lock().unwrap().get_mut(&user_id)
                        .unwrap().send_message(&message).unwrap(); },
                }
            }
        });
    }
}