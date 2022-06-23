use tokio::{
    net::{ToSocketAddrs, TcpListener},
    sync::mpsc::channel,
};
use super::{MansionServer, Connection, read_half, ServerEvent, write_half};
use crate::{Adapter, MessageType};
use flume::unbounded;
use std::sync::Arc;

pub struct MansionServerBuilder<M: MessageType> {
    adapter: Option<Box<dyn Adapter<Message = M>>>,
}

impl<M: MessageType> MansionServerBuilder<M> {
    pub fn new() -> Self {
        Self { adapter: None }
    }

    pub fn with_adapter(mut self, adapter: impl Adapter<Message = M>) -> Self {
        self.adapter = Some(Box::new(adapter));
        self
    }

    pub async fn listen(self, addr: impl ToSocketAddrs) -> crate::Result<MansionServer<M>> {
        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");
        let adapter = self.adapter.expect("Adapter must be set");

        let (ein, eout) = unbounded();

        tokio::spawn(async move {
            loop {
                if let Ok((tcp, addr)) = listener.accept().await {
                    let _ = ein.send_async(ServerEvent::connected(addr)).await;

                    let (send_queue, recv_queue) = channel(8);
                    let (rx, tx) = tcp.into_split();
                    let cn = Arc::new(Connection {
                        adapter: adapter.cloned().into(),
                        send_queue,
                        addr,
                    });

                    tokio::spawn(read_half(cn.clone(), ein.clone(), rx));
                    tokio::spawn(write_half(cn, recv_queue, tx));
                }
            }
        });

        Ok(MansionServer {
            recv: eout,
        })
    }
}
