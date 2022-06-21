use super::{ClientContext, Error, MansionServer};
use crate::{CallbackFuture, Intercept, MessageType};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MansionServerBuilder<M: MessageType, F: CallbackFuture<Option<M>, Error>> {
    on_msg: Arc<Mutex<dyn FnMut(&ClientContext, M) -> F + Send + Sync + 'static>>,
    stack: Vec<Box<dyn Intercept>>,
}

impl<M: MessageType, F: CallbackFuture<Option<M>, Error>> MansionServerBuilder<M, F> {
    pub fn new(on_message: impl FnMut(&ClientContext, M) -> F + Send + Sync + 'static) -> Self {
        Self {
            on_msg: Arc::new(Mutex::new(on_message)),
            stack: vec![],
        }
    }

    pub fn intercept(mut self, inc: impl Intercept) -> Self {
        self.stack.push(Box::new(inc));
        self
    }

    pub fn finish(self) -> MansionServer<M, F> {
        MansionServer {
            on_msg: self.on_msg,
            stack: self.stack.into(),
        }
    }
}
