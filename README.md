# Linkd

A simple actor model framework that takes inspiration from OTP in the erlang ecosystem.

Linkd is a high level abstraction that focuses more on high level features. It assumes you call it inside an async tokio runtime.

## Getting started

Your actor needs to define three parts:
- The type and initial value of internal state.
- The type of all incoming messages.
- The type of all possible responses.

You then implement the handler for your message type and you're done.

Below is a simple actor that can get pinged, and will respond with pong.

```rust
use async_trait::async_trait;
use linkd::{Actor, Handler};

struct MyState;

enum Messages {
    Ping,
}

#[derive(PartialEq)]
enum Responses {
    Pong,
}

#[async_trait]
impl Handler<MyState, Responses> for Messages {
    async fn handle(&self, _state: &MyState) -> Responses {
        match self {
            Messages::Ping => {
                println!("Hello darkness my old friend.");

                Responses::Pong
            }
        }
    }
}


#[tokio::main]
async fn main() {
    // Initialize the actor and immediately start it up
    let mut my_actor = Actor::new();
    my_actor.startup(MyState {}).await;

    // Call out to the actor and wait for a response.
    let response = my_actor.call(Messages::V4).await;
    
    if response == Responses::Pong {
        println!("Would you look at that!");
    }

    // Cast a message to the actor. This is fire and forget.
    my_actor.cast(Messages::V4).await;
}
```
