/*!
 * Rust Messenger
 *
 * This crate provides a backend messaging server built in Rust.
 * It interacts with Firebase to handle and store the database,
 * and a simple frontend library, "Courier," to display the
 * chat messaging service interface. The backend API implements
 * concurrent connections for each client connected to the
 * server, which uses websocket connections.
 *
 *
 * Functionality
 *
 * - Message service to send texts between two users.
 * - If a user sends a message to a receiver, and the receiver
 *     is not online, then there message will be waiting for
 *     them once they re-login.
 * - Multithreading to handle multiple users connected to
 *     the Rust Messenger service.
 *
 *
 * Schema
 *
 * Below is how we will store message data in our DB. Each user will have a unique user_id.
 * Associated with this user_id will be a username and an array of thread_id's (conversations).
 * Conversations will be sorted by timestamp so that we can display conversations temporally.
 * Each conversation will have a unique thread_id. Associated with this thread_id will be an array of
 * user_id's (the users involved in the conversation) and an array of message_id's (the actual messages).
 * Each message will have a user_id (the sender of the message), contents, and a timestamp.
 *
 *
 * ```
 * struct send_message {
 *    thread_id:     String,
 *    message: {
 *        user_id:   String,
 *        contents:  String,
 *        timestamp: u32,
 *    },
 *    action:        String //"send_message"
 * }
 * ```
 *
 * ```
 * create_thread ->
 *   {
 *       user_ids:      [String],
 *       message: {
 *           user_id:   String,
 *           contents:  String,
 *           timestamp: u32,
 *       },
 *       action:        String //"create_thread"
 *   }
 * ```
 *
 *
 *
 *
 *   get_user_threads ->
 *   {
 *       user_id:        `String`,
 *       start_index:    `u32`,
 *       end_index:      `u32`,
 *       action:         'get_user_threads'
 *   }
 *
 *
 *   get_thread_messages ->
 *   {
 *       thread_id:      `String`,
 *       start_index:    `u32`,
 *       end_index:      `u32`,
 *       action:         'get_thread_messages'
 *   }
 *
 */

pub mod db;
pub mod error;
pub mod message;
pub mod protocol;
pub mod threads;
pub mod users;


#[macro_use]
extern crate serde_derive;

