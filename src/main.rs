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

// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";


/// Initiates the websocket server connection. Creates a new thread for each
/// user connected. Stores a list of connected users in an `Arc<Mutex<HashMap<_>>>`
/// linking a username to their unique ID.
fn main() {
    println!("Binding to server...");
    let server = Server::bind(format!("{}:{}", IPADDRESS, PORT)).unwrap();

    println!("Creating connected Users...");
    let connected_users: Arc<Mutex<HashMap<String, Writer<TcpStream>>>> = Arc::new(Mutex::new(HashMap::new()));

    println!("Consuming Incoming Requests...");
    for request in server.filter_map(Result::ok) {

        println!("Getting UserId...");
        let user_id = match &request.request.subject.1 {
            &hyper::uri::RequestUri::AbsolutePath(ref path) => {
                let user_id = str::replace(&path, "/?user_id=", "");
                println!("{:?}", user_id);
                user_id
            }
            _ => { "TODO: reject websocket connection".to_owned() }
        };
        let clone = connected_users.clone();

        thread::spawn(move || {
            println!("Establishing Firebase Connection...");
            let firebase = db::connect();

            println!("Checking If There Is The Correct Protocol...");
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            println!("Selecting The Correct Protocol...");
            let client = request.use_protocol("rust-websocket").accept().unwrap();

            println!("Obtaining IpAddress...");
            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            println!("Parsing Receiver and Sender...");
            let (mut receiver, sender) = client.split().unwrap();

            println!("Inserting User Into HashMap...");
            match clone.lock() {
                Ok(mut map) => {
                    map.insert(user_id.clone(), sender);
                }
                Err(err) => {println!("Error locking Mutex {:?}", err)}
            }

            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(message) => { message }
                    Err(err)    => {
                        println!("No Incoming Message {:?}", err);
                        break;
                    }
                };

                match message {
                    OwnedMessage::Text(string) => {
                        println!("Turning data into json...");
                        let req: protocol::Request = serde_json::from_str(string.as_str()).unwrap();

                        match protocol::take_action(&req, &firebase, &user_id, &clone) {
                            Ok(res) => {
                                let reply = serde_json::to_string(&res).unwrap();
                                let message = OwnedMessage::Text(reply);
                                println!("Send Message To Client");
                                match clone.lock() {
                                    Ok(mut map) => {
                                        match map.get_mut(&user_id) {
                                            Some(receiver) => {
                                                match receiver.send_message(&message) {
                                                    Ok(success) => {println!("{:?}", success)}
                                                    Err(err) => {println!("error sending message during call to take_action {:?}", err)}
                                                }
                                            }
                                            None => {println!("User: {} Is Not Connected", &user_id)}
                                        }
                                    }
                                    Err(err) => {println!("Error locking Mutex {:?} during call to take_action", err)}
                                }
                            }
                            Err(_)  => panic!("Thread encountered an error!"),
                        }
                    }

                    OwnedMessage::Close(_) => {
                        println!("Attempting To Disconnect Client");
                        let message = OwnedMessage::Close(None);
                        match clone.lock() {
                            Ok(mut map) => {
                                match map.get_mut(&user_id) {
                                    Some(receiver) => {
                                        match receiver.send_message(&message) {
                                            Ok(success) => {println!("{:?}", success)}
                                            Err(err) => {println!("error sending discconnect message: {:?}", err)}
                                        }
                                    }
                                    None => {println!("User: {} Is Already Disconnected", &user_id)}
                                }

                            }
                            Err(err) => {println!("Error locking Mutex {:?} when trying to disconnect user {}", err, &user_id)}
                        }

                        match clone.lock() {
                            Ok(mut map) => {
                                match map.remove(&user_id) {
                                    Some(_) => { println!("Successfully removed receiver {}", &user_id)}
                                    None => {println!("User: {} Is Already Disconnected", &user_id)}
                                }

                            }
                            Err(err) => {println!("Error locking Mutex {:?} when trying to remove user {}", err, &user_id)}
                        }
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        println!("Ping Ping Ping");
                        let message = OwnedMessage::Pong(ping);
                        clone.lock().unwrap().get_mut(&user_id)
                            .unwrap().send_message(&message).unwrap();
                    }
                    _ => {
                        println!("This is happening");
                        clone.lock().unwrap().get_mut(&user_id)
                        .unwrap().send_message(&message).unwrap();
                    },
                }
            }
        });
    }
}
