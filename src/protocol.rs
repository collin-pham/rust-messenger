//! Interprets requests from frontend
//! and performs corresponding action
//! according to the action protocol.
extern crate websocket;
extern crate serde;
extern crate firebase;
extern crate hyper;
extern crate serde_json;

use self::firebase::Firebase;
use super::{error, message, threads, users};
use self::websocket::OwnedMessage;
use self::websocket::sender::Writer;
use std::str;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
/// Struct which responds to the frontend with a given
/// action that was requested, a body which is empty if the
/// client does not need further information, otherwise it
/// contains message contents, and a code indicating a
/// success or an error. Derives Serializability in order
/// to be transformed into JSON data.
pub struct Reply {
    pub action: String,
    pub body:   String,
    pub code:   u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub body:   serde_json::Value, //e.g. for send_message, thread_id and message {}
    pub action: String,
}

/// Matches an action and calls corresponding function to query
/// Firebase. For send_message, will ping connected_users who
/// are involved in the send_message to perform a live update.
pub fn take_action(
    request: &Request,
    firebase: &Firebase,
    id: &str,
    connected_users: &Arc<Mutex<HashMap<String, Writer<TcpStream>>>>)
    -> Result<Reply, error::ServerError> {

    println!("Action is {}", request.action);

    if request.action == "send_message" {
        println!("sending message...");
        return action_send_message(&request, firebase, id, connected_users)

    } else if request.action == "create_thread" {
        println!("creating thread...");
        return action_create_thread(&request, firebase)

    } else if request.action == "get_user_threads" {
        println!("getting user threads...");
        return action_get_user_threads(&request, firebase)

    } else if request.action == "get_thread_messages" {
        println!("getting thread messages...");
        return action_get_thread_messages(&request, firebase)

    } else {
        println!("not matching correctly");
        Err(error::ServerError::BadRequest)
    }
}

/// Creates a new_message inside of thread_id, updating all
/// involved users and pinging live updates to connected users
/// to the server. Returns a `Reply` or `error::ServerError`.
pub fn action_send_message(
    request: &Request,
    firebase: &Firebase,
    id: &str,
    connected_users: &Arc<Mutex<HashMap<String, Writer<TcpStream>>>>
) -> Result<Reply, error::ServerError> {

    let m_string = match request.body.get("message") {
        Some(m) => { m.to_string() },
        None => { return Err(error::ServerError::DatabaseFormatErr) },
    };

    let new_mes: message::Message = match serde_json::from_str(m_string.as_str()) {
        Ok(d) => { Some(d).unwrap() },
        Err(e) => {
            eprintln ! ("error {:?}", e);
            return Err(error::ServerError::DatabaseFormatErr)
        },
    };

    println ! ("Message struct is {:?}", new_mes);

    let thread_id = match request.body.get("thread_id") {
        Some(id) => id.as_str().unwrap(),
        None => {
            println ! ("Thread_id None value returned");
            return Err(error::ServerError::DatabaseFormatErr) },
    };

    let res = match message::create_message(thread_id, &new_mes, &firebase) {
        Ok(response) => response,
        Err(err) => {
            println ! ("Response None value returned");
            return Err(err) },
    };

    let user_ids = match threads::get_thread_user_ids(thread_id, firebase) {
        Ok(response) => {
            match serde_json::from_str(&response.body).unwrap() {
                serde_json::Value::Array(v) => v,
                _ => return  Err(error::ServerError::DatabaseFormatErr),
            }
        },
        Err(err) => {
            println ! ("Response None value returned");
            return Err(err) },
    };

    for u in user_ids.into_iter() {
        match users::update_user_threads(u.as_str().unwrap(), thread_id, &new_mes, &firebase) {
            Ok(response) => response,
            Err(err) => return Err(err),
        };
        if u != id {
            // TODO: Ensure Thread doesn't panic here
            match connected_users.lock().unwrap().get_mut(u.as_str().unwrap()) {
                Some(receiver) => {
                    let reply = Reply {
                        action  : "receive_message".to_owned(),
                        body    : format!("{{\"thread_id\":\"{}\", \"message\": {}}}", thread_id, m_string),
                        code    : 200,
                    };

                    let message = OwnedMessage::Text(serde_json::to_string(&reply).unwrap());
                    match receiver.send_message(&message) {
                        Ok(_)    => { },
                        Err(err) => return Err(error::ServerError::SendMessageErr(err)),
                    };
                }
                None => {println!("User not connected!")}
            }
        }
    }


    let code: u32 = match res.code {
        hyper::status::StatusCode::Ok => 200,
        hyper::status::StatusCode::BadRequest => 400,
        _ => 500,
    };

    let reply = Reply {
        action: request.action.to_string(),
        body: "".to_string(),
        code
    };

    Ok(reply)
}


