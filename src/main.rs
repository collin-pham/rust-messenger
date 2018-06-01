extern crate websocket;
extern crate rust_messenger;
extern crate serde;
extern crate serde_json;
extern crate firebase;

use std::thread;
use websocket::OwnedMessage;
use websocket::sync::Server;
use rust_messenger::{db, users, threads, message, error};
use rust_messenger::message::{ Message };
use serde_json::{ Value, Error };
use firebase::{Firebase, Response};

// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";


fn main() {
//    let res = users::get_user("SQrF5Bw5FndZMFl7eU3DldBJrsj1", &firebase);
//    println!("{}", res.ok().unwrap().body)

//    let res = users::get_user_threads("SQrF5Bw5FndZMFl7eU3DldBJrsj1", 0, 4, &firebase);
//    println!("{:?}", res.ok().unwrap().body);
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

//    let res = threads::get_thread_messages("-LDiVOO2Sd86pSVAFvHD",0, 6, &firebase);
//    println!("{:?}", res.ok().unwrap().body);



//    let res = threads::create_thread(vec!["0", "1"], &firebase);
//    println!("{}", res.ok().unwrap().body)

//    let res = message::create_message("-LDiVOO2Sd86pSVAFvHD", new_message, &firebase);
//    println!("{}", res.ok().unwrap().body);



    let server = Server::bind(format!("{}:{}", IPADDRESS, PORT)).unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {

            let firebase = db::connect();

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
                        println!("JSON data {:?}", string);
                        let json_v: Value = serde_json::from_str(string.as_str()).unwrap();

                        take_action(json_v, &firebase);

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

    fn take_action(json_v: serde_json::Value, firebase: &Firebase) -> Result<Response, error::ServerError> {
        let action = match json_v.get("action") {
            Some(a) => a.as_str().unwrap(),
            None => return Err(error::ServerError::DatabaseFormatErr),
        };
        println!("Action is {}", action);

        let m_string = match json_v.get("message") {
            Some(m) => { m.to_string() },
            None => { println!("None value returned");
                      return Err(error::ServerError::DatabaseFormatErr) },
        };

        let new_mes: message::Message = match serde_json::from_str(m_string.as_str()) {
            Ok(d)  => { Some(d).unwrap() },
            Err(e) => { eprintln!("error {:?}", e);
                        return Err(error::ServerError::DatabaseFormatErr) },
        };

        println!("Message struct is {:?}", new_mes);

        if action == "sendMessage" {
            println!("sending message...");
            let thread_id = match json_v.get("thread_id") {
                Some(id) => id.as_str().unwrap(),
                None => { return Err(error::ServerError::DatabaseFormatErr) },
            };

//            let user = match json_v.get("user_id") {
//                Some(id) => id.as_str().unwrap(),
//                None => return Err(error::ServerError::DatabaseFormatErr),
//            };

            let res = match message::create_message(thread_id, new_mes, &firebase) {
                Ok(response) => response,
                Err(err) => return Err(error::ServerError::ReqNotJSON),
            };

            Ok(res)
        } else {
            println!("not matching correctly");
            Err(error::ServerError::BadRequest)
        }
    }
}