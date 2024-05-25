pub mod registry;

use std::{any::TypeId, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use registry::Registry;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

// #[async_trait]
// trait ActorBehavior<State, Messages, Responses> {
//     fn new() -> Self;
//     async fn startup(&mut self, state: State);
//     async fn call(&mut self, message: Messages) -> Result<Responses, oneshot::error::RecvError>;
//     async fn cast(&mut self, message: Messages);
//     async fn fetch_from_registry(registry: Registry) -> &Self;
// }

pub struct Actor<State, Messages, Responses> {
    incoming_tx: Option<Sender<(Messages, Option<oneshot::Sender<Responses>>)>>,
    incoming_rx: Option<Receiver<(Messages, Option<oneshot::Sender<Responses>>)>>,
    context: PhantomData<State>,
}

impl<
        State: Send + 'static,
        Messages: Send + 'static + Handler<State, Responses>,
        Responses: Send + 'static,
    > Actor<State, Messages, Responses>
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<(Messages, Option<oneshot::Sender<Responses>>)>(100);
        Self {
            incoming_tx: Some(tx),
            incoming_rx: Some(rx),
            context: PhantomData,
        }
    }

    pub async fn startup(&mut self, state: State) {
        let mut rx = self
            .incoming_rx
            .take()
            .expect("You cannot startup an actor twice.");
        tokio::spawn(async move {
            let mut local_state = state;

            while let Some((m, maybe_reply_handle)) = rx.recv().await {
                let resp = m.handle(&mut local_state).await;
                if let Some(reply_handle) = maybe_reply_handle {
                    let _ = reply_handle.send(resp); // Maybe handle this result if we fail
                                                     // to reply better?
                }
            }
        });
    }

    pub async fn call(&self, message: Messages) -> Result<Responses, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel::<Responses>();

        let _ = self
            .incoming_tx
            .as_ref()
            .expect("Actor must be started before attempting to send messages")
            .send((message, Some(tx)))
            .await;

        rx.await
    }

    pub async fn cast(&self, message: Messages) {
        let _ = self
            .incoming_tx
            .as_ref()
            .expect("Actor must be started before attempting to send messages")
            .send((message, None))
            .await;
    }
}

trait Registerable {
    fn register(self, registry: &mut Registry);
    fn fetch_from_registry(registry: &Registry) -> &Self;
}

impl<
        State: Send + Sync + 'static,
        Messages: Send + Sync + 'static + Handler<State, Responses>,
        Responses: Send + Sync + 'static,
    > Registerable for Actor<State, Messages, Responses>
{
    fn register(self, registry: &mut Registry) {
        let type_ = TypeId::of::<Actor<State, Messages, Responses>>();

        registry.add_child(type_, Arc::new(self));
    }

    fn fetch_from_registry(registry: &Registry) -> &Self {
        let type_ = TypeId::of::<Actor<State, Messages, Responses>>();
        println!("{}", std::any::type_name::<Actor<State, Messages, Responses>>());

        registry
            .get_child(type_)
            .unwrap()
            .downcast_ref::<Actor<State, Messages, Responses>>()
            .expect("Guaranteed by HashMap structure")
    }
}

#[async_trait]
pub trait Handler<State, Responses> {
    async fn handle(&self, state: &mut State) -> Responses;
}

#[cfg(test)]
mod tests {
    use crate::{Actor, Handler, registry::Registry, Registerable};
    use async_trait::async_trait;


    type TestActor = Actor::<MyState, Messages, Responses>;

    struct MyState;

    enum Messages {
        V4,
        V6,
    }

    #[derive(PartialEq)]
    enum Responses {
        V4,
        V6,
    }

    #[async_trait]
    impl Handler<MyState, Responses> for Messages {
        async fn handle(&self, _state: &mut MyState) -> Responses {
            match self {
                Messages::V4 => {
                    println!("Hello darkness my old friend");

                    Responses::V4
                }
                Messages::V6 => Responses::V6,
            }
        }
    }

    #[tokio::test]
    async fn create_actor_call_and_cast() {
        let mut registry = Registry::new();

        let mut background_worker = TestActor::new();
        background_worker.startup(MyState {}).await;
        background_worker.register(&mut registry);
        
        let local_ref = TestActor::fetch_from_registry(&registry);
        let res = local_ref.call(Messages::V4).await;
        let res_next = local_ref.call(Messages::V6).await;

        assert!(res.is_ok());
        assert!(res.unwrap() == Responses::V4);

        assert!(res_next.is_ok());
        assert!(res_next.unwrap() == Responses::V6);
    }
}
