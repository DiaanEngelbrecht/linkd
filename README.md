# Linkd

A simple lightweight actor model framework that takes inspiration from OTP in the erlang ecosystem.

Linkd focuses more on high level features. It assumes you call it inside an async tokio runtime.

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

type HealthActor = Actor::<MyState, Messages, Responses>;

struct MyState;

enum Messages {
    Ping,
}

enum Responses {
    Pong,
}

#[async_trait]
impl Handler<MyState, Responses> for Messages {
    async fn handle(&self, _state: &mut MyState) -> Responses {
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

    // Create a registry(optional)
    let mut registry = Registry::new();

    // Initialize the actor and immediately start it up
    let mut my_actor = HealthActor::new();
    my_actor.startup(MyState {}).await;
    // Move the actor to the registry
    my_actor.register(&mut registry);
  

    // Go fetch the actor out of the registry from anywhere.
    let local_ref = HealthActor::fetch_from_registry(&registry);

    // Call out to the actor and wait for a response.
    let response = local_ref.call(Messages::Ping).await;
    
    if response == Responses::Pong {
        println!("Would you look at that!");
    }

    // Cast a message to the actor. This is akin to a fire and forget.
    local_ref.cast(Messages::V4).await;
}
```
