extern crate firebase;
extern crate hyper;
extern crate hyper_openssl;
extern crate rustc_serialize;
extern crate url;

use std::io;
use std::str;
use self::firebase::{Firebase, Response, ReqErr, ParseError,};
// User Table Functions

pub fn connect() -> Firebase {
    let firebase = match Firebase::new("https://courier-13efc.firebaseio.com") {
        Ok(connection)  => { connection }
        Err(_)          =>  { panic!("Could Not Establish Firebase Connection") }
    };

    firebase
}

pub fn get_user(user_id: &str, firebase: &Firebase) -> Result<Response, ServerError>{
    let user = match firebase.at(&format!("/users/{}", user_id)) {
        Err(err)            => { return Err(handleParseError(err)) }
        Ok(user)            => user
    };

    let res = match user.get() {
        Err(err)    => { return Err(handleReqErr(err)) }
        Ok(res)     => {
            if res.body == "null" {
                return Err(ServerError::InvalidUserId)
            }
            res
        }
    };

    Ok(res)
}

pub fn get_user_threads(user_id: &str, start_index: u32, end_index: u32, firebase: &Firebase)
    -> Result<Response, ServerError>
{
    let threads = match firebase.at(&format!("/users/{}/threads", user_id)) {
        Err(err)    => { return Err(handleParseError(err)) }
        Ok(user)    => user
    };

    let range = end_index - start_index;
    let res = match threads.order_by("\"timestamp\"").start_at(start_index).limit_to_first(range).get() {
        Err(err)    => { return Err(handleReqErr(err)) }
        Ok(threads) => threads
    };
    Ok(res)
}

pub fn update_user_threads(user_id: &str, ) {
    unimplemented!()
}

pub enum ServerError {
    ReqNotJSON,
    RespNotUTF8(str::Utf8Error),
    NetworkErr(hyper::error::Error),
    SslErr(hyper_openssl::openssl::error::ErrorStack),
    FirebaseIoErr(String),
    FirebaseIoJsonParseErr(rustc_serialize::json::DecoderError),
    OtherErr(io::Error),

    UrlHasNoPath,
    UrlIsNotHTTPS,
    Parser(url::ParseError),

    InvalidUserId,
}

fn handleParseError(err: ParseError) -> ServerError {
    match err {
        ParseError::UrlHasNoPath    => ServerError::UrlHasNoPath,
        ParseError::UrlIsNotHTTPS   => ServerError::UrlIsNotHTTPS,
        ParseError::Parser(err)     => ServerError::Parser(err),
    }
}

fn handleReqErr(err: ReqErr) -> ServerError {
    match err {
        ReqErr::ReqNotJSON                  => ServerError::ReqNotJSON,
        ReqErr::RespNotUTF8(err)            => ServerError::RespNotUTF8(err),
        ReqErr::NetworkErr(err)             => ServerError::NetworkErr(err),
        ReqErr::SslErr(err)                 => ServerError::SslErr(err),
        ReqErr::FirebaseIoErr(err)          => ServerError::FirebaseIoErr(err),
        ReqErr::FirebaseIoJsonParseErr(err) => ServerError::FirebaseIoJsonParseErr(err),
        ReqErr::OtherErr(err)               => ServerError::OtherErr(err),
    }
}