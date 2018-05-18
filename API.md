# rust-messenger API


We still need to determine the protocol to interact with these endpoints, but this is their functionality. 


## Endpoints

```/message``` -> post a message to the specified thread_id.

Potential Request Body
```
{
  sender_id: String,
    thread_id: String,
    contents : String,
}
```



```/threads``` -> return threads **n** through **m** from a user's thread array.

Potential Request Body
```
{
  user_id    : String,
    start_index: Int,
    end_index  : Int,
}
```


```/thread``` -> return messages **n** through **m** from a thread's message array

Potential Request Body
```
{
  thread_id: String,
    start_index: Int,
    end_index  : Int,
}
```
