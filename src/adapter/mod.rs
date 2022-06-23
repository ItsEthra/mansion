mod simple;
pub use simple::*;

#[cfg(feature = "encrypted")]
mod encrypted;
#[cfg(feature = "encrypted")]
pub use encrypted::*;

use crate::{Error, MessageType, SendSync};
use cursored::Cursored;

pub trait Adapter: SendSync {
    type Message: MessageType;

    #[cfg(feature = "server")]
    fn cloned(&self) -> Box<dyn Adapter<Message = Self::Message>>;

    fn encode(&self, msg: Self::Message, buf: &mut Cursored) -> Result<(), Error>;
    fn decode(&self, buf: &mut Cursored) -> Result<Self::Message, Error>;
}
