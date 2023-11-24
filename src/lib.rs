use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

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

    pub async fn call(
        &mut self,
        message: Messages,
    ) -> Result<Responses, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel::<Responses>();

        let _ = self
            .incoming_tx
            .as_mut()
            .expect("Actor must be started before attempting to send messages")
            .send((message, Some(tx)))
            .await;

        rx.await
    }

    pub async fn cast(&mut self, message: Messages) {
        let _ = self
            .incoming_tx
            .as_mut()
            .expect("Actor must be started before attempting to send messages")
            .send((message, None))
            .await;
    }
}

#[async_trait]
pub trait Handler<State, Responses> {
    async fn handle(&self, state: &State) -> Responses;
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use crate::{Actor, Handler};

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
        async fn handle(&self, _state: &MyState) -> Responses {
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
        let mut background_worker = Actor::new();
        background_worker.startup(MyState {}).await;

        let res = background_worker.call(Messages::V4).await;
        let res_next = background_worker.call(Messages::V6).await;

        assert!(res.is_ok());
        assert!(res.unwrap() == Responses::V4);

        assert!(res_next.is_ok());
        assert!(res_next.unwrap() == Responses::V6);
    }
}
