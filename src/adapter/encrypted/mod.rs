#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::*;

pub trait EncryptionTarget: Sized {
    fn is_request(&self) -> bool;
    fn is_response(&self) -> bool;

    fn request() -> Self;
    fn response() -> Self;
}
