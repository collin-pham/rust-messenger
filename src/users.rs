extern crate firebase;
extern crate serde_json;
extern crate hyper;

use self::firebase::{Firebase, Response};
use super::{error, message};

pub fn get_user(user_id: &str, firebase: &Firebase) -> Result<Response, error::ServerError>{
    let user = match firebase.at(&format!("/users/{}", user_id)) {
        Err(err)            => { return Err(error::handle_parse_error(err)) }
        Ok(user)            => user
    };

    let res = match user.get() {
        Err(err)    => { return Err(error::handle_req_error(err)) }
        Ok(res)     => {
            if res.body == "null" {
                return Err(error::ServerError::InvalidUserId)
            }
            res
        }
    };

    Ok(res)
}

pub fn get_user_threads(user_id: &str, start_index: u32, end_index: u32, firebase: &Firebase)
                        -> Result<Response, error::ServerError>
{
    let threads = match firebase.at(&format!("/users/{}/threads", user_id)) {
        Err(err)    => { return Err(error::handle_parse_error(err)) }
        Ok(user)    => user
    };

    let range = end_index - start_index;
    let res = match threads.order_by("\"timestamp\"").start_at(start_index).limit_to_first(range).get() {
        Err(err)    => { return Err(error::handle_req_error(err)) }
        Ok(threads) => threads
    };

    sort_user_threads(res.body)
}

pub fn update_user_threads(user_id: &str, thread_id: &str, new_message: message::Message, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let thread = match firebase.at(&format!("/users/{}/threads/{}", user_id, thread_id)) {
        Err(err)    => { return Err(error::handle_parse_error(err)) }
        Ok(user)    => user
    };
    let msg = message::new_message_to_user_json(new_message);
    let res = match thread.update(&msg) {
        Err(err)    => { return Err(error::handle_req_error(err)) }
        Ok(thread)  => { thread }
    };

    Ok(res)
}

fn sort_user_threads(threads: String) -> Result<Response, error::ServerError> {

    let threads = match serde_json::from_str(&threads).unwrap() {
        serde_json::Value::Object(map) => {
            let mut threads: Vec<_> = map.values().cloned().collect();
            threads.sort_by(|a, b| {
                b.get("timestamp").unwrap().as_u64().unwrap().cmp(&a.get("timestamp").unwrap().as_u64().unwrap())
            });

            threads
        }
        _ => { return Err(error::ServerError::ReqNotJSON) }
    };

    let res = Response {
        body: serde_json::Value::Array(threads).to_string(),
        code: hyper::status::StatusCode::Ok,
    };

    Ok(res)
}