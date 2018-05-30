extern crate websocket;
extern crate rust_messenger;

use std::thread;
use websocket::OwnedMessage;
use websocket::sync::Server;
use rust_messenger::{db, users, threads, message};
use rust_messenger::message::{ Message };
// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";


fn main() {
    let firebase = db::connect();

//    let res = users::get_user("SQrF5Bw5FndZMFl7eU3DldBJrsj1", &firebase);
//    println!("{}", res.ok().unwrap().body)

//    let res = db::get_user_threads("SQrF5Bw5FndZMFl7eU3DldBJrsj1", 1, 5, &firebase);
//    println!("{}", res.ok().unwrap().body)
//
//    let new_message = Message {
//        user_id: "0".to_owned(),
//        timestamp: 100,
//        contents: "This Is A Test Message".to_owned(),
//        read: false,
//    };
//    let res = users::update_user_threads("SQrF5Bw5FndZMFl7eU3DldBJrsj1", "6", new_message, &firebase);
//    println!("{}", res.ok().unwrap().body)

    let res = threads::get_thread("test_thread_id", &firebase);
    println!("{}", res.ok().unwrap().body)

//    let res = threads::create_thread(vec!["0", "1"], &firebase);
//    println!("{}", res.ok().unwrap().body)

//    let res = message::create_message("-LDiVOO2Sd86pSVAFvHD", new_message, &firebase);
//    println!("{}", res.ok().unwrap().body);





//    let server = Server::bind(format!("{}:{}", IPADDRESS, PORT)).unwrap();
//
//    for request in server.filter_map(Result::ok) {
//        // Spawn a new thread for each connection.
//        thread::spawn(move || {
//            if !request.protocols().contains(&"rust-websocket".to_string()) {
//                request.reject().unwrap();
//                return;
//            }
//
//            let mut client = request.use_protocol("rust-websocket").accept().unwrap();
//
//            let ip = client.peer_addr().unwrap();
//
//            println!("Connection from {}", ip);
//
//            let message = OwnedMessage::Text("Hello".to_string());
//            client.send_message(&message).unwrap();
//
//            let (mut receiver, mut sender) = client.split().unwrap();
//
//            for message in receiver.incoming_messages() {
//                let message = message.unwrap();
//                match message {
//                    OwnedMessage::Close(_) => {
//                        let message = OwnedMessage::Close(None);
//                        sender.send_message(&message).unwrap();
//                        println!("Client {} disconnected", ip);
//                        return;
//                    }
//                    OwnedMessage::Ping(ping) => {
//                        let message = OwnedMessage::Pong(ping);
//                        sender.send_message(&message).unwrap();
//                    }
//                    _ => sender.send_message(&message).unwrap(),
//                }
//            }
//        });
//    }
}