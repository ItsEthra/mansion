mod builder;
pub use builder::*;
mod context;
pub use context::*;
use flume::{bounded, unbounded, Receiver, Sender};

use crate::{CallbackFuture, Error, InterceptStack, MessageType, SendSync};
use std::{future::Future, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, ToSocketAddrs,
    },
};

pub struct MansionServer<M: MessageType, S: SendSync, F: CallbackFuture<Option<M>, Error>> {
    on_msg: Arc<dyn Fn(Arc<ClientContext<S>>, M) -> F + Send + Sync + 'static>,
    stack: InterceptStack,
    state: Arc<S>,
}

impl<M: MessageType, S: SendSync, F: CallbackFuture<Option<M>, Error>> MansionServer<M, S, F> {
    pub fn builder(
        on_message: impl Fn(Arc<ClientContext<S>>, M) -> F + Send + Sync + 'static,
    ) -> MansionServerBuilder<M, S, F> {
        MansionServerBuilder::new(on_message)
    }

    pub async fn listen(self, addr: impl ToSocketAddrs) -> crate::Result<()> {
        let listen = TcpListener::bind(addr).await?;

        loop {
            if let Ok((mut tcp, addr)) = listen.accept().await {
                let mut skip = false;
                for i in self.stack.iter() {
                    if i.on_connect(&mut tcp).await.is_err() {
                        skip = true;
                    }
                }
                if skip {
                    continue;
                }

                let (rx, tx) = tcp.into_split();
                let (send, recv) = unbounded();
                let (stop, shut) = bounded(1);

                let cn = Arc::new(Connection {
                    stack: self.stack.clone(),
                    ctx: Arc::new(ClientContext::<S> {
                        state: self.state.clone(),
                        addr,
                        stop,
                    }),
                });

                tokio::spawn(read_half(
                    self.on_msg.clone(),
                    cn.clone(),
                    send,
                    shut.clone(),
                    rx,
                ));
                tokio::spawn(write_half(cn, recv, shut, tx));
            }
        }
    }
}

struct Connection<S: SendSync> {
    stack: InterceptStack,
    ctx: Arc<ClientContext<S>>,
}

async fn read_half<S: SendSync, M: MessageType, F: CallbackFuture<Option<M>, Error>>(
    on_msg: Arc<dyn Fn(Arc<ClientContext<S>>, M) -> F + Send + Sync + 'static>,
    cn: Arc<Connection<S>>,
    send: Sender<(u16, M)>,
    shut: Receiver<()>,
    mut rx: OwnedReadHalf,
) -> crate::Result<()> {
    let mut buf = vec![];

    loop {
        let len = cancel_on_close(&shut, rx.read_u32()).await?? as usize;
        buf.resize(len, 0);

        let req_id = cancel_on_close(&shut, rx.read_u16()).await??;

        for i in cn.stack.iter() {
            i.on_pre_recv(&mut rx, req_id, len, &mut buf).await?;
        }

        cancel_on_close(&shut, rx.read_exact(&mut buf[..])).await??;

        for i in cn.stack.iter() {
            i.on_post_recv(&mut rx, req_id, len, &mut buf).await?;
        }

        let msg = bincode::deserialize::<M>(&buf[..])?;
        if let Some(msg) = on_msg(cn.ctx.clone(), msg).await? {
            send.send_async((req_id, msg))
                .await
                .map_err(|_| Error::Closed)?;
        }
    }
}

async fn write_half<S: SendSync, M: MessageType>(
    cn: Arc<Connection<S>>,
    recv: Receiver<(u16, M)>,
    shut: Receiver<()>,
    mut tx: OwnedWriteHalf,
) -> crate::Result<()> {
    while let Ok((req_id, msg)) = recv.recv_async().await {
        let mut buf = bincode::serialize(&msg)?;

        cancel_on_close(&shut, tx.write_u32(buf.len() as u32)).await??;
        cancel_on_close(&shut, tx.write_u16(req_id)).await??;

        for i in cn.stack.iter() {
            i.on_pre_send(&mut tx, req_id, buf.len(), &mut buf).await?;
        }

        cancel_on_close(&shut, tx.write_all(&buf[..])).await??;

        for i in cn.stack.iter() {
            i.on_post_send(&mut tx, req_id, buf.len(), &mut buf).await?;
        }
    }

    Err(Error::Closed)
}

async fn cancel_on_close<T>(
    close: &Receiver<()>,
    other: impl Future<Output = T>,
) -> crate::Result<T> {
    tokio::select! {
        val = other => {
            Ok(val)
        }
        _ = close.recv_async() => {
            Err(Error::Closed)
        }
    }
}
