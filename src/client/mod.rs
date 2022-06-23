mod req_map;
pub(crate) use req_map::*;
mod builder;
pub use builder::*;

use crate::{Adapter, Error, MessageType};
use cursored::Cursored;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::mpsc::{Receiver, Sender},
};

pub struct MansionClient<M: MessageType> {
    req_map: Arc<RequestMap<M>>,
    send_queue: Sender<(u16, M)>,
}

impl<M: MessageType> MansionClient<M> {
    pub fn builder() -> MansionClientBuilder<M> {
        MansionClientBuilder::new()
    }

    /// Sends the message and waits for the response from the server.
    pub async fn send_wait(&self, msg: M) -> crate::Result<M> {
        let (rid, wait) = self.req_map.push().await?;
        self.send_queue
            .send((rid, msg))
            .await
            .map_err(|_| Error::Closed)?;
        wait.await.map_err(|_| Error::Closed)
    }

    /// Sends the message without waiting for the response from the server.
    pub async fn send_forget(&self, msg: M) -> crate::Result<()> {
        self.send_queue
            .send((0, msg))
            .await
            .map_err(|_| Error::Closed)?;
        Ok(())
    }
}

pub(crate) struct Connection<M: MessageType> {
    req_map: Arc<RequestMap<M>>,
    adapter: Arc<dyn Adapter<Message = M>>,
}

pub(crate) async fn read_half<M: MessageType>(
    cn: Arc<Connection<M>>,
    mut rx: OwnedReadHalf,
) -> crate::Result<()> {
    let mut buf = Vec::new();

    loop {
        let req_id = rx.read_u16().await?;
        let len = rx.read_u32().await? as usize;
        buf.resize(len, 0);

        let mut c = Cursored::new(buf);
        let msg = cn.adapter.decode(&mut c)?;

        cn.req_map.notify(req_id, msg).await?;
        buf = c.into_inner();
    }
}

pub(crate) async fn write_half<M: MessageType>(
    cn: Arc<Connection<M>>,
    mut queue: Receiver<(u16, M)>,
    mut tx: OwnedWriteHalf,
) -> crate::Result<()> {
    let mut buf = Vec::new();

    while let Some((id, msg)) = queue.recv().await {
        tx.write_u16(id).await?;

        let mut c = Cursored::new(buf);
        cn.adapter.encode(msg, &mut c)?;
        buf = c.into_inner();

        tx.write_u32(buf.len() as u32).await?;
        tx.write_all(&buf[..]).await?;

        buf.clear();
    }

    Err(Error::Closed)
}
