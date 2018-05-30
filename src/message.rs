extern crate firebase;

use self::firebase::{Firebase, Response};
use super::{error, message};

pub struct Message {
    pub user_id:    String,
    pub timestamp:  usize,
    pub contents:   String,
    pub read:       bool,
}

pub fn create_message(thread_id: &str, new_message: Message, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let messages = match firebase.at(&format!("/threads/{}/message_ids", thread_id)) {
        Err(err)            => { return Err(error::handleParseError(err)) }
        Ok(user)            => user
    };

    let res = match messages.push(&new_message_to_thread_JSON(new_message)) {
        Err(err)    => { return Err(error::handleReqErr(err)) }
        Ok(thread)  => { thread }
    };

    Ok(res)
}

pub fn new_message_to_user_JSON(new_message: Message) -> String {
    format!("{{\"user_id\":\"{}\", \"timestamp\":{}, \"contents\":\"{}\", \"read\":{}}}",
            new_message.user_id,
            new_message.timestamp,
            new_message.contents,
            new_message.read,
    )
}

pub fn new_message_to_thread_JSON(new_message: Message) -> String {
    format!("{{\"user_id\":\"{}\", \"timestamp\":{}, \"contents\":\"{}\"}}",
            new_message.user_id,
            new_message.timestamp,
            new_message.contents,
    )
}