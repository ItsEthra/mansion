#![allow(where_clauses_object_safety)]

// #[cfg(all(not(feature = "client"), not(feature = "server")))]
// compile_error!("You must enable either `client` or `server` features");

// #[cfg(all(feature = "client", feature = "server"))]
// compile_error!("You should not use both `server` and `client` features");

// #[cfg(any(feature = "client", feature = "server"))]
mod error;
// #[cfg(any(feature = "client", feature = "server"))]
pub use error::*;

// #[cfg(feature = "client")]
mod req_map;

// #[cfg(feature = "client")]
pub mod client;
// #[cfg(feature = "server")]
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
