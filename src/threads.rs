extern crate firebase;

use self::firebase::{Firebase, Response};
use super::{error};

pub fn get_thread(thread_id: &str, firebase: &Firebase) -> Result<Response, error::ServerError>{
    let thread = match firebase.at(&format!("/threads/{}", thread_id)) {
        Err(err)            => { return Err(error::handleParseError(err)) }
        Ok(user)            => user
    };

    let res = match thread.get() {
        Err(err)    => { return Err(error::handleReqErr(err)) }
        Ok(res)     => {
            if res.body == "null" {
                return Err(error::ServerError::InvalidThreadId)
            }
            res
        }
    };

    Ok(res)
}

//pub fn push_new_message() {
//    unimplemented!();
//}

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