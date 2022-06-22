use super::{ClientContext, Error, MansionServer};
use crate::{CallbackFuture, Intercept, MessageType, SendSync};
use std::{sync::Arc, net::SocketAddr};

pub struct MansionServerBuilder<M: MessageType, S: SendSync, F1: CallbackFuture<Option<M>, Error>, F2: CallbackFuture<(), Error>> {
    on_msg: Arc<dyn Fn(Arc<ClientContext<S>>, M) -> F1 + Send + Sync + 'static>,
    on_dc: Option<Box<dyn Fn(Arc<S>, SocketAddr) -> F2 + Send + Sync + 'static>>,
    stack: Vec<Box<dyn Intercept>>,
    state: Option<S>,
}

impl<M: MessageType, S: SendSync, F1: CallbackFuture<Option<M>, Error>, F2: CallbackFuture<(), Error>>
    MansionServerBuilder<M, S, F1, F2>
{
    pub fn new(
        on_message: impl Fn(Arc<ClientContext<S>>, M) -> F1 + Send + Sync + 'static,
    ) -> Self {
        Self {
            on_msg: Arc::new(on_message),
            stack: vec![],
            state: None,
            on_dc: None,
        }
    }

    pub fn on_disconnect(mut self, on_disconnect: impl Fn(Arc<S>, SocketAddr) -> F2 + Send + Sync + 'static) -> Self {
        self.on_dc = Some(Box::new(on_disconnect));
        self
    }

    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    pub fn intercept(mut self, inc: impl Intercept) -> Self {
        self.stack.push(Box::new(inc));
        self
    }

    pub fn finish(self) -> MansionServer<M, S, F1, F2> {
        MansionServer {
            on_msg: self.on_msg,
            stack: self.stack.into(),
            state: self
                .state
                .expect("State must be set. If you don't want to use it set it to `()`.")
                .into(),
            on_dc: Arc::new(self.on_dc.expect("`on_disconnect` must be set, even if you don't use it.")),
        }
    }
}
