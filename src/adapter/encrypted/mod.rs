#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::*;

pub trait EncryptionTarget: Sized {
    fn request(&self) -> bool;
    fn response() -> Self;
}