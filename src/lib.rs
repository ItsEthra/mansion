#![allow(where_clauses_object_safety)]

mod req_map;

mod intercept;
pub use intercept::*;
mod error;
pub use error::*;

pub mod client;
pub mod server;

pub use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub trait SendSync: Send + Sync + 'static {}
impl<T> SendSync for T where T: Send + Sync + 'static {}

pub trait MessageType: SendSync + Serialize + DeserializeOwned + 'static {}
impl<T> MessageType for T where T: SendSync + Serialize + DeserializeOwned + 'static {}

pub trait CallbackFuture<M, E>: SendSync + Future<Output = std::result::Result<M, E>> {}
impl<M, E, T> CallbackFuture<M, E> for T where
    T: SendSync + Future<Output = std::result::Result<M, E>>
{
}
