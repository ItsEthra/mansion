mod builder;
pub use builder::*;
mod event;
pub use event::*;

use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{Sender, Receiver},
};
use crate::{MessageType, Adapter, Error};
use std::{sync::Arc, net::SocketAddr};
use cursored::Cursored;

pub struct MansionServer<M: MessageType> {
    recv: flume::Receiver<ServerEvent<M>>,
}

impl<M: MessageType> MansionServer<M> {
    pub fn builder() -> MansionServerBuilder<M> {
        MansionServerBuilder::new()
    }

    pub async fn message(&self) -> Option<ServerEvent<M>> {
        self.recv.recv_async().await.ok()
    }
}

pub(crate) struct Connection<M: MessageType> {
    addr: SocketAddr,
    send_queue: Sender<(u16, M)>,
    adapter: Arc<dyn Adapter<Message = M>>,
}

pub(crate) async fn read_half<M: MessageType>(
    cn: Arc<Connection<M>>,
    events: flume::Sender<ServerEvent<M>>,
    mut rx: OwnedReadHalf,
) -> crate::Result<()> {
    let out = async {
        let mut buf = Vec::new();

        loop {
            let req_id = rx.read_u16().await?;
            let len = rx.read_u32().await? as usize;
            buf.resize(len, 0);

            rx.read_exact(&mut buf).await?;

            let mut c = Cursored::new(buf);
            let msg = cn.adapter.decode(&mut c)?;
            buf = c.into_inner();

            events
                .send_async(ServerEvent::message(
                    cn.addr,
                    req_id,
                    cn.send_queue.clone(),
                    msg,
                ))
                .await
                .map_err(|_| Error::Closed)?;
        }
    }
    .await;

    events
        .send_async(ServerEvent::disconnected(cn.addr))
        .await
        .map_err(|_| Error::Closed)?;

    out
}

pub(crate) async fn write_half<M: MessageType>(
    cn: Arc<Connection<M>>,
    mut queue: Receiver<(u16, M)>,
    mut tx: OwnedWriteHalf,
) -> crate::Result<()> {
    let mut buf = Vec::new();

    while let Some((rid, msg)) = queue.recv().await {
        tx.write_u16(rid).await?;

        let mut c = Cursored::new(buf);
        cn.adapter.encode(msg, &mut c)?;
        buf = c.into_inner();

        tx.write_u32(buf.len() as u32).await?;
        tx.write_all(&buf[..]).await?;
    }

    Err(Error::Closed)
}
