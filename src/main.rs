extern crate websocket;
extern crate rust_messenger;
extern crate serde;
extern crate firebase;
extern crate hyper;

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::thread;
use websocket::OwnedMessage;
use websocket::sync::Server;
use rust_messenger::{db, users, threads, message, error};
use serde_json::{ Value };
use firebase::{Firebase, Response};

// GLOBALS //
const IPADDRESS  : &str = "127.0.0.1";
const PORT       : &str = "8080";

#[derive(Serialize, Deserialize, Debug)]
pub struct Reply {
    pub action: String,
    pub body:   String,
    pub code:   u32,
}


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

//            let message = OwnedMessage::Text("Hello".to_string());
//            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Text(string) => {

                        //get action type from JSON data
                        println!("JSON data {:?}", string);
//                        let json_v: Value = serde_json::from_str(string.as_str()).unwrap();
//
//                        let action = match json_v.get("action") {
//                            Some(a) => a.as_str().unwrap(),
//                            None => return,
//                        };
//                        let json_v = json!({
//                            "user_ids": ["id1", "id2"],
//                            "message": {
//                                "user_id": "id1",
//                                "contents": "test message",
//                                "timestamp": 4,
//                                "read": false,
//                            },
//                            "action": "create_thread",
//                        });
//                        let action: String = "create_thread".to_string();

                        let json_v = json!({
                            "user_id": "id1",
                            "start_index": 0,
                            "end_index": 10,
                            "action": "get_user_threads"
                        });
                        let action: String = "get_user_threads".to_string();

                        match take_action(&action, &json_v, &firebase) {
                            Ok(res) =>
                                { let reply = serde_json::to_string(&res).unwrap();
//                                    let reply = json!({
//                                    "action": action.to_string(),
//                                    "data": res.body,
//                                  });
                                  println!("Reply to frontend is {:?}", reply);
                                  //println!("Reply is {:?}", res);
                                  let message = OwnedMessage::Text(reply);
                                  sender.send_message(&message).unwrap();
                                }
                            Err(_)  => return,
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

    fn take_action(action: &str, json_v: &serde_json::Value, firebase: &Firebase) -> Result<Reply, error::ServerError> {

        println!("Action is {}", action);

        if action == "send_message" {
            println!("sending message...");

            let m_string = match json_v.get("message") {
                Some(m) => { m.to_string() },
                None => {
                    println!("None value returned");
                    return Err(error::ServerError::DatabaseFormatErr)
                },
            };

            let new_mes: message::Message = match serde_json::from_str(m_string.as_str()) {
                Ok(d) => { Some(d).unwrap() },
                Err(e) => {
                    eprintln!("error {:?}", e);
                    return Err(error::ServerError::DatabaseFormatErr)
                },
            };

            println!("Message struct is {:?}", new_mes);

            let thread_id = match json_v.get("thread_id") {
                Some(id) => id.as_str().unwrap(),
                None => { println!("Thread_id None value returned");
                          return Err(error::ServerError::DatabaseFormatErr) },
            };

            let res = match message::create_message(thread_id, &new_mes, &firebase) {
                Ok(response) => response,
                Err(err) => { println!("Response None value returned");

                              return Err(err) },
            };

            let code: u32 = match res.code {
                hyper::status::StatusCode::Ok => 200,
                hyper::status::StatusCode::BadRequest => 400,
                _ => 500,
            };

            let reply = Reply {
                action: action.to_string(),
                body: "".to_string(),
                code
            };

            Ok(reply)
        }
        else if action == "create_thread" {
            println!("creating thread...");

            let m_string = match json_v.get("message") {
                Some(m) => { m.to_string() },
                None => {
                    println!("None value returned");
                    return Err(error::ServerError::DatabaseFormatErr)
                },
            };

            let new_mes: message::Message = match serde_json::from_str(m_string.as_str()) {
                Ok(d) => { Some(d).unwrap() },
                Err(e) => {
                    eprintln!("error {:?}", e);
                    return Err(error::ServerError::DatabaseFormatErr)
                },
            };

            let user_ids: Vec<&str> = match json_v.get("user_ids") {
                Some(ids) => ids
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|ref x| x.as_str().unwrap())
                    .collect::<Vec<&str>>(),
                None => { println!("user_ids None value returned");
                          return Err(error::ServerError::DatabaseFormatErr) },
            };

            let create_res = match threads::create_thread(&user_ids, &firebase) {
                Ok(response) => response,
                Err(err) => { println!("create_thread None value returned");
                              return Err(err) },
            };

            let thread = match serde_json::from_str(&create_res.body).unwrap() {
                serde_json::Value::Object(map) => {
                    //println!("Map is {:?}", map);
                    //println!("{:?}", map.get("name").unwrap().to_string());
                    map.get("name").unwrap().to_string()
                },
                _ => {return Err(error::ServerError::ReqNotJSON) }
            };

            let user = new_mes.user_id.clone();
            println!("Thread is {:?}", thread);

            let res = match message::create_message(&thread, &new_mes, &firebase) {
                Ok(response) => response,
                Err(err) => { println!("Response None value returned");
                              return Err(err) },
            };

            for u in &user_ids {
                let r = match users::update_user_threads(&u, &thread, &new_mes, &firebase) {
                    Ok(response) => response,
                    Err(err) => return Err(err),
                };
            }

            let res = match users::update_user_threads(&user, &thread, &new_mes, &firebase) {
                Ok(response) => response,
                Err(err) => return Err(err),
            };

            let code: u32 = match res.code {
                hyper::status::StatusCode::Ok => 200,
                hyper::status::StatusCode::BadRequest => 400,
                _ => 500,
            };

            let reply = Reply {
                action: action.to_string(),
                body: "".to_string(),
                code
            };

            Ok(reply)
        }
        else if action == "get_user_threads" {
            println!("getting user threads...");

            let user_id = match json_v.get("user_id") {
                Some(id) => id.as_str().unwrap(),
                None => return Err(error::ServerError::DatabaseFormatErr),
            };

            let start_index = match json_v.get("start_index") {
                Some(i) => i.as_u64().unwrap() as u32,
                None => return Err(error::ServerError::DatabaseFormatErr),
            };
            println!("Start {}", start_index);

            let end_index = match json_v.get("end_index") {
                Some(i) => i.as_u64().unwrap() as u32,
                None => return Err(error::ServerError::DatabaseFormatErr),
            };
            println!("End {}", end_index);

            let res = match users::get_user_threads(user_id, start_index, end_index, &firebase) {
                Ok(response) => response,
                Err(err) => return Err(err),
            };

            let code: u32 = match res.code {
                hyper::status::StatusCode::Ok => 200,
                hyper::status::StatusCode::BadRequest => 400,
                _ => 500,
            };

            let reply = Reply {
                action: action.to_string(),
                body: res.body,
                code
            };

            Ok(reply)

        } else if action == "get_thread_messages" {
            println!("getting thread messages...");

            let thread_id = match json_v.get("thread_id") {
                Some(id) => id.as_str().unwrap(),
                None => return Err(error::ServerError::DatabaseFormatErr),
            };

            let start_index = match json_v.get("start_index") {
                Some(i) => i.as_u64().unwrap() as u32,
                None => return Err(error::ServerError::DatabaseFormatErr),
            };
            println!("Start {}", start_index);

            let end_index = match json_v.get("end_index") {
                Some(i) => i.as_u64().unwrap() as u32,
                None => return Err(error::ServerError::DatabaseFormatErr),
            };
            println!("End {}", end_index);

            let res = match threads::get_thread_messages(thread_id, start_index, end_index, &firebase) {
                Ok(response) => response,
                Err(err) => return Err(err),
            };

            let code: u32 = match res.code {
                hyper::status::StatusCode::Ok => 200,
                hyper::status::StatusCode::BadRequest => 400,
                _ => 500,
            };

            let reply = Reply {
                action: action.to_string(),
                body: res.body,
                code
            };

            Ok(reply)
        }

        else {
            println!("not matching correctly");
            Err(error::ServerError::BadRequest)
        }
    }
}