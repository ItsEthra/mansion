use crate::SendSync;
use async_trait::async_trait;
use std::{sync::Arc, net::SocketAddr};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

pub(crate) type InterceptStack = Arc<[Box<dyn ServerIntercept>]>;

#[async_trait]
pub trait ServerIntercept: SendSync {
    async fn on_connect(&self, _tcp: &mut TcpStream, _addr: SocketAddr) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_disconnect(&self, _addr: SocketAddr) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_recv(
        &self,
        _rx: &mut OwnedReadHalf,
        _addr: SocketAddr,
        _req_id: u16,
        _buf: &mut Vec<u8>,
    ) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_send(
        &self,
        _tx: &mut OwnedWriteHalf,
        _addr: SocketAddr,
        _req_id: u16,
        _buf: &mut Vec<u8>,
    ) -> Result<(), super::Error> {
        Ok(())
    }
}
