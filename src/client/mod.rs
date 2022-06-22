mod builder;
pub use builder::*;
mod intercept;
pub(crate) use intercept::*;

use crate::{req_map::RequestMap, Error, MessageType};
use flume::{Receiver, Sender};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

pub struct MansionClient<M: MessageType> {
    req_map: Arc<RequestMap<M>>,
    send_queue: Sender<(u16, M)>,
}

impl<M: MessageType> MansionClient<M> {
    pub fn builder() -> MansionClientBuilder<M> {
        MansionClientBuilder::new()
    }

    pub async fn send_wait(&self, msg: M) -> crate::Result<M> {
        if let Some((id, waiter)) = self.req_map.push().await {
            self.send_queue
                .send_async((id, msg))
                .await
                .map_err(|_| Error::Closed)?;
            waiter.await.map_err(|_| Error::Closed)
        } else {
            Err(Error::IdOverflow)
        }
    }

    pub async fn send_ignore(&self, msg: M) -> crate::Result<()> {
        self.send_queue
            .send_async((0, msg))
            .await
            .map_err(|_| Error::Closed)?;
        Ok(())
    }
}

pub(crate) async fn read_half<M: MessageType>(
    req_map: Arc<RequestMap<M>>,
    stack: InterceptStack,
    mut rx: OwnedReadHalf,
) -> crate::Result<()> {
    let mut buf = vec![];
    loop {
        let len = rx.read_u32().await? as usize;
        buf.resize(len, 0);

        let req_id = rx.read_u16().await?;

        for i in stack.iter() {
            i.on_pre_recv(&mut rx, req_id, len, &mut buf).await?;
        }

        rx.read_exact(&mut buf[..]).await?;

        for i in stack.iter() {
            i.on_post_recv(&mut rx, req_id, len, &mut buf).await?;
        }

        let msg = bincode::deserialize::<M>(&buf[..])?;

        if !req_map.notify(req_id, msg).await {
            break Err(Error::Closed);
        }
    }
}

pub(crate) async fn write_half<M: MessageType>(
    send_queue: Receiver<(u16, M)>,
    stack: InterceptStack,
    mut tx: OwnedWriteHalf,
) -> crate::Result<()> {
    while let Ok((req_id, m)) = send_queue.recv_async().await {
        // @TODO: Serialize into buffer to prevert re-allocation.
        let mut data = bincode::serialize(&m)?;

        tx.write_u32(data.len() as u32).await?;
        tx.write_u16(req_id).await?;

        for i in stack.iter() {
            i.on_pre_send(&mut tx, req_id, data.len(), &mut data)
                .await?;
        }

        tx.write_all(&data[..]).await?;

        for i in stack.iter() {
            i.on_post_send(&mut tx, req_id, data.len(), &mut data)
                .await?;
        }
    }

    Err(Error::Closed)
}
