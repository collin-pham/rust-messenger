//! Module with functions to create messages
//! holding necessary information for Firebase.
//! Can convert from `Message` struct to JSON and
//! vice versa.
extern crate firebase;

use self::firebase::{Firebase, Response};
use super::{error};

extern crate serde;
extern crate serde_json;


#[derive(Serialize, Deserialize, Debug)]
/// Message structure holding the user who sent the message, when the
/// message was sent, the text inside, and a read receipt. Derives
/// Serializability in order to be transformed into JSON data.
pub struct Message {
    pub user_id:    String,
    pub timestamp:  usize,
    pub contents:   String,
    pub read:       bool,
}

/// Sends a new Message to the specified conversation of `thread_id`, assuming that
/// the user already has a conversation started.
pub fn create_message(thread_id: &str, new_message: &Message, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let messages = match firebase.at(&format!("/threads/{}/message_ids", thread_id)) {
        Err(err)            => { //println!("Create_message error matching: {:?}", err);
                                 return Err(error::handle_parse_error(err)) }
        Ok(user)            => user
    };

    let res = match messages.push(&new_message_to_thread_json(new_message)) {
        Err(err)    => { //println!("Create_message error pushing: {:?}", err);
                         return Err(error::handle_req_error(err)) }
        Ok(thread)  => { thread }
    };

    Ok(res)
}

/// Converts Message struct into a JSON string for the user table.
pub fn new_message_to_user_json(new_message: &Message) -> String {
    format!("{{\"user_id\":\"{}\", \"timestamp\":{}, \"contents\":\"{}\", \"read\":{}}}",
            new_message.user_id,
            new_message.timestamp,
            new_message.contents,
            new_message.read,
    )
}

/// Converts Message struct into a JSON string for the thread table.
pub fn new_message_to_thread_json(new_message: &Message) -> String {
    format!("{{\"user_id\":\"{}\", \"timestamp\":{}, \"contents\":\"{}\"}}",
            new_message.user_id,
            new_message.timestamp,
            new_message.contents,
    )
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn message_user_json_test() {
        let m = Message {
            user_id: "a".to_string(),
            timestamp: 20,
            contents: "this is a test".to_string(),
            read: false,
        };

        let res = new_message_to_user_json(&m);

        assert_eq!(
            res,
            "{\"user_id\":\"a\", \"timestamp\":20, \"contents\":\"this is a test\", \"read\":false}"
        )
    }

    #[test]
    fn message_thread_json_test() {
        let m = Message {
            user_id: "a".to_string(),
            timestamp: 20,
            contents: "this is a test".to_string(),
            read: false,
        };

        let res = new_message_to_thread_json(&m);

        assert_eq!(
            res,
            "{\"user_id\":\"a\", \"timestamp\":20, \"contents\":\"this is a test\"}"
        )
    }
}

