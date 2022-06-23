use crate::{MessageType, Error};
use tokio::sync::mpsc::Sender;
use std::{net::SocketAddr, fmt::Debug};

pub struct ServerEvent<M: MessageType> {
    event: Event<M>,
    addr: SocketAddr,
    sender: Option<(u16, Sender<(u16, M)>)>,
}

impl<M: MessageType + Debug> Debug for ServerEvent<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerEvent").field("event", &self.event).field("addr", &self.addr).finish()
    }
}

impl<M: MessageType> ServerEvent<M> {
    pub(crate) fn connected(addr: SocketAddr) -> Self {
        Self {
            event: Event::Connected,
            sender: None,
            addr,
        }
    }

    pub(crate) fn message(addr: SocketAddr, req_id: u16, chn: Sender<(u16, M)>, msg: M) -> Self {
        Self {
            event: Event::Message(msg),
            sender: Some((req_id, chn)),
            addr,
        }
    }
}

impl<M: MessageType> ServerEvent<M> {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn into_addr_event(self) -> (SocketAddr, Event<M>) {
        (self.addr, self.event)
    }

    pub fn is_connected(&self) -> bool {
        matches!(&self.event, Event::Connected)
    }

    pub fn is_message(&self) -> bool {
        matches!(&self.event, Event::Message(_))
    }

    pub fn event(&self) -> &Event<M> {
        &self.event
    }

    pub fn into_event(self) -> Event<M> {
        self.event
    }

    pub async fn reply(self, msg: M) -> crate::Result<()> {
        let (rid, sender) = self
            .sender
            .expect("Cannot reply to non `Event::Message` event");
        sender.send((rid, msg)).await.map_err(|_| Error::Closed)?;
        Ok(())
    }
}

pub enum Event<M: MessageType> {
    Connected,
    Message(M),
}

impl<M: MessageType + Debug> Debug for Event<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected => write!(f, "Connected"),
            Self::Message(m) => f.debug_tuple("Message").field(m).finish(),
        }
    }
}
