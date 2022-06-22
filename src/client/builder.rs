use super::{InterceptStack, MansionClient, ClientIntercept};
use crate::{req_map::RequestMap, MessageType};
use std::{marker::PhantomData, sync::Arc};
use tokio::net::{TcpStream, ToSocketAddrs};
use flume::unbounded;

pub struct MansionClientBuilder<M: MessageType> {
    stack: Vec<Box<dyn ClientIntercept>>,
    ph_m: PhantomData<M>,
}

impl<M: MessageType> MansionClientBuilder<M> {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            ph_m: PhantomData,
        }
    }

    pub fn intercept(mut self, inc: impl ClientIntercept) -> Self {
        self.stack.push(Box::new(inc));
        self
    }

    pub async fn connect(self, addr: impl ToSocketAddrs) -> crate::Result<MansionClient<M>> {
        let mut tcp = TcpStream::connect(addr).await?;

        for i in self.stack.iter() {
            i.on_connect(&mut tcp).await?;
        }

        let (rx, tx) = tcp.into_split();
        let (send_queue, recv) = unbounded();
        let req_map = Arc::new(RequestMap::new());
        let stack: InterceptStack = self.stack.into();

        tokio::spawn(super::read_half::<M>(req_map.clone(), stack.clone(), rx));
        tokio::spawn(super::write_half::<M>(recv, stack.clone(), tx));

        Ok(MansionClient {
            send_queue,
            req_map,
        })
    }
}
