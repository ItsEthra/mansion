use std::{
    collections::HashMap,
    sync::atomic::{AtomicU16, Ordering},
};
use tokio::sync::{
    oneshot::{self, Receiver, Sender},
    Mutex,
};

pub struct RequestMap<M> {
    current: AtomicU16,
    map: Mutex<HashMap<u16, Sender<M>>>,
}

impl<M> RequestMap<M> {
    pub fn new() -> Self {
        Self {
            current: AtomicU16::new(0),
            map: Mutex::default(),
        }
    }

    /// Returns `None` if id collision occured.
    pub async fn push(&self) -> Option<(u16, Receiver<M>)> {
        let this = self.current.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        let mut lock = self.map.lock().await;
        if lock.contains_key(&this) {
            None
        } else {
            lock.insert(this, tx);
            Some((this, rx))
        }
    }

    /// Returns `true` if other half was successfully notified.
    /// Returns `false` if channel was closed or didn't exist.
    pub async fn notify(&self, id: u16, msg: M) -> bool {
        if let Some(s) = self.map.lock().await.remove(&id) {
            s.send(msg).is_ok()
        } else {
            false
        }
    }
}
