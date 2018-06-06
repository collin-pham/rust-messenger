//! Module to create a connection between the
//! rust-messenger crate and Firebase
extern crate firebase;

use self::firebase::Firebase;
// User Table Functions

/// Initiates a connection to the Courier Firebase DB.
pub fn connect() -> Firebase {
    let firebase = match Firebase::new("https://courier-13efc.firebaseio.com") {
        Ok(connection)  =>  { connection }
        Err(_)          =>  { panic!("Could Not Establish Firebase Connection") }
    };

    firebase
}
