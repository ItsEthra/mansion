use std::net::SocketAddr;
use flume::Sender;

pub struct ClientContext {
    pub(crate) addr: SocketAddr,
    pub(crate) stop: Sender<()>,
}

impl ClientContext {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn close(&self) {
        let _ = self.stop.send_async(()).await;
    }
}
