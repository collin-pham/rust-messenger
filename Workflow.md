# Persistent data store

rust-messenger will use a No-SQL DB (likely firebase) to handle and store messaging. 


## High level Workflow:

* User A sign in
* Connection established between User A's client and rust-messenger server.
* User A messages User B
* Client sends request to /message endpoint
* Server receives request
* Server parses request
  * If request body okay, return success (client shows message is sent)
    * For now, okay = valid thread_id, contents
  * Otherwise, return error (client shows message not sent)
* Server pushes new message data to correct thread_id
* Server updates both user's [thread_id] arrays
  * If User B is connected to server, write to their stream with message data. 
    *   Client updates front end accordingly
  * Otherwise, User B will see unread message next time they sign in. 


## Schema 

Below is how we will store message data in our DB. Each user will have a unique user_id. Associated with this user_id will be a username and an array of thread_id's (conversations). Conversations will be sorted by timestamp so that we can display conversations temporally. 

Each conversation will have a unique thread_id. Associated with this thread_id will be an array of user_id's (the users involved in the conversation) and an array of message_id's (the actual messages). 

Each message will have a user_id (the sender of the message), contents, and a timestamp.


### Users Table

```
[user_id]
  username
  [thread_id] (ordered by timestamp)
    timestamp
    last_msg 
    read
```
### Threads Table

```
[thread_id]
  [user_id] (this is an array so we can eventually support group messaging)
  [message_id] (ordered by timestamp)
    user_id 
    contents
    timestamp
```