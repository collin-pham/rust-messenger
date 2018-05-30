extern crate websocket;
extern crate rust_messenger;
extern crate serde_json;

use std::thread;
use websocket::OwnedMessage;
use websocket::sync::Server;
use rust_messenger::{db, users, threads, message};
use rust_messenger::message::{ Message };
use serde_json::{ Value, Error };

// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";


fn main() {
    let firebase = db::connect();

//    let res = users::get_user("SQrF5Bw5FndZMFl7eU3DldBJrsj1", &firebase);
//    println!("{}", res.ok().unwrap().body)

//    let res = users::get_user_threads("SQrF5Bw5FndZMFl7eU3DldBJrsj1", 1, 5, &firebase);
//    println!("{}", res.ok().unwrap().body);
//
//    let new_message = Message {
//        user_id: "0".to_owned(),
//        timestamp: 100,
//        contents: "This Is A Test Message".to_owned(),
//        read: false,
//    };
//    let res = users::update_user_threads("SQrF5Bw5FndZMFl7eU3DldBJrsj1", "6", new_message, &firebase);
//    println!("{}", res.ok().unwrap().body)
//
//    let res = threads::get_thread_user_ids("-LDiVOO2Sd86pSVAFvHD", &firebase);
//    println!("{}", res.ok().unwrap().body);

//    let res = threads::get_thread_messages("-LDiVOO2Sd86pSVAFvHD",0, 3, &firebase);
//    println!("{}", res.ok().unwrap().body);

//    let res = threads::create_thread(vec!["0", "1"], &firebase);
//    println!("{}", res.ok().unwrap().body)
//
//    let res = message::create_message("-LDiVOO2Sd86pSVAFvHD", new_message, &firebase);
//    println!("{}", res.ok().unwrap().body);



    let server = Server::bind(format!("{}:{}", IPADDRESS, PORT)).unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            let mut client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            let message = OwnedMessage::Text("Hello".to_string());
            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Text(string) => {

                        //get action type from JSON data
                        let v: Value = serde_json::from_str(string.as_str()).unwrap();
                        let action = match v["action"] {
                            Value::String(ref act) => act,
                            _ => ""
                        };
                        println!("Action is {}", action);

                        if action == "sendMessage" {
                            println!("checking threadIDs to get or create new message");
                            //if thread ID, call newMessage
                            //let thread = whatever passed,
                            //if thread is none, call createThread, get thread, call newMessage
                            //if not thread ID, make new thread, get thread, call newMessage on thread
                        }
                        //other actions here

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