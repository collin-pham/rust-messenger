extern crate firebase;
extern crate serde_json;
extern crate hyper;

use self::firebase::{Firebase, Response};
use super::{error};

pub fn get_thread_user_ids(thread_id: &str, firebase: &Firebase) -> Result<Response, error::ServerError> {
    let thread = match firebase.at(&format!("/threads/{}/user_ids", thread_id)) {
        Err(err)            => { return Err(error::handle_parse_error(err)) }
        Ok(user)            => user
    };

    let res = match thread.get() {
        Err(err)    => {
            println!("{:?}", err);
            return Err(error::handle_req_error(err))
        }
        Ok(res)     => {
            if res.body == "null" {
                return Err(error::ServerError::InvalidThreadId)
            }
            res
        }
    };

    Ok(res)
}

pub fn get_thread_messages(thread_id: &str, start_index: u32, end_index: u32, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let thread = match firebase.at(&format!("/threads/{}/message_ids", thread_id)) {
        Err(err)            => { return Err(error::handle_parse_error(err)) }
        Ok(user)            => user
    };

    let range = end_index - start_index;
    let res = match thread.order_by("\"timestamp\"").start_at(start_index).limit_to_first(range).get() {
        Err(err)    => {
            println!("{:?}", err);
            return Err(error::handle_req_error(err))
        }
        Ok(res)     => {
            if res.body == "null" {
                return Err(error::ServerError::InvalidThreadId)
            }
            res
        }
    };

    sort_thread_messages(res.body)
}

pub fn create_thread(user_ids: Vec<&str>, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let thread = match firebase.at("/threads") {
        Err(err)            => { return Err(error::handle_parse_error(err)) }
        Ok(user)            => user
    };

    let res = match thread.push(&build_thread_json(user_ids)) {
        Err(err)    => { return Err(error::handle_req_error(err)) }
        Ok(thread)  => { thread }
    };
    Ok(res)
}

fn sort_thread_messages(messages: String) -> Result<Response, error::ServerError> {
    let messages = match serde_json::from_str(&messages).unwrap() {
        serde_json::Value::Object(map) => {
            let mut messages: Vec<_> = map.values().cloned().collect();
            messages.sort_by(|a, b| {
                b.get("timestamp").unwrap().as_u64().unwrap().cmp(&a.get("timestamp").unwrap().as_u64().unwrap())
            });

            messages
        }
        _ => {return Err(error::ServerError::ReqNotJSON)}
    };

    let res = Response {
        body: serde_json::Value::Array(messages).to_string(),
        code: hyper::status::StatusCode::Ok,
    };

    Ok(res)
}
fn user_ids_to_str (user_ids: Vec<&str>) -> String {
    format!("{:?}", user_ids)
}

fn build_thread_json(user_ids: Vec<&str>) -> String{
    format!("{{\"user_ids\": {}}}", user_ids_to_str(user_ids))
}


#[cfg(test)]
mod thread_tests {
    use super::{get_thread_messages};
    use super::super::{db, users};

    #[test]
    fn get_thread_test() {
        let firebase = db::connect();

        let res = get_thread_messages("test_thread_id", 0, 2, &firebase);

        assert_eq!(
            res.ok().unwrap().body,
            "[\
            {\"contents\":\"fake data\",\"timestamp\":128,\"user_id\":1},\
            {\"contents\":\"fake data\",\"timestamp\":10,\"user_id\":0}\
            ]"
        )
    }
}