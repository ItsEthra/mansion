use super::{ClientContext, Error, MansionServer};
use crate::{CallbackFuture, Intercept, MessageType, SendSync};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MansionServerBuilder<M: MessageType, S: SendSync, F: CallbackFuture<Option<M>, Error>> {
    on_msg: Arc<Mutex<dyn FnMut(Arc<ClientContext<S>>, M) -> F + Send + Sync + 'static>>,
    stack: Vec<Box<dyn Intercept>>,
    state: Option<S>,
}

impl<M: MessageType, S: SendSync, F: CallbackFuture<Option<M>, Error>>
    MansionServerBuilder<M, S, F>
{
    pub fn new(
        on_message: impl FnMut(Arc<ClientContext<S>>, M) -> F + Send + Sync + 'static,
    ) -> Self {
        Self {
            on_msg: Arc::new(Mutex::new(on_message)),
            stack: vec![],
            state: None,
        }
    }

    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    pub fn intercept(mut self, inc: impl Intercept) -> Self {
        self.stack.push(Box::new(inc));
        self
    }

    pub fn finish(self) -> MansionServer<M, S, F> {
        MansionServer {
            on_msg: self.on_msg,
            stack: self.stack.into(),
            state: self
                .state
                .expect("State must be set. If you don't want to use it set it to `()`.")
                .into(),
        }
    }
}
