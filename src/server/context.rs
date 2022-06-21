use std::net::SocketAddr;

pub struct ClientContext {
    pub(crate) addr: SocketAddr,
}

impl ClientContext {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
