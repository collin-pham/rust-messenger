extern crate firebase;

use self::firebase::{Firebase, Response};
use super::{error, message};

pub fn get_thread_user_ids(thread_id: &str, firebase: &Firebase) -> Result<Response, error::ServerError> {
    let thread = match firebase.at(&format!("/threads/{}/user_ids", thread_id)) {
        Err(err)            => { return Err(error::handleParseError(err)) }
        Ok(user)            => user
    };

    let res = match thread.get() {
        Err(err)    => {
            println!("{:?}", err);
            return Err(error::handleReqErr(err))
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
        Err(err)            => { return Err(error::handleParseError(err)) }
        Ok(user)            => user
    };

    let range = end_index - start_index;
    let res = match thread.order_by("\"timestamp\"").limit_to_first(3).get() {
        Err(err)    => {
            println!("{:?}", err);
            return Err(error::handleReqErr(err))
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

pub fn create_thread(user_ids: Vec<&str>, firebase: &Firebase)
    -> Result<Response, error::ServerError>
{
    let thread = match firebase.at("/threads") {
        Err(err)            => { return Err(error::handleParseError(err)) }
        Ok(user)            => user
    };

    let res = match thread.push(&build_thread_JSON(user_ids)) {
        Err(err)    => { return Err(error::handleReqErr(err)) }
        Ok(thread)  => { thread }
    };
    Ok(res)
}

fn user_ids_to_str (user_ids: Vec<&str>) -> String {
    format!("{:?}", user_ids)
}

fn build_thread_JSON(user_ids: Vec<&str>) -> String{
    format!("{{\"user_ids\": {}}}", user_ids_to_str(user_ids))
}


#[cfg(test)]
mod thread_tests {
    use super::{get_thread};
    use super::super::{db, users};


    #[test]
    fn get_thread_test() {
        println!("Hidden output");
        let firebase = db::connect();

        let res = get_thread("test_thread_id", &firebase);

        assert_eq!(
            res.ok().unwrap().body,
            "{\"message_id\":[{\
            \"contents\":\"fake_data\",\
            \"timestamp\":0,\
            \"user_id\":0\
            }],\
            \"user_id\":[\"fake_user_0\",\"fake_user_1\"]\
            }"
        )
    }
}