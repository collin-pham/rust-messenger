extern crate firebase;

use self::firebase::{Firebase};
// User Table Functions

pub fn connect() -> Firebase {
    let firebase = match Firebase::new("https://courier-13efc.firebaseio.com") {
        Ok(connection)  => { connection }
        Err(_)          =>  { panic!("Could Not Establish Firebase Connection") }
    };

    firebase
}
