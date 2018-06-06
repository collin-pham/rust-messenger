//! User functions to interact with
//! Users table in Firebase. Can fetch user
//! data, including conversations, identification,
//! and can sort conversations returned.
extern crate firebase;
extern crate serde_json;
extern crate hyper;

use self::firebase::{Firebase, Response};
use super::{error, message};

/// Retrieves a user from Firebase, returning a Response whose body
/// includes the user's email, username, and list of conversation/thread IDs.
pub fn get_user(user_id: &str, firebase: &Firebase) -> Result<Response, error::ServerError>{
    let user = match firebase.at(&format!("/users/{}", user_id)) {
        Err(err)            => { return Err(error::handle_parse_error(err)) }
        Ok(user)            => user
    };

    let res = match user.get() {
        Err(err)    => { return Err(error::handle_req_error(err)) }
        Ok(res)     => {
            if res.body == "null" { return Err(error::ServerError::InvalidUserId) }
            res
        }
    };

    Ok(res)
}

/// Retrieves a user's threads from Firebase, returning a Response whose body
/// includes the thread ID which contains the most recent message in the conversation.
pub fn get_user_threads(user_id: &str, start_index: u32, end_index: u32, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let threads = match firebase.at(&format!("/users/{}/threads", user_id)) {
        Err(err)    => { return Err(error::handle_parse_error(err)) }
        Ok(user)    => user
    };

    let range = end_index - start_index;
    let res = match threads.order_by("\"timestamp\"").limit_to_last(range).get() {
        Err(err)    => { return Err(error::handle_req_error(err)) }
        Ok(threads) => {
            if threads.body == "null" {
                return Ok(Response {
                    body: serde_json::Value::Array(vec![]).to_string(),
                    code: hyper::status::StatusCode::Ok,
                })
            } else {
                threads
            }
        }
    };
    sort_user_threads(res.body)
}

/// Updates a user's thread conversation with the contents of a new Message.
/// Should be called for the sender and receiver of new_message.
pub fn update_user_threads(user_id: &str, thread_id: &str, new_message: &message::Message, firebase: &Firebase)
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

/// Sorts the threads temporally, so as to be displayed in a coherent manner
/// from the frontend. Called by `get_user_threads`.
fn sort_user_threads(threads: String) -> Result<Response, error::ServerError> {
    let threads = match serde_json::from_str(&threads).unwrap() {
        serde_json::Value::Object(json) => {
            let mut threads = vec![];
            // Put threads into vec.
            for (key, thread) in json.into_iter() {
                let mut item = serde_json::Map::new();
                item.insert(key, thread);

                threads.push(item);
            }

            // Sort threads by timestamp
            threads.sort_by(|a, b| {
                // We truly apologize for this ugly code.
                b.values().cloned().collect::<Vec<_>>()[0].get("timestamp").unwrap().as_u64().unwrap()
                    .cmp(&a.values().cloned().collect::<Vec<_>>()[0].get("timestamp").unwrap().as_u64().unwrap())
            });

            let mut sorted_threads = vec![];
            for item in threads.into_iter() {
                sorted_threads.push(serde_json::Value::Object(item));
            }
            sorted_threads
        }
        _ => { return Err(error::ServerError::ReqNotJSON) }
    };

    let res = Response {
        body: serde_json::Value::Array(threads).to_string(),
        code: hyper::status::StatusCode::Ok,
    };

    Ok(res)
}

#[cfg(test)]
mod users_tests {
    use super::*;
    use super::super::{db, message::Message};

    #[test]
    fn get_user_test() {
        let firebase = db::connect();
        let res = get_user("test_user_id", &firebase);
        assert_eq!
        ( res.ok().unwrap().body,
          "{\"email\":\"test@test.com\",\
            \"threads\":\
               {\"test_thread_id\":\
                  {\"contents\":\"Look! I sent you a message.\",\
                  \"read\":false,\
                  \"timestamp\":10,\
                  \"user_id\":\"test_user_id\"}},\
            \"username\":\"Test Name\"}"
        )
    }

    #[test]
    fn get_user_threads_test() {
        let firebase = db::connect();
        let res = get_user_threads("test_user_id", 0, 1, &firebase);
        assert_eq!
        ( res.ok().unwrap().body,
          "[{\"test_thread_id\":\
                {\"contents\":\"Look! I sent you a message.\",\
                 \"read\":false,\
                 \"timestamp\":10,\
                 \"user_id\":\"test_user_id\"}}]"
        )
    }

    #[test]
    fn update_user_threads_test() {
        let firebase = db::connect();

        let new_message = Message {
            user_id: "test_user_id_2".to_owned(),
            timestamp: 100,
            contents: "This Is A Test Message".to_owned(),
            read: false,
        };

        let res = update_user_threads("test_user_id_2", "test_thread_id_2", &new_message, &firebase);
        assert_eq!
        ( res.ok().unwrap().body,
          "{\"user_id\":\"test_user_id_2\",\"timestamp\":100,\"contents\":\"This Is A Test Message\",\"read\":false}"
        )
    }

    #[test]
    fn sort_user_threads_test() {
        let res = sort_user_threads(
            "{\"test_thread_id\":\
                {\"contents\":\"well hello there\",\
                \"read\":true,\
                \"timestamp\":5,\
                \"user_id\":\"test_user_id_2\"},\
              \"test_thread_id_2\":\
                {\"contents\":\"This Is A Test Message\",\
                \"read\":false,\
                \"timestamp\":100,\
                \"user_id\":\"test_user_id_2\"}}".to_string()
        );


        let sorted =  "[{\"test_thread_id_2\":\
                            {\"contents\":\"This Is A Test Message\",\
                            \"read\":false,\
                            \"timestamp\":100,\
                            \"user_id\":\"test_user_id_2\"}},\
                        {\"test_thread_id\":\
                            {\"contents\":\"well hello there\",\
                            \"read\":true,\
                            \"timestamp\":5,\
                            \"user_id\":\"test_user_id_2\"}}]".to_string();

       assert_eq!( res.ok().unwrap().body, sorted )
    }
}