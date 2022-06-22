use crate::SendSync;
use flume::Sender;
use std::{net::SocketAddr, sync::Arc};

pub struct ClientContext<S: SendSync> {
    pub(crate) addr: SocketAddr,
    pub(crate) stop: Sender<()>,
    pub(crate) state: Arc<S>,
}

impl<S: SendSync> ClientContext<S> {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn state(&self) -> &S {
        &*self.state
    }

    pub async fn close(&self) {
        let _ = self.stop.send_async(()).await;
    }
}
