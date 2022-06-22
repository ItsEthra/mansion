use crate::SendSync;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

pub(crate) type InterceptStack = Arc<[Box<dyn ClientIntercept>]>;

#[async_trait]
pub trait ClientIntercept: SendSync {
    async fn on_connect(&self, _tcp: &mut TcpStream) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_pre_recv(
        &self,
        _rx: &mut OwnedReadHalf,
        _req_id: u16,
        _len: usize,
        _buf: &mut Vec<u8>,
    ) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_post_recv(
        &self,
        _rx: &mut OwnedReadHalf,
        _req_id: u16,
        _len: usize,
        _buf: &mut Vec<u8>,
    ) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_pre_send(
        &self,
        _tx: &mut OwnedWriteHalf,
        _req_id: u16,
        _len: usize,
        _buf: &mut Vec<u8>,
    ) -> Result<(), super::Error> {
        Ok(())
    }

    async fn on_post_send(
        &self,
        _tx: &mut OwnedWriteHalf,
        _req_id: u16,
        _len: usize,
        _buf: &mut Vec<u8>,
    ) -> Result<(), super::Error> {
        Ok(())
    }
}
