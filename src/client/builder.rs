use super::{read_half, write_half, Connection, MansionClient, RequestMap};
use crate::{Adapter, MessageType};
use std::sync::Arc;
use tokio::{
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc::channel,
};

pub struct MansionClientBuilder<M: MessageType> {
    adapter: Option<Arc<dyn Adapter<Message = M>>>,
}

impl<M: MessageType> MansionClientBuilder<M> {
    pub fn new() -> Self {
        Self { adapter: None }
    }

    pub fn with_adapter(mut self, adapter: impl Adapter<Message = M>) -> Self {
        self.adapter = Some(Arc::new(adapter));
        self
    }

    pub async fn connect(self, addr: impl ToSocketAddrs) -> crate::Result<MansionClient<M>> {
        let req_map = Arc::new(RequestMap::new());
        let (rx, tx) = TcpStream::connect(addr).await?.into_split();
        let cn = Arc::new(Connection::<M> {
            adapter: self.adapter.expect("Adapter must be set"),
            req_map: req_map.clone(),
        });

        let (send_queue, queue) = channel(8);

        tokio::spawn(read_half(cn.clone(), rx));
        tokio::spawn(write_half(cn, queue, tx));

        Ok(MansionClient {
            send_queue,
            req_map,
        })
    }
}