/// Creates a new conversation thread if one doesn't exist. Updates all users
/// involved. Returns a `Reply` or `error::ServerError`.
pub fn action_create_thread(request: &Request, firebase: &Firebase) -> Result<Reply, error::ServerError> {
    let m_string = match request.body.get("message") {
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

    let user_ids: Vec<&str> = match request.body.get("user_ids") {
        Some(ids) => ids
            .as_array()
            .unwrap()
            .iter()
            .map(|ref x| x.as_str().unwrap())
            .collect::<Vec<&str>>(),
        None => {
            println!("user_ids None value returned");
            return Err(error::ServerError::DatabaseFormatErr) },
    };

    let create_res = match threads::create_thread(&user_ids, &firebase) {
        Ok(response) => response,
        Err(err) => {
            println!("create_thread None value returned");
            return Err(err) },
    };

    let thread = match serde_json::from_str(&create_res.body).unwrap() {
        serde_json::Value::Object(map) => {
            println!("{:?}", map);
            map.get("name").unwrap().to_string()
        },
        _ => {return Err(error::ServerError::ReqNotJSON) }
    };

    // Remove extra quotes from thread_id
    let thread = str::replace(&thread, "\"", "");

    let user = new_mes.user_id.clone();
    println!("Thread is {:?}", thread);

    match message::create_message(&thread, &new_mes, &firebase) {
        Ok(response) => response,
        Err(err) => {
            println!("Response None value returned");
            return Err(err) },
    };

    for u in &user_ids {
        match users::update_user_threads(&u, &thread, &new_mes, &firebase) {
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
        action: request.action.to_string(),
        body: format!("{{\"thread_id\":\"{}\"}}", thread),
        code
    };

    Ok(reply)
}


/// Queries a range of a user's given conversation threads and returns them.
/// Returns a `Reply` or `error::ServerError`.
pub fn action_get_user_threads(request: &Request, firebase: &Firebase) -> Result<Reply, error::ServerError> {
    let user_id = match request.body.get("user_id") {
        Some(id) => id.as_str().unwrap(),
        None => {
            println!("User ID error");
            return Err(error::ServerError::DatabaseFormatErr)
        }
    };

    let start_index = match request.body.get("start_index") {
        Some(i) => i.as_u64().unwrap() as u32,
        None => {
            println!("End index error");
            return Err(error::ServerError::DatabaseFormatErr)
        }
    };


    let end_index = match request.body.get("end_index") {
        Some(i) => i.as_u64().unwrap() as u32,
        None => {
            println!("End index error");
            return Err(error::ServerError::DatabaseFormatErr)
        }
    };


    let res = match users::get_user_threads(user_id, start_index, end_index, &firebase) {
        Ok(response) => response,
        Err(err) => {
            println!("Error get_user_threads {:?}", err);
            return Err(err)
        }
    };

    let code: u32 = match res.code {
        hyper::status::StatusCode::Ok => 200,
        hyper::status::StatusCode::BadRequest => 400,
        _ => 500,
    };

    let reply = Reply {
        action: request.action.to_string(),
        body: res.body,
        code
    };

    Ok(reply)
}


/// Queries a range of messages for a given thread conversation.
/// Returns a `Reply` or `error::ServerError`.
pub fn action_get_thread_messages(request: &Request, firebase: &Firebase) -> Result<Reply, error::ServerError> {
    let thread_id = match request.body.get("thread_id") {
        Some(id) => id.as_str().unwrap(),
        None => {
            println!("Thread ID error");
            return Err(error::ServerError::DatabaseFormatErr)
        }
    };

    let start_index = match request.body.get("start_index") {
        Some(i) => i.as_u64().unwrap() as u32,
        None => {
            println!("End index error");
            return Err(error::ServerError::DatabaseFormatErr)
        }
    };

    let end_index = match request.body.get("end_index") {
        Some(i) => i.as_u64().unwrap() as u32,
        None => {
            println!("End index error");
            return Err(error::ServerError::DatabaseFormatErr)
        }
    };

    let res = match threads::get_thread_messages(thread_id, start_index, end_index, &firebase) {
        Ok(response) => response,
        Err(err) => {
            println!("Error get_user_threads {:?}", err);
            match err {
                error::ServerError::InvalidThreadId => {
                    return Ok(Reply{
                        action: request.action.to_string(),
                        body: "Invalid Thread Id".to_string(),
                        code: 404
                    })
                }
                _ => return Err(err)
            }
        }
    };

    let code: u32 = match res.code {
        hyper::status::StatusCode::Ok => 200,
        hyper::status::StatusCode::BadRequest => 400,
        _ => 500,
    };

    let reply = Reply {
        action: request.action.to_string(),
        body: res.body,
        code
    };

    Ok(reply)
}