# rust-messenger
A backend messaging server built in Rust.

## What we plan to build

We plan to build a backend messaging server built in Rust. This server should expose an API that is able to interact with both mobile and web clients such that it could power mobile or web based messaging applications. 

We will also implement a lightweight frontend web application to interact with our API to prove the system works.


## Why it is interesting 

* After implementing an HTTP web server, we think it would be interesting to extend this further to implement the back-end of a chat-based messenger.

* Being able to efficiently design the back-end will be a worthwhile use of having learned Rust.

* Having an API for users to see the real-time messaging with another user would be a useful application and rewarding to see implemented.


## Potential anticipiated difficulties

We see a few potential difficulties:

* Determining what protocol to use for client-server communication. We need to do more research into what the industry is doing, and decide from there.

* Designing backend to be able to handle an extremely high number of concurrent requests.

* Designing messaging logic and DB schema, i.e. how we will think about data, users, messages, etc.


## Concrete Functional Requirements

* Guaranteed messaging functionality (i.e our server will always deliver messages)

* Well defined and documented API

* Simple frontend library


## High level architecture:

* Spawn new thread per message (to a limit?)

* If reciever is not connected to server, store message in DB and change reciever state to indicate they have unread messages.

* Once reciever connects to server, check state, send messages if necessary. 


## Use Cases / Examples

* Message service to send messages between two users.

* One person sends a message to another person. The server receives the message, attempting to send it to the recipient. If the recipient is not online, the server/threads need to handle how to store and eventually send the message to the recipient.

* To the user, they will send a message, and the recipient will receive it, and vice versa.